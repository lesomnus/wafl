package tests

import (
	"archive/tar"
	"bytes"
	"compress/gzip"
	"context"
	"crypto/sha256"
	"encoding/json"
	"fmt"
	"io"
	"net"
	"net/http"
	"os"
	"os/exec"
	"path/filepath"
	"strings"
	"sync"
	"testing"
	"time"

	"github.com/lesomnus/wafl/kubernetes/internal/platform"
	core "k8s.io/api/core/v1"
	apierrors "k8s.io/apimachinery/pkg/api/errors"
	meta "k8s.io/apimachinery/pkg/apis/meta/v1"
	"k8s.io/apimachinery/pkg/apis/meta/v1/unstructured"
	"k8s.io/apimachinery/pkg/types"
)

type suite struct {
	t                                                      *testing.T
	c                                                      *platform.Clients
	ctx                                                    context.Context
	ns, kubeconfig, gateway, control, cluster, assets, cli string
	localBundles                                           map[string]string
}

func eventually(t *testing.T, label string, timeout time.Duration, f func() bool) {
	t.Helper()
	deadline := time.Now().Add(timeout)
	for time.Now().Before(deadline) {
		if f() {
			return
		}
		time.Sleep(250 * time.Millisecond)
	}
	t.Fatalf("timeout: %s", label)
}
func (s *suite) command(name string, args ...string) []byte {
	s.t.Helper()
	cmd := exec.CommandContext(s.ctx, name, args...)
	data, e := cmd.CombinedOutput()
	if e != nil {
		s.t.Fatalf("%s failed: %v\n%s", name, e, data)
	}
	return data
}
func (s *suite) client(args ...string) []byte {
	s.t.Helper()
	stdout, stderr, e := s.clientOutput(append([]string{"--tenant", s.ns}, args...)...)
	if e != nil {
		s.t.Fatalf("WAFL API client failed: %v\n%s\n%s", e, stdout, stderr)
	}
	return stdout
}
func (s *suite) clientOutput(args ...string) ([]byte, []byte, error) {
	s.t.Helper()
	cmd := exec.CommandContext(s.ctx, s.cli, append([]string{"--endpoint", s.control}, args...)...)
	// Only the test harness is an operator. The client has neither Kubernetes
	// credentials nor kubectl/Docker/kind executables available.
	for _, variable := range os.Environ() {
		key, _, _ := strings.Cut(variable, "=")
		switch key {
		case "PATH", "KUBECONFIG", "KUBERNETES_SERVICE_HOST", "KUBERNETES_SERVICE_PORT":
		default:
			cmd.Env = append(cmd.Env, variable)
		}
	}
	cmd.Env = append(cmd.Env, "PATH=/no-kubernetes-helpers", "KUBECONFIG=/unavailable/kubeconfig", "KUBERNETES_SERVICE_HOST=127.0.0.1", "KUBERNETES_SERVICE_PORT=1")
	var stdout, stderr bytes.Buffer
	cmd.Stdout, cmd.Stderr = &stdout, &stderr
	e := cmd.Run()
	return stdout.Bytes(), stderr.Bytes(), e
}
func (s *suite) bundle(guest string) map[string]any {
	s.t.Helper()
	data, e := os.ReadFile(filepath.Join(os.Getenv("WAFL_TEST_GUEST_DIR"), guest+".wasm"))
	if e != nil {
		s.t.Fatal(e)
	}
	var b bytes.Buffer
	z := gzip.NewWriter(&b)
	tarfile := tar.NewWriter(z)
	if e = tarfile.WriteHeader(&tar.Header{Name: "main.wasm", Mode: 0644, Size: int64(len(data))}); e != nil {
		s.t.Fatal(e)
	}
	if _, e = tarfile.Write(data); e != nil {
		s.t.Fatal(e)
	}
	if e = tarfile.Close(); e != nil {
		s.t.Fatal(e)
	}
	if e = z.Close(); e != nil {
		s.t.Fatal(e)
	}
	sum := fmt.Sprintf("%x", sha256.Sum256(b.Bytes()))
	path := filepath.Join(s.t.TempDir(), sum+".tar.gz")
	if e = os.WriteFile(path, b.Bytes(), 0600); e != nil {
		s.t.Fatal(e)
	}
	if s.localBundles == nil {
		s.localBundles = map[string]string{}
	}
	s.localBundles["sha256:"+sum] = path
	nodes := s.command("kind", "get", "nodes", "--name", s.cluster)
	for _, node := range strings.Fields(string(nodes)) {
		s.command("docker", "exec", node, "mkdir", "-p", "/wafl-assets")
		s.command("docker", "cp", path, node+":/wafl-assets/"+sum+".tar.gz")
		s.command("docker", "exec", node, "chmod", "0644", "/wafl-assets/"+sum+".tar.gz")
	}
	return map[string]any{"url": s.assets + "/" + sum + ".tar.gz", "digest": "sha256:" + sum}
}
func (s *suite) add(name string, spec map[string]any) *unstructured.Unstructured {
	s.t.Helper()
	spec["domain"] = name + "." + s.ns + ".example"
	v := &unstructured.Unstructured{Object: map[string]any{"apiVersion": platform.Group + "/v1alpha1", "kind": "App", "metadata": map[string]any{"name": name, "namespace": s.ns}, "spec": spec}}
	a, e := s.c.Dynamic.Resource(platform.Apps).Namespace(s.ns).Create(s.ctx, v, meta.CreateOptions{})
	if e != nil {
		s.t.Fatal(e)
	}
	return a
}
func (s *suite) patch(name string, v map[string]any) {
	s.t.Helper()
	b, _ := json.Marshal(map[string]any{"spec": v})
	if _, e := s.c.Dynamic.Resource(platform.Apps).Namespace(s.ns).Patch(s.ctx, name, types.MergePatchType, b, meta.PatchOptions{}); e != nil {
		s.t.Fatal(e)
	}
}
func (s *suite) pods(a *unstructured.Unstructured) []core.Pod {
	s.t.Helper()
	p, e := s.c.Kube.CoreV1().Pods(s.ns).List(s.ctx, meta.ListOptions{LabelSelector: platform.AppUID + "=" + string(a.GetUID())})
	if e != nil {
		s.t.Fatal(e)
	}
	return p.Items
}
func isReady(p core.Pod) bool {
	if p.DeletionTimestamp != nil {
		return false
	}
	for _, c := range p.Status.Conditions {
		if c.Type == core.PodReady && c.Status == core.ConditionTrue {
			return true
		}
	}
	return false
}
func (s *suite) ready(a *unstructured.Unstructured) core.Pod {
	s.t.Helper()
	var result core.Pod
	eventually(s.t, "App "+a.GetName()+" ready", 2*time.Minute, func() bool {
		for _, p := range s.pods(a) {
			if isReady(p) {
				result = p
				return true
			}
		}
		return false
	})
	return result
}
func (s *suite) http(name, path string) (int, string, error) {
	ctx, cancel := context.WithTimeout(s.ctx, 3*time.Second)
	defer cancel()
	r, _ := http.NewRequestWithContext(ctx, "GET", s.gateway+path, nil)
	r.Host = name + "." + s.ns + ".example"
	v, e := http.DefaultClient.Do(r)
	if e != nil {
		return 0, "", e
	}
	defer v.Body.Close()
	b, e := io.ReadAll(v.Body)
	return v.StatusCode, string(b), e
}
func (s *suite) response(name, want string) {
	s.t.Helper()
	eventually(s.t, "gateway "+name+" -> "+want, 45*time.Second, func() bool {
		status, body, e := s.http(name, "/")
		return e == nil && status == 200 && strings.HasPrefix(body, want)
	})
}
func (s *suite) gone(p core.Pod) {
	s.t.Helper()
	eventually(s.t, "old execution released", 60*time.Second, func() bool {
		_, e := s.c.Kube.CoreV1().Pods(p.Namespace).Get(s.ctx, p.Name, meta.GetOptions{})
		return apierrors.IsNotFound(e)
	})
}
func (s *suite) remove(a *unstructured.Unstructured) {
	s.t.Helper()
	if e := s.c.Dynamic.Resource(platform.Apps).Namespace(s.ns).Delete(s.ctx, a.GetName(), meta.DeleteOptions{}); e != nil {
		s.t.Fatal(e)
	}
	eventually(s.t, "App deleted", 60*time.Second, func() bool { return len(s.pods(a)) == 0 })
}
func (s *suite) forward(resource string, remote int) string {
	s.t.Helper()
	l, e := net.Listen("tcp", "127.0.0.1:0")
	if e != nil {
		s.t.Fatal(e)
	}
	port := l.Addr().(*net.TCPAddr).Port
	l.Close()
	ctx, cancel := context.WithCancel(s.ctx)
	cmd := exec.CommandContext(ctx, "kubectl", "--kubeconfig", s.kubeconfig, "-n", "wafl-system", "port-forward", resource, fmt.Sprintf("%d:%d", port, remote))
	if e = cmd.Start(); e != nil {
		cancel()
		s.t.Fatal(e)
	}
	s.t.Cleanup(func() { cancel(); _ = cmd.Wait() })
	address := fmt.Sprintf("127.0.0.1:%d", port)
	eventually(s.t, "port forward", 15*time.Second, func() bool {
		c, e := net.DialTimeout("tcp", address, time.Second)
		if e == nil {
			c.Close()
		}
		return e == nil
	})
	return "http://" + address
}
func TestNativePlatform(t *testing.T) {
	kubeconfig := os.Getenv("WAFL_TEST_KUBECONFIG")
	if kubeconfig == "" {
		t.Skip("set WAFL_TEST_KUBECONFIG to a disposable installed kind cluster")
	}
	ctx, cancel := context.WithTimeout(context.Background(), 15*time.Minute)
	defer cancel()
	c, e := platform.Open(kubeconfig, "")
	if e != nil {
		t.Fatal(e)
	}
	s := &suite{t: t, c: c, ctx: ctx, ns: fmt.Sprintf("wafl-test-%d", time.Now().UnixNano()), kubeconfig: kubeconfig, cluster: os.Getenv("WAFL_TEST_CLUSTER"), assets: "http://wafl-assets.wafl-system.svc:8090", cli: os.Getenv("WAFL_TEST_CLI")}
	if s.cluster == "" {
		s.cluster = "wafl-test"
	}
	if _, e = c.Kube.CoreV1().Namespaces().Create(ctx, &core.Namespace{ObjectMeta: meta.ObjectMeta{Name: s.ns}}, meta.CreateOptions{}); e != nil {
		t.Fatal(e)
	}
	t.Cleanup(func() {
		_ = c.Dynamic.Resource(platform.Apps).Namespace(s.ns).DeleteCollection(context.Background(), meta.DeleteOptions{}, meta.ListOptions{})
		_ = c.Kube.CoreV1().Namespaces().Delete(context.Background(), s.ns, meta.DeleteOptions{})
	})
	s.gateway = os.Getenv("WAFL_TEST_GATEWAY")
	if s.gateway == "" {
		s.gateway = s.forward("svc/wafl-traefik", 80)
	}
	socket := s.bundle("socket")
	grpc := s.bundle("grpc")
	t.Run("readiness_rollout_and_service", func(t *testing.T) {
		s := *s
		s.t = t
		a := s.add("web", map[string]any{"manifest": map[string]any{"bundle": socket, "env": map[string]any{"GREETING": "old"}}, "readiness": map[string]any{"http": "/ready"}})
		old := s.ready(a)
		s.response("web", "old:")
		s.patch("web", map[string]any{"manifest": map[string]any{"env": map[string]any{"GREETING": "new", "READY": "false"}}})
		eventually(t, "unready replacement exists", 30*time.Second, func() bool { return len(s.pods(a)) >= 2 })
		for n := 0; n < 10; n++ {
			status, body, e := s.http("web", "/")
			if e != nil || status != 200 || !strings.HasPrefix(body, "old:") {
				t.Fatalf("unready update displaced healthy version: %d %q %v", status, body, e)
			}
			time.Sleep(200 * time.Millisecond)
		}
		s.patch("web", map[string]any{"manifest": map[string]any{"env": map[string]any{"READY": "true"}}})
		s.response("web", "new:")
		s.gone(old)
		services, e := c.Kube.CoreV1().Services(s.ns).List(ctx, meta.ListOptions{LabelSelector: platform.AppUID + "=" + string(a.GetUID())})
		if e != nil || len(services.Items) != 1 {
			t.Fatalf("App Service: %v", e)
		}
		// A normal ClusterIP must route through gateway admission too.
		d, e := c.Kube.AppsV1().Deployments(s.ns).List(ctx, meta.ListOptions{LabelSelector: platform.AppUID + "=" + string(a.GetUID())})
		if e != nil || len(d.Items) != 1 {
			t.Fatal(e)
		}
		_ = services
		body := s.command("kubectl", "--kubeconfig", s.kubeconfig, "-n", "wafl-system", "exec", "deployment/wafl-traefik", "--", "wget", "-qO-", "http://"+services.Items[0].Name+"."+s.ns+".svc/")
		if !strings.HasPrefix(string(body), "new:") {
			t.Fatalf("ClusterIP route failed: %s", body)
		}
		s.remove(a)
	})
	t.Run("grpc_secret_rotation_drains_old_stream", func(t *testing.T) {
		s := *s
		s.t = t
		source := &core.Secret{ObjectMeta: meta.ObjectMeta{Name: "greeting", Namespace: s.ns}, Data: map[string][]byte{"value": []byte("old-secret")}}
		source, e = c.Kube.CoreV1().Secrets(s.ns).Create(ctx, source, meta.CreateOptions{})
		if e != nil {
			t.Fatal(e)
		}
		a := s.add("rpc", map[string]any{"manifest": map[string]any{"bundle": grpc, "protocol": "h2c"}, "readiness": map[string]any{"grpc": ""}, "rollout": map[string]any{"surge": int64(1), "unavailable": int64(0), "drainMs": int64(60000)}, "secrets": []any{map[string]any{"env": "GREETING", "secretKeyRef": map[string]any{"name": "greeting", "key": "value"}}}})
		old := s.ready(a)
		var stream *http.Response
		var writer *io.PipeWriter
		eventually(t, "gRPC route", 45*time.Second, func() bool {
			if stream != nil {
				stream.Body.Close()
				writer.Close()
			}
			stream, writer, e = s.openStream("rpc")
			return e == nil && stream.Header.Get("Echo-Version") == "old-secret"
		})
		defer stream.Body.Close()
		defer writer.Close()
		frame := []byte{0, 0, 0, 0, 4, 'p', 'i', 'n', 'g'}
		if _, e = writer.Write(frame); e != nil {
			t.Fatal(e)
		}
		reply := make([]byte, len(frame))
		if _, e = io.ReadFull(stream.Body, reply); e != nil || !bytes.Equal(reply, frame) {
			t.Fatalf("initial stream: %v", e)
		}
		source.Data["value"] = []byte("new-secret")
		if _, e = c.Kube.CoreV1().Secrets(s.ns).Update(ctx, source, meta.UpdateOptions{}); e != nil {
			t.Fatal(e)
		}
		eventually(t, "new secret served", 45*time.Second, func() bool {
			v, w, e := s.openStream("rpc")
			if e != nil {
				return false
			}
			w.Close()
			io.Copy(io.Discard, v.Body)
			v.Body.Close()
			return v.Header.Get("Echo-Version") == "new-secret"
		})
		p, e := c.Kube.CoreV1().Pods(s.ns).Get(ctx, old.Name, meta.GetOptions{})
		if e != nil || p.DeletionTimestamp == nil {
			t.Fatalf("old stream Pod should be draining: %v", e)
		}
		if _, e = writer.Write(frame); e != nil {
			t.Fatal(e)
		}
		if _, e = io.ReadFull(stream.Body, reply); e != nil || !bytes.Equal(reply, frame) {
			t.Fatalf("old stream interrupted by rollout: %v", e)
		}
		writer.Close()
		if _, e = io.Copy(io.Discard, stream.Body); e != nil {
			t.Fatal(e)
		}
		if stream.Trailer.Get("Grpc-Status") != "0" {
			t.Fatalf("lost trailers: %v", stream.Trailer)
		}
		s.gone(old)
		for _, p := range s.pods(a) {
			b, _ := json.Marshal(p)
			if strings.Contains(string(b), "new-secret") || strings.Contains(string(b), "old-secret") {
				t.Fatal("resolved secret leaked in Pod")
			}
		}
		s.remove(a)
	})
	t.Run("cold_start_load_scaling_and_runner_failure", func(t *testing.T) {
		s := *s
		s.t = t
		a := s.add("cold", map[string]any{"manifest": map[string]any{"bundle": socket, "env": map[string]any{"GREETING": "cold"}}, "scaling": map[string]any{"min": int64(0), "max": int64(2), "target": int64(1), "upMs": int64(1000), "downMs": int64(1000), "idleMs": int64(4000), "waitMs": int64(60000)}})
		eventually(t, "zero deployment exists", 30*time.Second, func() bool {
			d, e := c.Kube.AppsV1().Deployments(s.ns).List(ctx, meta.ListOptions{LabelSelector: platform.AppUID + "=" + string(a.GetUID())})
			return e == nil && len(d.Items) == 1 && *d.Items[0].Spec.Replicas == 0
		})
		if len(s.pods(a)) != 0 {
			t.Fatal("zero App started before request")
		}
		s.response("cold", "cold:")
		load, cancel := context.WithCancel(ctx)
		var wg sync.WaitGroup
		for n := 0; n < 8; n++ {
			wg.Add(1)
			go func() {
				defer wg.Done()
				for load.Err() == nil {
					_, _, _ = s.http("cold", "/slow")
				}
			}()
		}
		eventually(t, "load creates second execution", 45*time.Second, func() bool {
			n := 0
			for _, p := range s.pods(a) {
				if isReady(p) {
					n++
				}
			}
			return n == 2
		})
		cancel()
		wg.Wait()
		// Hold a minimum while forcing loss of an actual runner Pod.
		s.patch("cold", map[string]any{"scaling": map[string]any{"min": int64(1)}})
		old := s.ready(a)
		physical := strings.TrimPrefix(old.Annotations[platform.RunnerPod], "wafl-system/")
		if physical == "" {
			t.Fatal("physical runner identity missing")
		}
		if e = c.Kube.CoreV1().Pods("wafl-system").Delete(ctx, physical, meta.DeleteOptions{GracePeriodSeconds: platform.Ptr(int64(0))}); e != nil {
			t.Fatal(e)
		}
		eventually(t, "runner failure replacement", 60*time.Second, func() bool {
			for _, p := range s.pods(a) {
				if p.UID != old.UID && isReady(p) && p.Spec.NodeName != old.Spec.NodeName {
					return true
				}
			}
			return false
		})
		s.response("cold", "cold:")
		s.gone(old)
		s.patch("cold", map[string]any{"scaling": map[string]any{"min": int64(0)}})
		eventually(t, "idle scale to zero", 45*time.Second, func() bool { return len(s.pods(a)) == 0 })
		s.response("cold", "cold:")
		s.remove(a)
	})
	t.Run("provider_and_controller_restart", func(t *testing.T) {
		s := *s
		s.t = t
		a := s.add("recovery", map[string]any{"manifest": map[string]any{"bundle": socket, "env": map[string]any{"GREETING": "stable"}}})
		old := s.ready(a)
		s.response("recovery", "stable:")
		physical := strings.TrimPrefix(old.Annotations[platform.RunnerPod], "wafl-system/")
		runner, e := c.Kube.CoreV1().Pods("wafl-system").Get(ctx, physical, meta.GetOptions{})
		if e != nil {
			t.Fatal(e)
		}
		count := int32(0)
		for _, cs := range runner.Status.ContainerStatuses {
			if cs.Name == "node" {
				count = cs.RestartCount
			}
		}
		// Stopping just the provider must preserve the runner session and execution.
		command := exec.CommandContext(ctx, "kubectl", "--kubeconfig", s.kubeconfig, "-n", "wafl-system", "exec", physical, "-c", "node", "--", "sh", "-c", "kill -TERM 1")
		_ = command.Run()
		eventually(t, "provider container restarted", 45*time.Second, func() bool {
			p, e := c.Kube.CoreV1().Pods("wafl-system").Get(ctx, physical, meta.GetOptions{})
			if e != nil {
				return false
			}
			for _, cs := range p.Status.ContainerStatuses {
				if cs.Name == "node" && cs.Ready && cs.RestartCount > count {
					return true
				}
			}
			return false
		})
		s.response("recovery", "stable:")
		current, e := c.Kube.CoreV1().Pods(s.ns).Get(ctx, old.Name, meta.GetOptions{})
		if e != nil || current.UID != old.UID || current.Annotations[platform.Instance] != old.Annotations[platform.Instance] || current.Annotations[platform.Session] != old.Annotations[platform.Session] {
			t.Fatalf("provider restart replaced live execution: %v", e)
		}
		deployments, e := c.Kube.AppsV1().Deployments(s.ns).List(ctx, meta.ListOptions{LabelSelector: platform.AppUID + "=" + string(a.GetUID())})
		if e != nil || len(deployments.Items) != 1 {
			t.Fatal(e)
		}
		d := deployments.Items[0]
		lease, e := c.Kube.CoordinationV1().Leases("wafl-system").Get(ctx, "wafl-controller", meta.GetOptions{})
		if e != nil {
			t.Fatal(e)
		}
		holder := *lease.Spec.HolderIdentity
		leader := holder[:strings.LastIndex(holder, "-")]
		if e = c.Kube.CoreV1().Pods("wafl-system").Delete(ctx, leader, meta.DeleteOptions{GracePeriodSeconds: platform.Ptr(int64(0))}); e != nil {
			t.Fatal(e)
		}
		eventually(t, "new reconciliation leader", 40*time.Second, func() bool {
			status, body, e := s.http("recovery", "/")
			if e != nil || status != 200 || !strings.HasPrefix(body, "stable:") {
				t.Fatalf("controller failover interrupted data path: %v %d %s", e, status, body)
			}
			l, e := c.Kube.CoordinationV1().Leases("wafl-system").Get(ctx, "wafl-controller", meta.GetOptions{})
			return e == nil && l.Spec.HolderIdentity != nil && *l.Spec.HolderIdentity != "" && *l.Spec.HolderIdentity != holder
		})
		time.Sleep(2 * time.Second)
		next, e := c.Kube.AppsV1().Deployments(s.ns).Get(ctx, d.Name, meta.GetOptions{})
		if e != nil || next.Spec.Template.Annotations[platform.Revision] != d.Spec.Template.Annotations[platform.Revision] {
			t.Fatalf("leader change invented an App revision: %v", e)
		}
		s.remove(a)
	})
	t.Run("cli_controller_api_without_kubernetes", func(t *testing.T) {
		if s.cli == "" {
			t.Skip("set WAFL_TEST_CLI to test native CLI")
		}
		s := *s
		s.t = t
		s.control = os.Getenv("WAFL_TEST_CONTROL")
		if s.control == "" {
			s.control = s.forward("svc/wafl-controller", 8082)
		}
		dir := t.TempDir()
		secret := filepath.Join(dir, "value")
		if e = os.WriteFile(secret, []byte("cli-secret"), 0600); e != nil {
			t.Fatal(e)
		}
		spec := map[string]any{"alias": "cli", "domain": "cli." + s.ns + ".example", "manifest": map[string]any{"bundle": map[string]any{"url": "file:" + s.localBundles[socket["digest"].(string)]}}, "secrets": []any{map[string]any{"env": "GREETING", "file": "value"}}}
		b, _ := json.Marshal(spec)
		file := filepath.Join(dir, "app.yaml")
		if e = os.WriteFile(file, b, 0600); e != nil {
			t.Fatal(e)
		}
		s.client("app", "add", file)
		s.response("cli", "cli-secret:")
		logs := s.client("app", "logs", "@cli")
		if !strings.Contains(string(logs), "guest boot") {
			t.Fatalf("missing guest log: %s", logs)
		}
		a, e := c.Dynamic.Resource(platform.Apps).Namespace(s.ns).Get(ctx, "cli", meta.GetOptions{})
		if e != nil {
			t.Fatal(e)
		}
		b, _ = json.Marshal(a)
		if strings.Contains(string(b), "cli-secret") {
			t.Fatal("CLI leaked file contents")
		}
		old := s.ready(a)
		for _, result := range [][]byte{s.client("app", "get", "@cli"), s.client("app", "list")} {
			if !bytes.Contains(result, []byte(`"cli"`)) || bytes.Contains(result, []byte("cli-secret")) {
				t.Fatal("App API response is missing App or exposes a secret")
			}
		}
		s.catalog(old)
		if e = os.WriteFile(secret, []byte("rotated-secret"), 0600); e != nil {
			t.Fatal(e)
		}
		patch := filepath.Join(dir, "patch.yaml")
		if e = os.WriteFile(patch, []byte("secrets:\n- env: GREETING\n  file: value\n"), 0600); e != nil {
			t.Fatal(e)
		}
		s.client("app", "patch", "@cli", "--file", patch)
		s.response("cli", "rotated-secret:")
		s.gone(old)
		s.client("traffic", "list")
		s.client("app", "erase", "@cli")
		eventually(t, "API erased execution", 60*time.Second, func() bool { return len(s.pods(a)) == 0 })
	})
}
func (s *suite) openStream(name string) (*http.Response, *io.PipeWriter, error) {
	r, w := io.Pipe()
	ctx, cancel := context.WithTimeout(s.ctx, 120*time.Second)
	s.t.Cleanup(cancel)
	req, e := http.NewRequestWithContext(ctx, "POST", s.gateway+"/echo.Echo/Stream", r)
	if e != nil {
		w.Close()
		return nil, w, e
	}
	req.Host = name + "." + s.ns + ".example"
	req.Header.Set("Content-Type", "application/grpc")
	req.Header.Set("TE", "trailers")
	req.Header.Set("Grpc-Timeout", "120S")
	transport := http.DefaultTransport.(*http.Transport).Clone()
	transport.Protocols = new(http.Protocols)
	transport.Protocols.SetUnencryptedHTTP2(true)
	transport.ResponseHeaderTimeout = 5 * time.Second
	// Reverse proxies may wait for the first response DATA before flushing
	// headers. Start a valid empty gRPC message before awaiting the headers.
	prime := []byte{0, 0, 0, 0, 0}
	go func() { _, _ = w.Write(prime) }()
	client := &http.Client{Transport: transport}
	response, e := client.Do(req)
	if e != nil {
		w.Close()
	} else if response.Header.Get("Echo-Version") != "" {
		data := make([]byte, 5)
		_, e = io.ReadFull(response.Body, data)
		if e == nil && !bytes.Equal(data, prime) {
			e = fmt.Errorf("invalid streaming echo")
		}
	}
	return response, w, e
}
