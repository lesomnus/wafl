package traefik

import (
	"context"
	"encoding/base64"
	"encoding/json"
	"net/http"
	"net/http/httptest"
	"sync"
	"sync/atomic"
	"testing"
	"time"
)

func eventually(t *testing.T, f func() bool) {
	t.Helper()
	deadline := time.Now().Add(5 * time.Second)
	for !f() {
		if time.Now().After(deadline) {
			t.Fatal("condition did not become true")
		}
		time.Sleep(10 * time.Millisecond)
	}
}
func TestAdmissionSurvivesReloadDrainsStreamsAndNeverReplays(t *testing.T) {
	var draining atomic.Bool
	var registrations, fallback, upstream atomic.Int32
	var mu sync.Mutex
	var latest observation
	cp := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		if r.URL.Path == "/wafl.TrafficService/Add" {
			registrations.Add(1)
			if draining.Load() {
				w.WriteHeader(400)
				_, _ = w.Write([]byte(`{"code":"failed_precondition"}`))
				return
			}
			_, _ = w.Write([]byte(`{}`))
			return
		}
		var report observation
		if e := json.NewDecoder(r.Body).Decode(&report); e != nil {
			t.Error(e)
			w.WriteHeader(400)
			return
		}
		mu.Lock()
		latest = report
		mu.Unlock()
		report.Draining = draining.Load()
		_ = json.NewEncoder(w).Encode(report)
	}))
	defer cp.Close()
	activation := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		fallback.Add(1)
		if r.Host != "app.example" {
			t.Error("Host changed")
		}
		w.WriteHeader(202)
	}))
	defer activation.Close()
	id := make([]byte, 16)
	id[8] = 0x80
	id[9] = 11
	config := &Config{Control: cp.URL, Activate: activation.URL, Placement: base64.StdEncoding.EncodeToString(id), IntervalMs: 100}
	entered := make(chan struct{})
	finish := make(chan struct{})
	next := http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		upstream.Add(1)
		close(entered)
		<-finish
		w.WriteHeader(502)
	})
	first, e := New(context.Background(), next, config, "first")
	if e != nil {
		t.Fatal(e)
	}
	second, e := New(context.Background(), next, config, "reload")
	if e != nil {
		t.Fatal(e)
	}
	if first.(*middleware).gate != second.(*middleware).gate {
		t.Fatal("reload reopened admission")
	}
	done := make(chan struct{})
	go func() {
		first.ServeHTTP(httptest.NewRecorder(), httptest.NewRequest("POST", "http://app.example/stream", nil))
		close(done)
	}()
	<-entered
	eventually(t, func() bool {
		mu.Lock()
		defer mu.Unlock()
		return latest.Active == 1 && latest.Started == 1 && latest.Busy > 0
	})
	draining.Store(true)
	eventually(t, func() bool { g := first.(*middleware).gate; g.mu.Lock(); defer g.mu.Unlock(); return g.closed })
	response := httptest.NewRecorder()
	second.ServeHTTP(response, httptest.NewRequest("POST", "http://app.example/new", nil))
	if response.Code != 202 || upstream.Load() != 1 || fallback.Load() != 1 {
		t.Fatal("closed gate sent a request to the retiring endpoint")
	}
	eventually(t, func() bool { mu.Lock(); defer mu.Unlock(); return latest.Closed && latest.Active == 1 })
	close(finish)
	<-done
	eventually(t, func() bool { mu.Lock(); defer mu.Unlock(); return latest.Closed && latest.Active == 0 })
	if fallback.Load() != 1 {
		t.Fatal("upstream failure was replayed")
	}
	eventually(t, func() bool {
		registry.Lock()
		defer registry.Unlock()
		return registry.gates[first.(*middleware).gate.key] == nil
	})
	// A late configuration reload also needs registration. A retired placement
	// refuses it, including when no observer previously used that configuration.
	third, e := New(context.Background(), next, config, "late")
	if e != nil {
		t.Fatal(e)
	}
	third.ServeHTTP(httptest.NewRecorder(), httptest.NewRequest("GET", "http://app.example/", nil))
	if upstream.Load() != 1 || fallback.Load() != 2 || registrations.Load() != 2 {
		t.Fatal("late registration bypassed retirement")
	}
}

func TestRegistrationFailurePrecedesForwarding(t *testing.T) {
	cp := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) { w.WriteHeader(503) }))
	defer cp.Close()
	id := make([]byte, 16)
	id[8] = 0x80
	id[9] = 11
	id[15] = 1
	h, e := New(context.Background(), http.HandlerFunc(func(http.ResponseWriter, *http.Request) { t.Error("unregistered request was forwarded") }), &Config{Control: cp.URL, Activate: cp.URL, Placement: base64.StdEncoding.EncodeToString(id), IntervalMs: 100}, "failure")
	if e != nil {
		t.Fatal(e)
	}
	response := httptest.NewRecorder()
	h.ServeHTTP(response, httptest.NewRequest("POST", "http://app.example/", nil))
	if response.Code != 503 {
		t.Fatal(response.Code)
	}
	h.(*middleware).gate.forget()
}
