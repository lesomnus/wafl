package platform

import (
	"context"
	"encoding/json"
	"net/http"
	"net/http/httptest"
	"strings"
	"testing"

	"connectrpc.com/connect"
	wf "github.com/lesomnus/wafl/kubernetes/gen/wafl"
	core "k8s.io/api/core/v1"
	meta "k8s.io/apimachinery/pkg/apis/meta/v1"
	"k8s.io/apimachinery/pkg/apis/meta/v1/unstructured"
	"k8s.io/apimachinery/pkg/runtime"
	"k8s.io/apimachinery/pkg/types"
	"k8s.io/client-go/kubernetes/fake"
	ktesting "k8s.io/client-go/testing"
)

func fixture() (*unstructured.Unstructured, *wf.App) {
	a := &unstructured.Unstructured{Object: map[string]any{"apiVersion": Group + "/v1alpha1", "kind": "App", "metadata": map[string]any{"name": "example", "namespace": "demo", "uid": "11111111-1111-4111-8111-111111111111"}, "spec": map[string]any{"domain": "example.test", "manifest": map[string]any{"bundle": map[string]any{"url": "http://assets/app.tar.gz", "digest": "sha256:" + strings.Repeat("a", 64)}}}}}
	s, e := decodeApp(a)
	if e != nil {
		panic(e)
	}
	return a, s
}
func TestSecretRotationPinsOldRevision(t *testing.T) {
	a, s := fixture()
	s.Secrets = []*wf.Secret{{Env: "PASSWORD", Source: &wf.Secret_SecretKeyRef{SecretKeyRef: &wf.SecretKeyRef{Name: "database", Key: "url"}}}}
	source := &core.Secret{ObjectMeta: meta.ObjectMeta{Name: "database", Namespace: "demo", UID: types.UID("source"), ResourceVersion: "1"}, Data: map[string][]byte{"url": []byte("canary-old")}}
	c := &Clients{Kube: fake.NewClientset(source)}
	first, e := c.snapshot(context.Background(), a, s)
	if e != nil {
		t.Fatal(e)
	}
	old := deployment(a, s, first)
	source.ResourceVersion = "2"
	source.Data["url"] = []byte("canary-new")
	if _, e = c.Kube.CoreV1().Secrets("demo").Update(context.Background(), source, meta.UpdateOptions{}); e != nil {
		t.Fatal(e)
	}
	second, e := c.snapshot(context.Background(), a, s)
	if e != nil {
		t.Fatal(e)
	}
	next := deployment(a, s, second)
	if first[0].ValueFrom.SecretKeyRef.Name == second[0].ValueFrom.SecretKeyRef.Name || old.Spec.Template.Annotations[Revision] == next.Spec.Template.Annotations[Revision] {
		t.Fatal("rotation did not produce an immutable new revision")
	}
	oldSecret, e := c.Kube.CoreV1().Secrets("demo").Get(context.Background(), first[0].ValueFrom.SecretKeyRef.Name, meta.GetOptions{})
	if e != nil || string(oldSecret.Data["value"]) != "canary-old" {
		t.Fatal("old snapshot changed")
	}
	b, _ := json.Marshal(next)
	if strings.Contains(string(b), "canary-") {
		t.Fatal("secret escaped into Pod template")
	}
	if next.Spec.Strategy.RollingUpdate.MaxUnavailable.IntValue() != 0 || next.Spec.Strategy.RollingUpdate.MaxSurge.IntValue() != 1 {
		t.Fatal("unsafe rollout defaults")
	}
}
func TestFailedSecretResolutionDoesNotChangeDeployment(t *testing.T) {
	a, s := fixture()
	c := &Clients{Kube: fake.NewClientset()}
	s.Secrets = []*wf.Secret{{Env: "TOKEN", Source: &wf.Secret_SecretKeyRef{SecretKeyRef: &wf.SecretKeyRef{Name: "missing", Key: "value"}}}}
	if _, e := c.snapshot(context.Background(), a, s); e == nil {
		t.Fatal("required secret missing")
	}
	s.Secrets[0].GetSecretKeyRef().Optional = true
	if values, e := c.snapshot(context.Background(), a, s); e != nil || len(values) != 0 {
		t.Fatal(e)
	}
}
func TestUnsupportedPodIsRejectedBeforeProvisioning(t *testing.T) {
	a, s := fixture()
	template := deployment(a, s, nil).Spec.Template
	p := &core.Pod{ObjectMeta: template.ObjectMeta, Spec: template.Spec}
	p.Spec.SecurityContext = &core.PodSecurityContext{} // API default
	if _, e := validatePod(p); e != nil {
		t.Fatal(e)
	}
	p.Spec.SecurityContext.RunAsUser = Ptr(int64(1000))
	if _, e := validatePod(p); e == nil {
		t.Fatal("Linux security settings silently ignored")
	}
	p.Spec.SecurityContext = nil
	delete(p.Spec.Containers[0].Resources.Requests, Memory)
	if _, e := validatePod(p); e == nil {
		t.Fatal("underreported memory accepted")
	}
	p.Spec.Containers[0].Resources.Requests = deployment(a, s, nil).Spec.Template.Spec.Containers[0].Resources.Requests
	p.Spec.Containers[0].Command = []string{"sh"}
	if _, e := validatePod(p); e == nil {
		t.Fatal("Linux exec accepted")
	}
	p.Spec.Containers[0].Command = nil
	p.Spec.InitContainers = []core.Container{{Name: "sidecar", Image: "busybox"}}
	if _, e := validatePod(p); e == nil {
		t.Fatal("unsupported init container accepted")
	}
}
func TestHTTPReadinessAndGRPCDeadline(t *testing.T) {
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		if r.URL.Path != "/ready" {
			w.WriteHeader(503)
		} else {
			w.WriteHeader(204)
		}
	}))
	defer server.Close()
	a, s := fixture()
	s.Readiness = &wf.Probe{Kind: &wf.Probe_Http{Http: "/ready"}}
	p := &core.Pod{Spec: deployment(a, s, nil).Spec.Template.Spec}
	if e := probe(context.Background(), p, strings.TrimPrefix(server.URL, "http://")); e != nil {
		t.Fatal(e)
	}
	p.Spec.Containers[0].ReadinessProbe.HTTPGet.Path = "/initializing"
	if e := probe(context.Background(), p, strings.TrimPrefix(server.URL, "http://")); e == nil {
		t.Fatal("failed readiness admitted")
	}

}
func TestObserverRegistrationFencesConcurrentDeletion(t *testing.T) {
	a, s := fixture()
	template := deployment(a, s, nil).Spec.Template
	p := &core.Pod{ObjectMeta: meta.ObjectMeta{Name: "app", Namespace: "demo", UID: "22222222-2222-4222-8222-222222222222", Labels: template.Labels, Annotations: template.Annotations}, Spec: template.Spec, Status: core.PodStatus{Phase: core.PodRunning, Conditions: []core.PodCondition{{Type: core.PodReady, Status: core.ConditionTrue}}}}
	p.Annotations[Endpoint] = "127.0.0.1:20000"
	p.Labels[Placement] = UUID(placementID(p))
	k := fake.NewClientset(p)
	service := &Traffic{Clients: &Clients{Kube: k}, Namespace: "wafl-system"}
	// Delete after listing the candidate but before the post-registration GET.
	k.PrependReactor("create", "configmaps", func(action ktesting.Action) (bool, runtime.Object, error) {
		copy := p.DeepCopy()
		copy.DeletionTimestamp = Ptr(meta.Now())
		if e := k.Tracker().Update(core.SchemeGroupVersion.WithResource("pods"), copy, "demo"); e != nil {
			t.Fatal(e)
		}
		return false, nil, nil
	})
	id := ID(13, "gate")
	result, e := service.Add(context.Background(), connect.NewRequest(&wf.TrafficAddRequest{Id: id, Placement: &wf.PlacementRef{Key: &wf.PlacementRef_Id{Id: placementID(p)}}, Session: "gateway"}))
	if e != nil {
		t.Fatal(e)
	}
	if !result.Msg.Draining {
		t.Fatal("admitted a gate after deletion")
	}
	done, e := service.drained(context.Background(), string(p.UID))
	if e != nil || done {
		t.Fatal("unacknowledged gate allowed reuse")
	}
	_, e = service.Report(context.Background(), connect.NewRequest(&wf.TrafficReportRequest{Ref: &wf.TrafficRef{Key: &wf.TrafficRef_Id{Id: id}}, Sequence: 1, Closed: true}))
	if e != nil {
		t.Fatal(e)
	}
	done, e = service.drained(context.Background(), string(p.UID))
	if e != nil || !done {
		t.Fatal("closed gate did not allow cleanup")
	}
}

func TestCNIAddressReuseWaitsForOldVirtualNode(t *testing.T) {
	old := &core.Node{ObjectMeta: meta.ObjectMeta{Name: "old-runner", Labels: map[string]string{Runtime: "wasmtime"}, Annotations: map[string]string{Runner: "http://10.0.0.5:8081"}}}
	c := &Clients{Kube: fake.NewClientset(old)}
	p := NewProvider(c, "new-runner", "http://10.0.0.5:8081", "wafl-system")
	if e := p.addressAvailable(context.Background()); e == nil {
		t.Fatal("fresh journal reused quarantined IP")
	}
	p.Runner = NewRunner("http://10.0.0.6:8081")
	if e := p.addressAvailable(context.Background()); e != nil {
		t.Fatal(e)
	}
	p.Runner = NewRunner("http://10.0.0.5:8081")
	if e := c.Kube.CoreV1().Nodes().Delete(context.Background(), old.Name, meta.DeleteOptions{}); e != nil {
		t.Fatal(e)
	}
	if e := p.addressAvailable(context.Background()); e != nil {
		t.Fatal(e)
	}
}
func TestFinishCompletesAPIDeletionAfterCleanup(t *testing.T) {
	pod := &core.Pod{ObjectMeta: meta.ObjectMeta{Name: "test", Namespace: "demo", UID: "pod", Finalizers: []string{Finalizer}, DeletionTimestamp: Ptr(meta.Now())}}
	k := fake.NewClientset(pod)
	provider := NewProvider(&Clients{Kube: k}, "node", "http://runner", "wafl-system")
	if e := provider.finish(context.Background(), pod); e != nil {
		t.Fatal(e)
	}
	zero := false
	for _, action := range k.Actions() {
		if d, ok := action.(ktesting.DeleteAction); ok && d.GetResource().Resource == "pods" {
			o := d.GetDeleteOptions()
			zero = o.GracePeriodSeconds != nil && *o.GracePeriodSeconds == 0 && o.Preconditions != nil && *o.Preconditions.UID == pod.UID
		}
	}
	if !zero {
		t.Fatal("guest cleanup did not complete Kubernetes Pod deletion")
	}
}

func TestTrafficPagingSelectionAndClosedGateCounters(t *testing.T) {
	app := ID(10, "app")
	items := []*wf.Traffic{{Id: ID(13, "a"), App: &wf.App{Id: app}, Session: "first"}, {Id: ID(13, "b"), App: &wf.App{Id: app}, Session: "second"}, {Id: ID(13, "other"), App: &wf.App{Id: ID(10, "other")}}}
	request := &wf.TrafficListRequest{Size: 1, Filters: []*wf.TrafficFilter{{App: &wf.AppRef{Key: &wf.AppRef_Id{Id: app}}}}}
	first, e := trafficPage(request, items)
	if e != nil || len(first.Msg.Items) != 1 || first.Msg.Next == "" {
		t.Fatalf("first page: %v", e)
	}
	request.After = first.Msg.Next
	next, e := trafficPage(request, items)
	if e != nil || len(next.Msg.Items) != 1 || next.Msg.Next != "" || UUID(first.Msg.Items[0].Id) == UUID(next.Msg.Items[0].Id) {
		t.Fatalf("second page: %v", e)
	}
	request.Reverse = true
	if _, e = trafficPage(request, items); e == nil {
		t.Fatal("cursor order changed")
	}
	selected := clone(items[0])
	selectFields(selected.ProtoReflect(), (&wf.TrafficSelect{Session: Ptr(true)}).ProtoReflect())
	if len(selected.Id) != 16 || selected.Session != "first" || selected.App != nil {
		t.Fatal("selection contract ignored")
	}
	record := &wf.Traffic{Id: ID(13, "closed"), Closed: true, Active: 1, Started: 2, Sequence: 1}
	cm := &core.ConfigMap{ObjectMeta: meta.ObjectMeta{Name: trafficName(record.Id), Namespace: "wafl-system"}, Data: map[string]string{"value": trafficData(record)}}
	service := &Traffic{Clients: &Clients{Kube: fake.NewClientset(cm)}, Namespace: "wafl-system"}
	_, e = service.Report(context.Background(), connect.NewRequest(&wf.TrafficReportRequest{Ref: &wf.TrafficRef{Key: &wf.TrafficRef_Id{Id: record.Id}}, Sequence: 2, Closed: true, Started: 3, Active: 1}))
	if connect.CodeOf(e) != connect.CodeFailedPrecondition {
		t.Fatalf("closed observer admitted another request: %v", e)
	}
}
