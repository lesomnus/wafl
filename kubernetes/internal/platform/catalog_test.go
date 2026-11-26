package platform

import (
	"bytes"
	"context"
	"fmt"
	"io"
	"net/http"
	"net/http/httptest"
	"slices"
	"strconv"
	"strings"
	"sync/atomic"
	"testing"
	"time"

	"connectrpc.com/connect"
	wf "github.com/lesomnus/wafl/kubernetes/gen/wafl"
	"github.com/lesomnus/wafl/kubernetes/gen/wafl/waflconnect"
	core "k8s.io/api/core/v1"
	meta "k8s.io/apimachinery/pkg/apis/meta/v1"
	"k8s.io/client-go/kubernetes/fake"
)

func catalogID(domain, n byte) []byte {
	return []byte{0, 0, 0, 0, 0, 0, 0x80, 0, 0x80, domain, 0, 0, 0, 0, 0, n}
}

type capturedCommand struct {
	session, revision, token string
	id                       []byte
	withdrawn                bool
}
type catalogGuest struct {
	runner          *wf.Runner
	instances       []*wf.Instance
	reservations    []*wf.Reservation
	commands        chan capturedCommand
	emit, cancelled chan struct{}
	failPage        bool
}

func guest(n byte, ids ...byte) *catalogGuest {
	g := &catalogGuest{runner: &wf.Runner{Id: catalogID(8, n), Alias: "self", Capabilities: []string{"reservation-recovery-v1"}}, commands: make(chan capturedCommand, 10), emit: make(chan struct{}), cancelled: make(chan struct{})}
	for _, id := range ids {
		g.instances = append(g.instances, &wf.Instance{Id: catalogID(7, id), Alias: "shared", Session: g.runner.Id, Revision: 9, Secrets: map[string]string{"TOKEN": "private-canary"}})
	}
	return g
}

type catalogRunnerStub struct {
	waflconnect.UnimplementedRunnerServiceHandler
	g *catalogGuest
}

func (s *catalogRunnerStub) List(context.Context, *connect.Request[wf.RunnerListRequest]) (*connect.Response[wf.RunnerListResponse], error) {
	return connect.NewResponse(&wf.RunnerListResponse{Items: []*wf.Runner{clone(s.g.runner)}}), nil
}

type catalogInstanceStub struct {
	waflconnect.UnimplementedInstanceServiceHandler
	g *catalogGuest
}

func (s *catalogInstanceStub) Get(_ context.Context, r *connect.Request[wf.InstanceGetRequest]) (*connect.Response[wf.Instance], error) {
	for _, v := range s.g.instances {
		if instanceMatches(r.Msg.Ref, v) {
			return connect.NewResponse(clone(v)), nil
		}
	}
	return nil, catalogError(connect.CodeNotFound, "missing")
}
func (s *catalogInstanceStub) List(_ context.Context, r *connect.Request[wf.InstanceListRequest]) (*connect.Response[wf.InstanceListResponse], error) {
	if r.Header().Get("wafl-session") != UUID(s.g.runner.Id) {
		return nil, catalogError(connect.CodeAborted, "missing receiving session")
	}
	if s.g.failPage && r.Msg.After != "" {
		return nil, catalogError(connect.CodeUnavailable, "private-canary")
	}
	values := slices.Clone(s.g.instances)
	if r.Msg.Reverse {
		slices.Reverse(values)
	}
	offset, _ := strconv.Atoi(r.Msg.After)
	if offset >= len(values) {
		return connect.NewResponse(&wf.InstanceListResponse{}), nil
	}
	next := ""
	if offset+1 < len(values) {
		next = strconv.Itoa(offset + 1)
	}
	return connect.NewResponse(&wf.InstanceListResponse{Items: []*wf.Instance{clone(values[offset])}, Next: next}), nil
}
func (s *catalogInstanceStub) Start(_ context.Context, r *connect.Request[wf.InstanceStartRequest]) (*connect.Response[wf.Instance], error) {
	s.g.commands <- capturedCommand{session: r.Header().Get("wafl-session"), revision: r.Header().Get("wafl-revision"), id: r.Msg.Ref.GetId()}
	return connect.NewResponse(clone(s.g.instances[0])), nil
}
func (s *catalogInstanceStub) Tail(ctx context.Context, r *connect.Request[wf.InstanceTailRequest], out *connect.ServerStream[wf.Log]) error {
	if r.Header().Get("wafl-session") != UUID(s.g.runner.Id) {
		return catalogError(connect.CodeAborted, "missing receiving session")
	}
	if e := out.Send(&wf.Log{Sequence: 1, Data: []byte("first\n")}); e != nil {
		return e
	}
	if !r.Msg.Follow {
		return nil
	}
	select {
	case <-s.g.emit:
	case <-ctx.Done():
		close(s.g.cancelled)
		return ctx.Err()
	}
	if e := out.Send(&wf.Log{Sequence: 2, Data: []byte("second\n")}); e != nil {
		return e
	}
	<-ctx.Done()
	close(s.g.cancelled)
	return ctx.Err()
}

type catalogReservationStub struct {
	waflconnect.UnimplementedReservationServiceHandler
	g *catalogGuest
}

func (s *catalogReservationStub) List(_ context.Context, r *connect.Request[wf.ReservationListRequest]) (*connect.Response[wf.ReservationListResponse], error) {
	if r.Header().Get("wafl-session") != UUID(s.g.runner.Id) {
		return nil, catalogError(connect.CodeAborted, "missing receiving session")
	}
	out := []*wf.Reservation{}
	for _, v := range s.g.reservations {
		out = append(out, clone(v))
	}
	if r.Msg.Reverse {
		slices.Reverse(out)
	}
	return connect.NewResponse(&wf.ReservationListResponse{Items: out}), nil
}
func (s *catalogReservationStub) Release(_ context.Context, r *connect.Request[wf.ReservationReleaseRequest]) (*connect.Response[wf.ReservationReleaseResponse], error) {
	s.g.commands <- capturedCommand{session: r.Header().Get("wafl-session"), token: r.Msg.Token, withdrawn: r.Msg.Withdrawn}
	return nil, catalogError(connect.CodeAborted, "allocation changed")
}

type catalogHarness struct {
	catalog      *Catalog
	runners      waflconnect.RunnerServiceClient
	instances    waflconnect.InstanceServiceClient
	reservations waflconnect.ReservationServiceClient
}

func addCatalogNode(t *testing.T, c *Catalog, name, endpoint, namespace string) {
	t.Helper()
	_, e := c.Clients.Kube.CoreV1().Nodes().Create(context.Background(), &core.Node{ObjectMeta: meta.ObjectMeta{Name: name, Labels: map[string]string{Runtime: "wasmtime"}, Annotations: map[string]string{Group + "/namespace": namespace, Runner: endpoint}}}, meta.CreateOptions{})
	if e != nil {
		t.Fatal(e)
	}
}
func newCatalogHarness(t *testing.T, guests ...*catalogGuest) *catalogHarness {
	t.Helper()
	c := &Catalog{Clients: &Clients{Kube: fake.NewClientset()}, Namespace: "system", Timeout: 200 * time.Millisecond, Parallel: 2}
	for i, g := range guests {
		mux := http.NewServeMux()
		mux.Handle(waflconnect.NewRunnerServiceHandler(&catalogRunnerStub{g: g}))
		mux.Handle(waflconnect.NewInstanceServiceHandler(&catalogInstanceStub{g: g}))
		mux.Handle(waflconnect.NewReservationServiceHandler(&catalogReservationStub{g: g}))
		server := httptest.NewServer(mux)
		t.Cleanup(server.Close)
		addCatalogNode(t, c, fmt.Sprintf("runner-%d", i), server.URL, "system")
	}
	mux := http.NewServeMux()
	mux.Handle(waflconnect.NewRunnerServiceHandler(&RunnerAPI{Catalog: c}))
	mux.Handle(waflconnect.NewInstanceServiceHandler(&InstanceAPI{Catalog: c}))
	mux.Handle(waflconnect.NewReservationServiceHandler(&ReservationAPI{Catalog: c}))
	server := httptest.NewServer(mux)
	t.Cleanup(server.Close)
	return &catalogHarness{c, waflconnect.NewRunnerServiceClient(server.Client(), server.URL, connect.WithProtoJSON()), waflconnect.NewInstanceServiceClient(server.Client(), server.URL, connect.WithProtoJSON()), waflconnect.NewReservationServiceClient(server.Client(), server.URL, connect.WithProtoJSON())}
}
func offlineCatalogNode(t *testing.T, h *catalogHarness) {
	t.Helper()
	stop := make(chan struct{})
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		_, _ = io.Copy(io.Discard, r.Body)
		select {
		case <-r.Context().Done():
		case <-stop:
		}
	}))
	t.Cleanup(func() { close(stop); server.Close() })
	addCatalogNode(t, h.catalog, "offline", server.URL, "system")
	// An unavailable node from another deployment must not enter the result.
	addCatalogNode(t, h.catalog, "foreign", server.URL, "other")
}

func TestCatalogRetainedNodeCannotClaimReusedPodAddress(t *testing.T) {
	g := guest(1)
	h := newCatalogHarness(t, g)
	ctx := context.Background()
	node, e := h.catalog.Clients.Kube.CoreV1().Nodes().Get(ctx, "runner-0", meta.GetOptions{})
	if e != nil {
		t.Fatal(e)
	}
	_, e = h.catalog.Clients.Kube.CoreV1().Pods("system").Create(ctx, &core.Pod{ObjectMeta: meta.ObjectMeta{Name: "physical", Namespace: "system", UID: "current"}}, meta.CreateOptions{})
	if e != nil {
		t.Fatal(e)
	}
	node.Annotations[RunnerPod] = "system/physical"
	node.Annotations[Group+"/runnerUID"] = "current"
	if _, e = h.catalog.Clients.Kube.CoreV1().Nodes().Update(ctx, node, meta.UpdateOptions{}); e != nil {
		t.Fatal(e)
	}
	old := node.DeepCopy()
	old.Name = "retained-old" // sorts before the current node with the same URL
	old.Annotations[Group+"/runnerUID"] = "previous"
	if _, e = h.catalog.Clients.Kube.CoreV1().Nodes().Create(ctx, old, meta.CreateOptions{}); e != nil {
		t.Fatal(e)
	}
	result, e := h.runners.List(ctx, connect.NewRequest(&wf.RunnerListRequest{}))
	if e != nil {
		t.Fatal(e)
	}
	if len(result.Msg.Items) != 1 || result.Msg.Items[0].Alias != "runner-0" {
		t.Fatalf("retained node claimed new process: %v", result.Msg.Items)
	}
	if warnings := result.Header().Values("wafl-warning"); len(warnings) != 1 || !strings.Contains(warnings[0], "retained-old") {
		t.Fatalf("missing retained-node warning: %v", warnings)
	}
	if _, e = h.runners.Get(ctx, connect.NewRequest(&wf.RunnerGetRequest{Ref: &wf.RunnerRef{Key: &wf.RunnerRef_Alias{Alias: "retained-old"}}})); connect.CodeOf(e) != connect.CodeUnavailable {
		t.Fatalf("retained-node lookup reached replacement: %v", e)
	}
}
func TestCatalogPartialPagesUseGlobalOrderAndScope(t *testing.T) {
	h := newCatalogHarness(t, guest(1, 1, 3, 5), guest(2, 2, 4))
	offlineCatalogNode(t, h)
	ctx := context.Background()
	q := &wf.InstanceListRequest{Size: 2}
	got := []byte{}
	for {
		page, e := h.instances.List(ctx, connect.NewRequest(q))
		if e != nil {
			t.Fatal(e)
		}
		if warnings := page.Header().Values("wafl-warning"); len(warnings) != 1 || !strings.Contains(warnings[0], "offline") {
			t.Fatalf("missing or out-of-scope warnings: %v", warnings)
		}
		for _, v := range page.Msg.Items {
			got = append(got, v.Id[15])
			if len(v.Secrets) != 0 {
				t.Fatal("Secret values escaped aggregation")
			}
		}
		if page.Msg.Next == "" {
			break
		}
		q.After = page.Msg.Next
	}
	if !bytes.Equal(got, []byte{1, 2, 3, 4, 5}) {
		t.Fatalf("bad merged order: %v", got)
	}
	reverse, e := h.instances.List(ctx, connect.NewRequest(&wf.InstanceListRequest{Size: 2, Reverse: true}))
	if e != nil {
		t.Fatal(e)
	}
	if reverse.Msg.Items[0].Id[15] != 5 || reverse.Msg.Items[1].Id[15] != 4 {
		t.Fatal("reverse order failed")
	}
	scoped := request(&wf.InstanceListRequest{After: reverse.Msg.Next, Reverse: true}, catalogID(8, 1), 0)
	if _, e = h.instances.List(ctx, scoped); connect.CodeOf(e) != connect.CodeInvalidArgument {
		t.Fatal("cursor crossed session scope")
	}
	if _, e = h.instances.List(ctx, connect.NewRequest(&wf.InstanceListRequest{After: reverse.Msg.Next})); connect.CodeOf(e) != connect.CodeInvalidArgument {
		t.Fatal("cursor crossed direction")
	}
	runners, e := h.runners.List(ctx, connect.NewRequest(&wf.RunnerListRequest{Size: 1}))
	if e != nil || len(runners.Msg.Items) != 1 || runners.Msg.Next == "" {
		t.Fatal("runner List did not paginate")
	}
	second, e := h.runners.List(ctx, connect.NewRequest(&wf.RunnerListRequest{Size: 1, After: runners.Msg.Next}))
	if e != nil || len(second.Msg.Items) != 1 || second.Msg.Items[0].Alias != "runner-1" {
		t.Fatal("runner aliases/order failed")
	}
}
func TestCatalogAllUnavailableIsEmptyButGetIsUnavailable(t *testing.T) {
	h := newCatalogHarness(t)
	offlineCatalogNode(t, h)
	ctx := context.Background()
	runners, e := h.runners.List(ctx, connect.NewRequest(&wf.RunnerListRequest{}))
	if e != nil || len(runners.Msg.Items) != 0 || runners.Header().Get("wafl-warning") == "" {
		t.Fatal("all-failed runner List did not succeed with warnings")
	}
	instances, e := h.instances.List(ctx, connect.NewRequest(&wf.InstanceListRequest{}))
	if e != nil || len(instances.Msg.Items) != 0 || instances.Header().Get("wafl-warning") == "" {
		t.Fatal("all-failed instance List did not succeed with warnings")
	}
	reservations, e := h.reservations.List(ctx, connect.NewRequest(&wf.ReservationListRequest{}))
	if e != nil || len(reservations.Msg.Items) != 0 || reservations.Header().Get("wafl-warning") == "" {
		t.Fatal("all-failed reservation List did not succeed with warnings")
	}
	_, e = h.instances.Get(ctx, connect.NewRequest(&wf.InstanceGetRequest{Ref: instanceRef(catalogID(7, 1))}))
	if connect.CodeOf(e) != connect.CodeUnavailable {
		t.Fatalf("connection failure looked like absence: %v", e)
	}
	canceled, cancel := context.WithCancel(ctx)
	cancel()
	_, e = (&RunnerAPI{Catalog: h.catalog}).List(canceled, connect.NewRequest(&wf.RunnerListRequest{}))
	if connect.CodeOf(e) != connect.CodeCanceled {
		t.Fatal("caller cancellation was swallowed")
	}
}
func TestCatalogKeepsSuccessfulPagesAndSanitizesWarnings(t *testing.T) {
	g := guest(1, 1, 2)
	g.failPage = true
	h := newCatalogHarness(t, g)
	page, e := h.instances.List(context.Background(), connect.NewRequest(&wf.InstanceListRequest{}))
	if e != nil || len(page.Msg.Items) != 1 || page.Msg.Items[0].Id[15] != 1 {
		t.Fatalf("lost successful page: %v", e)
	}
	if warnings := page.Header().Get("wafl-warning"); warnings == "" || strings.Contains(warnings, "private-canary") {
		t.Fatal("missing or unsafe warning")
	}
}
func TestCatalogTargetsAndCommandsPreserveConditions(t *testing.T) {
	g := guest(1, 1)
	h := newCatalogHarness(t, g, guest(2, 2))
	offlineCatalogNode(t, h)
	ctx := context.Background()
	alias := &wf.InstanceRef{Key: &wf.InstanceRef_Alias{Alias: "shared"}}
	if _, e := h.instances.Get(ctx, connect.NewRequest(&wf.InstanceGetRequest{Ref: alias})); connect.CodeOf(e) != connect.CodeFailedPrecondition {
		t.Fatal("ambiguous alias selected a runner")
	}
	value, e := h.instances.Get(ctx, request(&wf.InstanceGetRequest{Ref: alias}, g.runner.Id, 0))
	if e != nil || !bytes.Equal(value.Msg.Id, g.instances[0].Id) || len(value.Msg.Secrets) != 0 {
		t.Fatal("scoped lookup failed")
	}
	if _, e = h.instances.Get(ctx, request(&wf.InstanceGetRequest{Ref: instanceRef(catalogID(7, 99))}, g.runner.Id, 0)); connect.CodeOf(e) != connect.CodeNotFound {
		t.Fatal("unrelated failure blocked a known missing target")
	}
	if _, e = h.instances.Start(ctx, connect.NewRequest(&wf.InstanceStartRequest{Ref: alias})); connect.CodeOf(e) != connect.CodeInvalidArgument {
		t.Fatal("unscoped alias mutation was accepted")
	}
	started, e := h.instances.Start(ctx, request(&wf.InstanceStartRequest{Ref: alias}, g.runner.Id, 9))
	if e != nil || len(started.Msg.Secrets) != 0 {
		t.Fatal(e)
	}
	captured := <-g.commands
	if captured.session != UUID(g.runner.Id) || captured.revision != "9" || !bytes.Equal(captured.id, g.instances[0].Id) {
		t.Fatal("mutation changed identity or revision")
	}
	_, e = h.instances.Start(ctx, connect.NewRequest(&wf.InstanceStartRequest{Ref: instanceRef(g.instances[0].Id)}))
	if e != nil {
		t.Fatal(e)
	}
	if captured = <-g.commands; captured.session != UUID(g.runner.Id) {
		t.Fatal("resolved mutation was not fenced")
	}
	release := request(&wf.ReservationReleaseRequest{Ref: &wf.ReservationRef{Key: &wf.ReservationRef_Id{Id: catalogID(9, 1)}}, Token: "old-token", Withdrawn: true}, g.runner.Id, 0)
	if _, e = h.reservations.Release(ctx, release); connect.CodeOf(e) != connect.CodeAborted {
		t.Fatal("release conflict was swallowed")
	}
	captured = <-g.commands
	if captured.session != UUID(g.runner.Id) || captured.token != "old-token" || !captured.withdrawn {
		t.Fatal("release token, receiver, or confirmation changed")
	}
	if len(g.commands) != 0 {
		t.Fatal("mutation was retried")
	}
	_, e = h.instances.Add(ctx, connect.NewRequest(&wf.InstanceAddRequest{}))
	if connect.CodeOf(e) != connect.CodeInvalidArgument {
		t.Fatal("Add silently chose a runner")
	}
}
func TestCatalogTailOutlivesDiscoveryAndPropagatesCancellation(t *testing.T) {
	g := guest(1, 1)
	h := newCatalogHarness(t, g)
	ctx, cancel := context.WithCancel(context.Background())
	defer cancel()
	stream, e := h.instances.Tail(ctx, connect.NewRequest(&wf.InstanceTailRequest{Ref: instanceRef(g.instances[0].Id), Follow: true}))
	if e != nil {
		t.Fatal(e)
	}
	defer stream.Close()
	if !stream.Receive() || stream.Msg().Sequence != 1 {
		t.Fatal("missing first log")
	}
	time.Sleep(2 * h.catalog.Timeout)
	close(g.emit)
	if !stream.Receive() || stream.Msg().Sequence != 2 {
		t.Fatalf("discovery deadline killed follow stream: %v", stream.Err())
	}
	cancel()
	select {
	case <-g.cancelled:
	case <-time.After(3 * time.Second):
		t.Fatal("stream cancellation did not reach runner")
	}
}
func TestCatalogSharedReservationPoolIsDeduplicated(t *testing.T) {
	a, b := guest(1), guest(2)
	v := &wf.Reservation{Id: catalogID(9, 1), Address: "127.0.0.1:20000", Token: "allocation", Session: catalogID(8, 99), Status: wf.ReservationStatus_RESERVATION_STATUS_ORPHAN}
	a.reservations = []*wf.Reservation{v}
	b.reservations = []*wf.Reservation{clone(v)}
	h := newCatalogHarness(t, a, b)
	ctx := context.Background()
	page, e := h.reservations.List(ctx, connect.NewRequest(&wf.ReservationListRequest{Size: 1}))
	if e != nil || len(page.Msg.Items) != 1 || page.Msg.Next != "" {
		t.Fatal("shared pool was duplicated")
	}
	value, e := h.reservations.Get(ctx, connect.NewRequest(&wf.ReservationGetRequest{Ref: &wf.ReservationRef{Key: &wf.ReservationRef_Address{Address: v.Address}}}))
	if e != nil || !bytes.Equal(value.Msg.Session, v.Session) {
		t.Fatal("historical owner was replaced with receiving runner")
	}
}

func TestCatalogConflictingReservationRequiresExplicitReceiver(t *testing.T) {
	a, b := guest(1), guest(2)
	a.reservations = []*wf.Reservation{{Id: catalogID(9, 1), Address: "127.0.0.1:20000", Token: "old"}}
	b.reservations = []*wf.Reservation{clone(a.reservations[0])}
	b.reservations[0].Token = "new"
	h := newCatalogHarness(t, a, b)
	ctx := context.Background()
	page, e := h.reservations.List(ctx, connect.NewRequest(&wf.ReservationListRequest{}))
	if e != nil || len(page.Msg.Items) != 1 || page.Header().Get("wafl-warning") == "" {
		t.Fatal("conflicting shared-pool observations were not reported")
	}
	q := &wf.ReservationGetRequest{Ref: &wf.ReservationRef{Key: &wf.ReservationRef_Id{Id: a.reservations[0].Id}}}
	if _, e = h.reservations.Get(ctx, connect.NewRequest(q)); connect.CodeOf(e) != connect.CodeAborted {
		t.Fatalf("conflicting token was silently selected: %v", e)
	}
	v, e := h.reservations.Get(ctx, request(q, b.runner.Id, 0))
	if e != nil || v.Msg.Token != "new" {
		t.Fatal("explicit receiving session was ignored")
	}
}

func TestCatalogSameInstanceIDInDifferentSessionsSurvivesPagination(t *testing.T) {
	h := newCatalogHarness(t, guest(1, 1), guest(2, 1))
	ctx := context.Background()
	first, e := h.instances.List(ctx, connect.NewRequest(&wf.InstanceListRequest{Size: 1}))
	if e != nil || len(first.Msg.Items) != 1 || first.Msg.Next == "" {
		t.Fatal("missing first session")
	}
	second, e := h.instances.List(ctx, connect.NewRequest(&wf.InstanceListRequest{Size: 1, After: first.Msg.Next}))
	if e != nil || len(second.Msg.Items) != 1 || second.Msg.Next != "" || bytes.Equal(first.Msg.Items[0].Session, second.Msg.Items[0].Session) {
		t.Fatal("colliding IDs lost their receiving session")
	}
}

type catalogTransport func(*http.Request) (*http.Response, error)

func (f catalogTransport) RoundTrip(r *http.Request) (*http.Response, error) { return f(r) }

func TestCatalogFanoutIsBoundedAndCancellationReachesRunners(t *testing.T) {
	c := &Catalog{Clients: &Clients{Kube: fake.NewClientset()}, Namespace: "system", Parallel: 3, Timeout: time.Minute}
	for i := range 10 {
		addCatalogNode(t, c, fmt.Sprintf("runner-%d", i), fmt.Sprintf("http://runner-%d", i), "system")
	}
	started := make(chan struct{}, 10)
	var active atomic.Int32
	c.HTTP = &http.Client{Transport: catalogTransport(func(r *http.Request) (*http.Response, error) {
		active.Add(1)
		defer active.Add(-1)
		started <- struct{}{}
		<-r.Context().Done()
		return nil, r.Context().Err()
	})}
	ctx, cancel := context.WithTimeout(context.Background(), 3*time.Second)
	defer cancel()
	done := make(chan error, 1)
	go func() {
		_, e := (&RunnerAPI{Catalog: c}).List(ctx, connect.NewRequest(&wf.RunnerListRequest{}))
		done <- e
	}()
	for range 3 {
		select {
		case <-started:
		case <-ctx.Done():
			t.Fatal("workers did not start")
		}
	}
	select {
	case <-started:
		t.Fatal("fanout exceeded configured parallelism")
	case <-time.After(40 * time.Millisecond):
	}
	cancel()
	select {
	case e := <-done:
		if connect.CodeOf(e) != connect.CodeCanceled || active.Load() != 0 {
			t.Fatalf("cancellation did not close upstream queries: %v (%d active)", e, active.Load())
		}
	case <-time.After(time.Second):
		t.Fatal("caller cancellation waited for runner timeout")
	}
}
