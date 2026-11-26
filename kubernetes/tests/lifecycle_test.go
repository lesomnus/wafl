package tests

import (
	"bytes"
	"fmt"
	"io"
	"net/http"
	"os"
	"sync"
	"sync/atomic"
	"testing"
	"time"

	"github.com/lesomnus/wafl/kubernetes/internal/platform"
	apps "k8s.io/api/apps/v1"
	core "k8s.io/api/core/v1"
	apierrors "k8s.io/apimachinery/pkg/api/errors"
	meta "k8s.io/apimachinery/pkg/apis/meta/v1"
	"k8s.io/apimachinery/pkg/apis/meta/v1/unstructured"
	"k8s.io/client-go/util/retry"
)

var echoFrame = []byte{0, 0, 0, 0, 4, 'p', 'i', 'n', 'g'}

// Every request is counted, including failures. Do not hide transition errors
// behind eventually() retries or a successful request after the rollout.
func (s *suite) probe(name string, builds ...string) func() {
	s.t.Helper()
	transport := http.DefaultTransport.(*http.Transport).Clone()
	transport.Protocols = new(http.Protocols)
	transport.Protocols.SetUnencryptedHTTP2(true)
	client := &http.Client{Transport: transport, Timeout: 10 * time.Second}
	var stop atomic.Bool
	var count, failed atomic.Int64
	var mu sync.Mutex
	var first string
	var wg sync.WaitGroup
	for range 2 {
		wg.Add(1)
		go func() {
			defer wg.Done()
			for !stop.Load() {
				build, err := s.call(client, name)
				valid := false
				for _, want := range builds {
					valid = valid || build == want
				}
				if err != nil || !valid {
					failed.Add(1)
					mu.Lock()
					if first == "" {
						first = fmt.Sprintf("build=%q error=%v", build, err)
					}
					mu.Unlock()
				}
				count.Add(1)
				time.Sleep(50 * time.Millisecond)
			}
		}()
	}
	var once sync.Once
	finish := func() {
		once.Do(func() {
			stop.Store(true)
			wg.Wait()
			transport.CloseIdleConnections()
			s.t.Logf("gateway requests=%d errors=%d first_error=%s", count.Load(), failed.Load(), first)
			if count.Load() < 10 || failed.Load() != 0 {
				s.t.Errorf("transition traffic: requests=%d failures=%d: %s", count.Load(), failed.Load(), first)
			}
		})
	}
	s.t.Cleanup(finish)
	eventually(s.t, "traffic probe started", 10*time.Second, func() bool { return count.Load() >= 10 })
	return finish
}

func (s *suite) call(client *http.Client, name string) (string, error) {
	r, err := http.NewRequestWithContext(s.ctx, "POST", s.gateway+"/echo.Echo/Stream", bytes.NewReader(echoFrame))
	if err != nil {
		return "", err
	}
	r.Host = name + "." + s.ns + ".example"
	r.Header.Set("Content-Type", "application/grpc")
	r.Header.Set("TE", "trailers")
	r.Header.Set("Grpc-Timeout", "10S")
	v, err := client.Do(r)
	if err != nil {
		return "", err
	}
	defer v.Body.Close()
	b, err := io.ReadAll(v.Body)
	if err != nil || v.StatusCode != 200 || !bytes.Equal(b, echoFrame) || v.Trailer.Get("Grpc-Status") != "0" {
		return "", fmt.Errorf("status=%d body=%q trailers=%v: %v", v.StatusCode, b, v.Trailer, err)
	}
	return v.Header.Get("Echo-Build"), nil
}

func (s *suite) stream(name, build string) (*http.Response, *io.PipeWriter) {
	s.t.Helper()
	var response *http.Response
	var writer *io.PipeWriter
	eventually(s.t, "gateway serves binary "+build, 60*time.Second, func() bool {
		if response != nil {
			response.Body.Close()
		}
		if writer != nil {
			writer.Close()
		}
		var err error
		response, writer, err = s.openStream(name)
		return err == nil && response.StatusCode == 200 && response.Header.Get("Echo-Build") == build
	})
	s.t.Cleanup(func() { writer.Close(); response.Body.Close() })
	return response, writer
}

func (s *suite) finishStream(response *http.Response, writer *io.PipeWriter) {
	s.t.Helper()
	if _, err := writer.Write(echoFrame); err != nil {
		s.t.Fatal("existing stream interrupted:", err)
	}
	b := make([]byte, len(echoFrame))
	if _, err := io.ReadFull(response.Body, b); err != nil || !bytes.Equal(b, echoFrame) {
		s.t.Fatalf("existing stream lost data: %v %q", err, b)
	}
	writer.Close()
	if _, err := io.Copy(io.Discard, response.Body); err != nil || response.Trailer.Get("Grpc-Status") != "0" {
		s.t.Fatalf("stream did not finish cleanly: %v trailers=%v", err, response.Trailer)
	}
	response.Body.Close()
}

// Keep one stream on every execution so any runner selected for removal must
// preserve a stream until it finishes, rather than testing only empty runners.
func (s *suite) holdStreams(name, build string, count int) func() {
	s.t.Helper()
	type stream struct {
		response *http.Response
		writer   *io.PipeWriter
	}
	streams := map[string]stream{}
	eventually(s.t, "open stream on each execution", 30*time.Second, func() bool {
		r, w := s.stream(name, build)
		id := r.Header.Get("Echo-Instance")
		if id == "" {
			s.t.Fatal("fixture must identify its execution")
		}
		if _, found := streams[id]; found {
			s.finishStream(r, w)
		} else {
			streams[id] = stream{r, w}
		}
		return len(streams) == count
	})
	return func() {
		for _, stream := range streams {
			s.finishStream(stream.response, stream.writer)
		}
	}
}

func (s *suite) draining(pods []core.Pod) {
	s.t.Helper()
	eventually(s.t, "existing execution enters graceful drain", 60*time.Second, func() bool {
		for _, p := range pods {
			current, err := s.c.Kube.CoreV1().Pods(p.Namespace).Get(s.ctx, p.Name, meta.GetOptions{})
			if err == nil && current.DeletionTimestamp != nil {
				return true
			}
		}
		return false
	})
}

func (s *suite) executions(a *unstructured.Unstructured, count int) []core.Pod {
	s.t.Helper()
	var result []core.Pod
	eventually(s.t, fmt.Sprintf("%d stable executions", count), 90*time.Second, func() bool {
		result = s.pods(a)
		if len(result) != count {
			return false
		}
		for _, p := range result {
			if !isReady(p) {
				return false
			}
		}
		return true
	})
	return result
}

func (s *suite) runners(count int) []core.Pod {
	s.t.Helper()
	var result []core.Pod
	eventually(s.t, fmt.Sprintf("%d runner Pods and virtual Nodes", count), 120*time.Second, func() bool {
		pods, err := s.c.Kube.CoreV1().Pods("wafl-system").List(s.ctx, meta.ListOptions{LabelSelector: "app.kubernetes.io/component=runner"})
		if err != nil || len(pods.Items) != count {
			return false
		}
		result = pods.Items
		for _, p := range result {
			if !isReady(p) {
				return false
			}
		}
		nodes, err := s.c.Kube.CoreV1().Nodes().List(s.ctx, meta.ListOptions{LabelSelector: platform.Runtime + "=wasmtime"})
		if err != nil || len(nodes.Items) != count {
			return false
		}
		for _, n := range nodes.Items {
			if n.Spec.Unschedulable {
				return false
			}
			ready := false
			for _, c := range n.Status.Conditions {
				ready = ready || c.Type == core.NodeReady && c.Status == core.ConditionTrue
			}
			if !ready {
				return false
			}
		}
		return true
	})
	return result
}

func (s *suite) changeRunners(change func(*apps.Deployment)) {
	s.t.Helper()
	err := retry.RetryOnConflict(retry.DefaultBackoff, func() error {
		d, err := s.c.Kube.AppsV1().Deployments("wafl-system").Get(s.ctx, "wafl-runner", meta.GetOptions{})
		if err != nil {
			return err
		}
		change(d)
		_, err = s.c.Kube.AppsV1().Deployments("wafl-system").Update(s.ctx, d, meta.UpdateOptions{})
		return err
	})
	if err != nil {
		s.t.Fatal(err)
	}
}

// Reserve more than half of a virtual Node's memory budget so the scheduler
// must use distinct real runners. The fixture itself allocates only a few MiB.
func (s *suite) spreadMemory() int64 {
	s.t.Helper()
	nodes, err := s.c.Kube.CoreV1().Nodes().List(s.ctx, meta.ListOptions{LabelSelector: platform.Runtime + "=wasmtime"})
	if err != nil || len(nodes.Items) == 0 {
		s.t.Fatalf("virtual Nodes: %v", err)
	}
	q := nodes.Items[0].Status.Allocatable[platform.Memory]
	return q.Value() * 3 / 4
}

func (s *suite) released(p core.Pod) {
	s.t.Helper()
	s.gone(p)
	eventually(s.t, "execution receipt removed", 60*time.Second, func() bool {
		_, err := s.c.Kube.CoreV1().ConfigMaps("wafl-system").Get(s.ctx, "wafl-execution-"+string(p.UID), meta.GetOptions{})
		return apierrors.IsNotFound(err)
	})
}

func (s *suite) lifecycle(t *testing.T) {
	if !t.Run("unscheduled_scale_down", func(t *testing.T) {
		s := *s
		s.t = t
		app := s.add("pending", map[string]any{"replicas": int64(1), "manifest": map[string]any{"bundle": s.bundle("grpc-a"), "protocol": "h2c", "memoryBytes": s.spreadMemory() * 4}})
		eventually(t, "capacity leaves an unassigned Pod", 30*time.Second, func() bool {
			pods := s.pods(app)
			return len(pods) == 1 && pods[0].Spec.NodeName == "" && pods[0].Status.Phase == core.PodPending
		})
		s.patch("pending", map[string]any{"replicas": int64(0)})
		eventually(t, "cancelled unscheduled Pod is deleted", 15*time.Second, func() bool { return len(s.pods(app)) == 0 })
		s.remove(app)
	}) {
		return
	}
	if !t.Run("binary_update_and_rollback", func(t *testing.T) {
		s := *s
		s.t = t
		a, b := s.bundle("grpc-a"), s.bundle("grpc-b")
		if a["digest"] == b["digest"] {
			t.Fatal("binary versions must have different digests")
		}
		t.Logf("bundle A=%s B=%s", a["digest"], b["digest"])
		app := s.add("binary", map[string]any{"manifest": map[string]any{"bundle": a, "protocol": "h2c"}, "readiness": map[string]any{"grpc": ""}, "rollout": map[string]any{"surge": int64(1), "unavailable": int64(0), "drainMs": int64(60000)}})
		s.executions(app, 1)
		initial, writer := s.stream("binary", "a")
		stop := s.probe("binary", "a", "b")
		for _, next := range []struct {
			build  string
			bundle map[string]any
		}{{"b", b}, {"a", a}} {
			old := s.executions(app, 1)[0]
			s.patch("binary", map[string]any{"manifest": map[string]any{"bundle": next.bundle}})
			fresh, freshWriter := s.stream("binary", next.build)
			s.finishStream(initial, writer)
			s.released(old)
			initial, writer = fresh, freshWriter
			t.Logf("new binary %s serves requests; previous stream completed and execution released", next.build)
		}
		s.finishStream(initial, writer)
		stop()
		s.remove(app)
	}) {
		return
	}
	if !t.Run("runner_scale_out_and_in", func(t *testing.T) {
		s := *s
		s.t = t
		s.runners(2)
		app := s.add("runner-scale", map[string]any{"replicas": int64(2), "manifest": map[string]any{"bundle": s.bundle("grpc-a"), "protocol": "h2c", "memoryBytes": s.spreadMemory()}, "readiness": map[string]any{"grpc": ""}})
		s.executions(app, 2)
		v, w := s.stream("runner-scale", "a")
		s.finishStream(v, w)
		stop := s.probe("runner-scale", "a")
		s.changeRunners(func(d *apps.Deployment) { d.Spec.Replicas = platform.Ptr(int32(3)) })
		s.runners(3)
		s.patch("runner-scale", map[string]any{"replicas": int64(3)})
		before := s.executions(app, 3)
		nodes := map[string]bool{}
		for _, p := range before {
			nodes[p.Spec.NodeName] = true
		}
		if len(nodes) != 3 {
			t.Fatal("new runner did not host an execution")
		}
		finishStreams := s.holdStreams("runner-scale", "a", 3)
		s.changeRunners(func(d *apps.Deployment) { d.Spec.Replicas = platform.Ptr(int32(2)) })
		s.draining(before)
		finishStreams()
		s.runners(2)
		s.patch("runner-scale", map[string]any{"replicas": int64(2)})
		after := s.executions(app, 2)
		for _, p := range before {
			found := false
			for _, current := range after {
				found = found || current.UID == p.UID
			}
			if !found {
				s.released(p)
			}
		}
		stop()
		t.Log("runner Pods and virtual Nodes 2 -> 3 -> 2; every runner hosted WASM; removed executions released")
		s.remove(app)
	}) {
		return
	}
	t.Run("runner_provider_image_upgrade", func(t *testing.T) {
		s := *s
		s.t = t
		baseline := os.Getenv("WAFL_TEST_BASELINE_IMAGE")
		if baseline == "" {
			t.Fatal("WAFL_TEST_BASELINE_IMAGE is required; use scripts/test-kubernetes.sh")
		}
		d, err := s.c.Kube.AppsV1().Deployments("wafl-system").Get(s.ctx, "wafl-runner", meta.GetOptions{})
		if err != nil {
			t.Fatal(err)
		}
		candidate := d.Spec.Template.Spec.Containers[0].Image
		if candidate == baseline {
			t.Fatal("upgrade must use distinct images")
		}
		setImage := func(image string) {
			s.changeRunners(func(d *apps.Deployment) {
				for i := range d.Spec.Template.Spec.Containers {
					d.Spec.Template.Spec.Containers[i].Image = image
				}
			})
		}
		waitImage := func(image string) []core.Pod {
			eventually(t, "runner/provider image "+image, 120*time.Second, func() bool {
				d, err := s.c.Kube.AppsV1().Deployments("wafl-system").Get(s.ctx, "wafl-runner", meta.GetOptions{})
				return err == nil && d.Status.ObservedGeneration >= d.Generation && d.Status.UpdatedReplicas == 2 && d.Status.Replicas == 2 && d.Status.ReadyReplicas == 2
			})
			pods := s.runners(2)
			for _, p := range pods {
				for _, c := range p.Spec.Containers {
					if c.Image != image {
						t.Fatalf("unexpected %s image %s", c.Name, c.Image)
					}
				}
			}
			return pods
		}
		setImage(baseline)
		oldRunners := waitImage(baseline)
		app := s.add("upgrade", map[string]any{"replicas": int64(2), "manifest": map[string]any{"bundle": s.bundle("grpc-a"), "protocol": "h2c", "memoryBytes": s.spreadMemory()}, "readiness": map[string]any{"grpc": ""}})
		oldExecutions := s.executions(app, 2)
		v, w := s.stream("upgrade", "a")
		s.finishStream(v, w)
		stop := s.probe("upgrade", "a")
		finishStreams := s.holdStreams("upgrade", "a", 2)
		setImage(candidate)
		s.draining(oldExecutions)
		finishStreams()
		newRunners := waitImage(candidate)
		s.executions(app, 2)
		for _, p := range oldExecutions {
			s.released(p)
		}
		for _, old := range oldRunners {
			for _, next := range newRunners {
				for _, previous := range old.Status.ContainerStatuses {
					for _, current := range next.Status.ContainerStatuses {
						if previous.Name == current.Name && (current.ImageID == "" || current.ImageID == previous.ImageID) {
							t.Fatalf("%s image content did not change: %s", current.Name, current.ImageID)
						}
					}
				}
			}
		}
		stop()
		t.Logf("upgraded runner + provider from %s to %s", baseline, candidate)
		s.remove(app)
	})
}
