package platform

import (
	"bytes"
	"context"
	"encoding/base64"
	"encoding/hex"
	"encoding/json"
	"errors"
	"fmt"
	"log/slog"
	"net/http"
	"net/url"
	"slices"
	"strings"
	"sync"
	"time"

	"connectrpc.com/connect"
	wf "github.com/lesomnus/wafl/kubernetes/gen/wafl"
	"google.golang.org/protobuf/proto"
	core "k8s.io/api/core/v1"
	meta "k8s.io/apimachinery/pkg/apis/meta/v1"
)

// Catalog reads the supervised runners. Observations are live and are never
// authoritative evidence for releasing a reservation or retrying a mutation.
type Catalog struct {
	Clients   *Clients
	Namespace string
	Timeout   time.Duration
	Parallel  int
	HTTP      *http.Client
}
type member struct {
	name, url   string
	unavailable bool
}
type observation[T any] struct {
	member member
	runner *wf.Runner
	value  T
}
type catalogResult[T any] struct {
	items          []observation[T]
	warnings       []string
	selected       []byte
	matched        int
	failedSelected bool
}
type catalogResource interface {
	proto.Message
	GetId() []byte
}

func catalogError(code connect.Code, message string) error {
	return connect.NewError(code, errors.New(message))
}
func catalogSession(header http.Header) ([]byte, error) {
	values := header.Values("wafl-session")
	if len(values) == 0 {
		return nil, nil
	}
	if len(values) != 1 {
		return nil, catalogError(connect.CodeInvalidArgument, "duplicate wafl-session")
	}
	value := values[0]
	if len(value) != 36 || value[8] != '-' || value[13] != '-' || value[18] != '-' || value[23] != '-' {
		return nil, catalogError(connect.CodeInvalidArgument, "wafl-session must be a Runner UUID")
	}
	id, e := hex.DecodeString(strings.ReplaceAll(value, "-", ""))
	if e != nil || !validID(id, 8) {
		return nil, catalogError(connect.CodeInvalidArgument, "wafl-session must be a Runner UUID")
	}
	return id, nil
}
func (c *Catalog) client(m member) *RunnerClient {
	h := c.HTTP
	if h == nil {
		h = http.DefaultClient
	}
	return runnerClient(m.url, h)
}
func (c *Catalog) timeout() time.Duration {
	if c.Timeout > 0 {
		return c.Timeout
	}
	return 3 * time.Second
}
func (c *Catalog) members(ctx context.Context) ([]member, error) {
	nodes, e := c.Clients.Kube.CoreV1().Nodes().List(ctx, meta.ListOptions{LabelSelector: Runtime + "=wasmtime"})
	if e != nil {
		return nil, rpcError(e)
	}
	// Virtual nodes from another platform namespace are outside this endpoint's
	// scope. Retained nodes remain visible during drain and runner failure.
	slices.SortFunc(nodes.Items, func(a, b core.Node) int { return strings.Compare(a.Name, b.Name) })
	out := []member{}
	seen := map[string]bool{}
	var physical map[string]string
	for _, node := range nodes.Items {
		if strings.HasPrefix(node.Annotations[RunnerPod], c.Namespace+"/") {
			pods, err := c.Clients.Kube.CoreV1().Pods(c.Namespace).List(ctx, meta.ListOptions{})
			if err != nil {
				return nil, rpcError(err)
			}
			physical = map[string]string{}
			for _, p := range pods.Items {
				physical[p.Namespace+"/"+p.Name] = string(p.UID)
			}
			break
		}
	}
	for _, n := range nodes.Items {
		ns := n.Annotations[Group+"/namespace"]
		if ns == "" {
			ns, _, _ = strings.Cut(n.Annotations[RunnerPod], "/")
		}
		if ns != c.Namespace {
			continue
		}
		endpoint := strings.TrimRight(n.Annotations[Runner], "/")
		unavailable := false
		if pod := n.Annotations[RunnerPod]; pod != "" {
			uid := n.Annotations[Group+"/runnerUID"]
			unavailable = uid == "" || physical[pod] != uid
		}
		if endpoint != "" && seen[endpoint] && !unavailable {
			continue
		}
		if !unavailable {
			seen[endpoint] = true
		}
		out = append(out, member{name: n.Name, url: endpoint, unavailable: unavailable})
	}
	return out, nil
}

// Bounded workers and a separate deadline for every runner isolate failures.
// Successful pages from a runner are retained even if a later page fails.
func collect[T any](ctx context.Context, c *Catalog, header http.Header, operation string, fn func(context.Context, member, *wf.Runner) ([]T, error)) (*catalogResult[T], error) {
	session, e := catalogSession(header)
	if e != nil {
		return nil, e
	}
	nodes, e := c.members(ctx)
	if e != nil {
		return nil, e
	}
	type part struct {
		values []T
		runner *wf.Runner
		err    error
	}
	parts := make([]part, len(nodes))
	jobs := make(chan int, len(nodes))
	for i := range nodes {
		jobs <- i
	}
	close(jobs)
	parallel := c.Parallel
	if parallel <= 0 {
		parallel = 16
	}
	parallel = min(parallel, len(nodes))
	var workers sync.WaitGroup
	for range parallel {
		workers.Add(1)
		go func() {
			defer workers.Done()
			for i := range jobs {
				if ctx.Err() != nil {
					return
				}
				node := nodes[i]
				step, cancel := context.WithTimeout(ctx, c.timeout())
				func() {
					defer cancel()
					if node.unavailable {
						parts[i].err = catalogError(connect.CodeUnavailable, "runner Pod identity is no longer present")
						return
					}
					u, err := url.Parse(node.url)
					if err != nil || u.Host == "" || u.User != nil || (u.Scheme != "http" && u.Scheme != "https") {
						parts[i].err = catalogError(connect.CodeFailedPrecondition, "invalid runner endpoint")
						return
					}
					r, err := c.client(node).Status(step)
					if err == nil && !validID(r.GetId(), 8) {
						err = catalogError(connect.CodeDataLoss, "invalid runner identity")
					}
					if err != nil {
						parts[i].err = err
						return
					}
					if len(session) > 0 && !bytes.Equal(session, r.Id) {
						return
					}
					r.Alias = node.name
					r.Control = node.url
					parts[i].runner = r
					parts[i].values, parts[i].err = fn(step, node, r)
				}()
			}
		}()
	}
	workers.Wait()
	if ctx.Err() != nil {
		return nil, contextError(ctx.Err())
	}
	out := &catalogResult[T]{selected: session}
	for i, p := range parts {
		if p.runner != nil {
			out.matched++
		}
		if p.err != nil {
			if p.runner != nil {
				out.failedSelected = true
			}
			code := connect.CodeOf(p.err)
			if errors.Is(p.err, context.DeadlineExceeded) {
				code = connect.CodeDeadlineExceeded
			}
			warning := fmt.Sprintf("runner %s query incomplete: %s", nodes[i].name, code)
			slog.Warn("runner query incomplete", "runner", nodes[i].name, "operation", operation, "code", code.String())
			out.warnings = append(out.warnings, warning)
		}
		for _, v := range p.values {
			out.items = append(out.items, observation[T]{nodes[i], p.runner, v})
		}
	}
	return out, nil
}
func contextError(e error) error {
	code := connect.CodeCanceled
	if errors.Is(e, context.DeadlineExceeded) {
		code = connect.CodeDeadlineExceeded
	}
	return catalogError(code, "request interrupted")
}
func warningHeaders(header http.Header, warnings []string) {
	// Bound RPC metadata even during a fleet-wide outage. Every failure is still
	// written to the controller log; never copy arbitrary upstream error bodies.
	used, shown := 0, 0
	for _, warning := range warnings {
		if used+len(warning) > 3500 {
			break
		}
		header.Add("wafl-warning", warning)
		used += len(warning)
		shown++
	}
	if shown < len(warnings) {
		header.Add("wafl-warning", fmt.Sprintf("%d additional runner failures; see controller logs", len(warnings)-shown))
	}
}
func observedResponse[T any, V any](value *V, result *catalogResult[T]) *connect.Response[V] {
	r := connect.NewResponse(value)
	warningHeaders(r.Header(), result.warnings)
	return r
}
func missing[T any](result *catalogResult[T]) error {
	if len(result.selected) > 0 && result.matched > 0 && !result.failedSelected {
		return catalogError(connect.CodeNotFound, "resource not found in selected runner")
	}
	if len(result.warnings) > 0 {
		return catalogError(connect.CodeUnavailable, "target could not be located because a runner query failed")
	}
	if len(result.selected) > 0 && result.matched == 0 {
		return catalogError(connect.CodeFailedPrecondition, "selected runner session is no longer available")
	}
	return catalogError(connect.CodeNotFound, "resource not found in supervised runners")
}
func unique[T any](result *catalogResult[T]) (observation[T], error) {
	var zero observation[T]
	if len(result.items) == 0 {
		return zero, missing(result)
	}
	if len(result.items) > 1 {
		return zero, catalogError(connect.CodeFailedPrecondition, "reference matches multiple runners; specify wafl-session and an ID")
	}
	return result.items[0], nil
}

type catalogCursor struct {
	Version     int
	Kind, Scope string
	ID, Session []byte
	Reverse     bool
}
type catalogPage struct {
	size   int
	cursor catalogCursor
}

func newCatalogPage(namespace, kind string, header http.Header, filters any, size int32, after string, reverse bool, domain byte) (catalogPage, error) {
	session, e := catalogSession(header)
	if e != nil {
		return catalogPage{}, e
	}
	n := int(size)
	if n <= 0 {
		n = 100
	}
	n = min(n, 1000)
	p := catalogPage{size: n, cursor: catalogCursor{Version: 1, Kind: kind, Scope: digest([]any{namespace, kind, session, filters}), Reverse: reverse}}
	if after != "" {
		var old catalogCursor
		b, e := base64.RawURLEncoding.DecodeString(after)
		if len(after) > 1024 || e != nil || json.Unmarshal(b, &old) != nil || old.Version != 1 || old.Kind != kind || old.Scope != p.cursor.Scope || old.Reverse != reverse || !validID(old.ID, domain) || (len(old.Session) > 0 && !validID(old.Session, 8)) {
			return catalogPage{}, catalogError(connect.CodeInvalidArgument, "invalid controller cursor for this scope, filter or order")
		}
		p.cursor = old
	}
	return p, nil
}
func (p catalogPage) beyond(id, session []byte) bool {
	if len(p.cursor.ID) == 0 {
		return true
	}
	cmp := bytes.Compare(id, p.cursor.ID)
	if cmp == 0 {
		cmp = bytes.Compare(session, p.cursor.Session)
	}
	if p.cursor.Reverse {
		return cmp < 0
	}
	return cmp > 0
}
func pageItems[T catalogResource](p catalogPage, items []observation[T], tie bool) ([]T, string) {
	key := func(v observation[T]) []byte {
		if tie {
			return v.runner.Id
		}
		return nil
	}
	slices.SortStableFunc(items, func(a, b observation[T]) int {
		cmp := bytes.Compare(a.value.GetId(), b.value.GetId())
		if cmp == 0 {
			cmp = bytes.Compare(key(a), key(b))
		}
		if p.cursor.Reverse {
			return -cmp
		}
		return cmp
	})
	filtered := []observation[T]{}
	for _, v := range items {
		if p.beyond(v.value.GetId(), key(v)) {
			filtered = append(filtered, v)
		}
	}
	next := ""
	if len(filtered) > p.size {
		last := filtered[p.size-1]
		cursor := p.cursor
		cursor.ID = last.value.GetId()
		cursor.Session = key(last)
		b, _ := json.Marshal(cursor)
		next = base64.RawURLEncoding.EncodeToString(b)
		filtered = filtered[:p.size]
	}
	values := make([]T, 0, len(filtered))
	for _, v := range filtered {
		values = append(values, v.value)
	}
	return values, next
}

// Read only as far as this runner can contribute to the requested merged page.
// Upstream cursors remain private to that runner and are never reused globally.
func scan[T catalogResource](ctx context.Context, p catalogPage, session []byte, domain byte, match func(T) bool, fetch func(string) ([]T, string, error)) ([]T, error) {
	values := []T{}
	after := ""
	seen := map[string]bool{}
	var last []byte
	for {
		if ctx.Err() != nil {
			return values, ctx.Err()
		}
		items, next, e := fetch(after)
		if e != nil {
			return values, e
		}
		for _, v := range items {
			id := v.GetId()
			if !validID(id, domain) {
				return nil, catalogError(connect.CodeDataLoss, "invalid resource identity")
			}
			if last != nil {
				cmp := bytes.Compare(id, last)
				if !p.cursor.Reverse && cmp <= 0 || p.cursor.Reverse && cmp >= 0 {
					return nil, catalogError(connect.CodeDataLoss, "invalid runner page order")
				}
			}
			last = id
			if p.beyond(id, session) && match(v) {
				values = append(values, v)
				if len(values) > p.size {
					return values, nil
				}
			}
		}
		if next == "" {
			return values, nil
		}
		if seen[next] || len(items) == 0 {
			return nil, catalogError(connect.CodeDataLoss, "invalid runner continuation")
		}
		seen[next] = true
		after = next
	}
}
