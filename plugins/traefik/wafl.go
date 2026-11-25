// Package traefik observes the lifetime of direct requests, including streaming
// bodies. The controller never treats a missing heartbeat as a completed drain.
package traefik

import (
	"bytes"
	"context"
	"crypto/rand"
	"encoding/base64"
	"encoding/hex"
	"encoding/json"
	"errors"
	"fmt"
	"io"
	"net/http"
	"net/http/httputil"
	"net/url"
	"reflect"
	"strings"
	"sync"
	"time"
)

type Config struct {
	Control    string `json:"control"`
	Activate   string `json:"activate"`
	Placement  string `json:"placement"`
	IntervalMs int    `json:"intervalMs"`
}

func CreateConfig() *Config { return &Config{IntervalMs: 500} }

var registry = struct {
	sync.Mutex
	gates map[string]*gate
}{gates: make(map[string]*gate)}

type gate struct {
	mu                      sync.Mutex
	register                sync.Mutex
	key                     string
	config                  Config
	id, session             string
	client                  *http.Client
	registered, closed      bool
	active                  uint32
	started, sequence, busy uint64
	fraction                uint64
	date                    time.Time
}
type middleware struct {
	next     http.Handler
	fallback http.Handler
	gate     *gate
}

func origin(value string) (*url.URL, error) {
	u, e := url.Parse(value)
	if e != nil || u.Scheme != "http" || u.Host == "" || u.User != nil || (u.Path != "" && u.Path != "/") || u.RawQuery != "" || u.Fragment != "" {
		return nil, errors.New("WAFL endpoints must be HTTP origins")
	}
	return u, nil
}
func New(_ context.Context, next http.Handler, config *Config, _ string) (http.Handler, error) {
	if _, e := origin(config.Control); e != nil {
		return nil, e
	}
	target, e := origin(config.Activate)
	if e != nil {
		return nil, e
	}
	placement, e := base64.StdEncoding.DecodeString(config.Placement)
	if e != nil || len(placement) != 16 || placement[8]&0xc0 != 0x80 || placement[9] != 11 {
		return nil, errors.New("invalid placement ID")
	}
	if config.IntervalMs < 100 || config.IntervalMs > 5000 {
		return nil, errors.New("invalid report interval")
	}
	key := strings.TrimRight(config.Control, "/") + "/" + config.Placement
	registry.Lock()
	g := registry.gates[key]
	if g == nil {
		id := make([]byte, 16)
		session := make([]byte, 16)
		if _, e = rand.Read(id); e == nil {
			_, e = rand.Read(session)
		}
		if e != nil {
			registry.Unlock()
			return nil, e
		}
		id[6] = (id[6] & 0x0f) | 0x80
		id[8] = 0x80
		id[9] = 13
		g = &gate{key: key, config: *config, id: base64.StdEncoding.EncodeToString(id), session: hex.EncodeToString(session), client: &http.Client{Timeout: 2 * time.Second}, date: time.Now()}
		registry.gates[key] = g
	}
	registry.Unlock()
	proxy := httputil.NewSingleHostReverseProxy(target)
	// Activators accept h2c. Keep fallback streaming and arbitrary gRPC trailers
	// on HTTP/2 as well; an HTTP/1 hop cannot announce unknown trailers in advance.
	transport, e := activationTransport()
	if e != nil {
		return nil, e
	}
	proxy.Transport = transport
	director := proxy.Director
	proxy.Director = func(r *http.Request) { host := r.Host; director(r); r.Host = host }
	proxy.FlushInterval = -1
	return &middleware{next: next, fallback: proxy, gate: g}, nil
}

func activationTransport() (*http.Transport, error) {
	transport := http.DefaultTransport.(*http.Transport).Clone()
	// Go 1.24 added Transport.Protocols. Traefik's Yaegi symbol catalogue still
	// omits the named Protocols type, even when the host runtime supports it.
	// Reflect only this public API; no unsafe access or internal HTTP/2 types.
	field := reflect.ValueOf(transport).Elem().FieldByName("Protocols")
	if !field.IsValid() || !field.CanSet() || field.Kind() != reflect.Ptr {
		return nil, errors.New("WAFL activation requires Go HTTP/2 protocol selection")
	}
	protocols := reflect.New(field.Type().Elem())
	enable := protocols.MethodByName("SetUnencryptedHTTP2")
	if !enable.IsValid() {
		return nil, errors.New("Go runtime lacks unencrypted HTTP/2")
	}
	enable.Call([]reflect.Value{reflect.ValueOf(true)})
	field.Set(protocols)
	return transport, nil
}

type observation struct {
	Sequence uint64 `json:"sequence,string"`
	Active   uint32 `json:"active"`
	Started  uint64 `json:"started,string"`
	Busy     uint64 `json:"busyMs,string"`
	Closed   bool   `json:"closed"`
	Draining bool   `json:"draining"`
}
type rpcError struct {
	code   string
	status int
}

func (e *rpcError) Error() string {
	return fmt.Sprintf("controller returned %d (%s)", e.status, e.code)
}
func (g *gate) rpc(method string, request interface{}) (observation, error) {
	var result observation
	body, e := json.Marshal(request)
	if e != nil {
		return result, e
	}
	req, e := http.NewRequest(http.MethodPost, strings.TrimRight(g.config.Control, "/")+"/wafl.TrafficService/"+method, bytes.NewReader(body))
	if e != nil {
		return result, e
	}
	req.Header.Set("Content-Type", "application/json")
	req.Header.Set("Connect-Protocol-Version", "1")
	response, e := g.client.Do(req)
	if e != nil {
		return result, e
	}
	defer response.Body.Close()
	data, e := io.ReadAll(io.LimitReader(response.Body, 1024*1024))
	if e != nil {
		return result, e
	}
	if response.StatusCode != http.StatusOK {
		var errorBody struct {
			Code string `json:"code"`
		}
		_ = json.Unmarshal(data, &errorBody)
		return result, &rpcError{code: errorBody.Code, status: response.StatusCode}
	}
	e = json.Unmarshal(data, &result)
	return result, e
}
func (g *gate) ready() error {
	g.mu.Lock()
	ready := g.registered || g.closed
	g.mu.Unlock()
	if ready {
		return nil
	}
	g.register.Lock()
	defer g.register.Unlock()
	g.mu.Lock()
	done := g.registered || g.closed
	g.mu.Unlock()
	if done {
		return nil
	}
	result, e := g.rpc("Add", map[string]interface{}{"id": g.id, "session": g.session, "placement": map[string]string{"id": g.config.Placement}})
	if e != nil {
		if remote, ok := e.(*rpcError); ok && remote.code == "failed_precondition" {
			g.mu.Lock()
			g.closed = true
			g.mu.Unlock()
			g.forget()
			return nil
		}
		return e
	}
	g.mu.Lock()
	g.registered = true
	g.closed = result.Draining || result.Closed
	g.mu.Unlock()
	go g.reports()
	return nil
}
func (g *gate) integrate() {
	now := time.Now()
	elapsed := now.Sub(g.date).Nanoseconds()
	g.date = now
	if elapsed > 0 {
		// Store milliseconds, carrying sub-millisecond work across samples.
		// Accumulating active*nanoseconds wraps after only months under load.
		fraction := uint64(elapsed%1000000)*uint64(g.active) + g.fraction
		addition := uint64(elapsed/1000000)*uint64(g.active) + fraction/1000000
		g.fraction = fraction % 1000000
		if addition > ^uint64(0)-g.busy {
			g.busy = ^uint64(0)
		} else {
			g.busy += addition
		}
	}
}
func (g *gate) enter() bool {
	g.mu.Lock()
	defer g.mu.Unlock()
	if g.closed || g.active == ^uint32(0) {
		return false
	}
	g.integrate()
	g.active++
	g.started++
	return true
}
func (g *gate) leave() { g.mu.Lock(); g.integrate(); g.active--; g.mu.Unlock() }
func (g *gate) forget() {
	registry.Lock()
	if registry.gates[g.key] == g {
		delete(registry.gates, g.key)
	}
	registry.Unlock()
}
func (g *gate) reports() {
	for {
		time.Sleep(time.Duration(g.config.IntervalMs) * time.Millisecond)
		g.mu.Lock()
		g.integrate()
		g.sequence++
		snapshot := observation{Sequence: g.sequence, Active: g.active, Started: g.started, Busy: g.busy, Closed: g.closed}
		g.mu.Unlock()
		result, e := g.rpc("Report", struct {
			Ref      map[string]string `json:"ref"`
			Sequence uint64            `json:"sequence,string"`
			Active   uint32            `json:"active"`
			Started  uint64            `json:"started,string"`
			Busy     uint64            `json:"busyMs,string"`
			Closed   bool              `json:"closed"`
		}{map[string]string{"id": g.id}, snapshot.Sequence, snapshot.Active, snapshot.Started, snapshot.Busy, snapshot.Closed})
		if e != nil {
			continue
		}
		g.mu.Lock()
		if result.Draining || result.Closed {
			g.closed = true
		}
		finished := snapshot.Closed && snapshot.Active == 0 && result.Closed && result.Active == 0
		g.mu.Unlock()
		if finished {
			g.forget()
			return
		}
	}
}
func (m *middleware) ServeHTTP(w http.ResponseWriter, r *http.Request) {
	if e := m.gate.ready(); e != nil {
		http.Error(w, "WAFL observer registration unavailable", http.StatusServiceUnavailable)
		return
	}
	if !m.gate.enter() {
		// No request bytes were forwarded to a WASM endpoint. Only this case
		// may fall back to activation; errors after next.ServeHTTP never retry.
		m.fallback.ServeHTTP(w, r)
		return
	}
	defer m.gate.leave()
	m.next.ServeHTTP(w, r)
}
