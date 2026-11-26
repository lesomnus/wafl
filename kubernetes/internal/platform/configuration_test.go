package platform

import (
	"connectrpc.com/connect"
	"context"
	wf "github.com/lesomnus/wafl/kubernetes/gen/wafl"
	core "k8s.io/api/core/v1"
	meta "k8s.io/apimachinery/pkg/apis/meta/v1"
	"k8s.io/apimachinery/pkg/runtime"
	"k8s.io/apimachinery/pkg/runtime/schema"
	dynamicfake "k8s.io/client-go/dynamic/fake"
	"k8s.io/client-go/kubernetes/fake"
	"testing"
)

func TestProxyRoutingAndWake(t *testing.T) {
	ctx := context.Background()
	a, spec := fixture()
	a.Object["spec"].(map[string]any)["replicas"] = int64(0)
	a.Object["spec"].(map[string]any)["scaling"] = map[string]any{"min": int64(0), "max": int64(2), "waiting": int64(1)}
	spec.Replicas = 0
	d := deployment(a, spec, nil)
	clients := &Clients{Kube: fake.NewClientset(&core.Namespace{ObjectMeta: meta.ObjectMeta{Name: "demo"}}, d), Dynamic: dynamicfake.NewSimpleDynamicClientWithCustomListKinds(runtime.NewScheme(), map[schema.GroupVersionResource]string{Apps: "AppList"}, a)}
	c := &Controller{Clients: clients, Namespace: "wafl-system"}
	snapshot, err := c.routing(ctx)
	if err != nil || snapshot.Version != 1 || len(snapshot.Routes) != 1 || snapshot.Routes[0].Suspended || len(snapshot.Routes[0].Targets) != 0 {
		t.Fatalf("cold route: %v %v", snapshot, err)
	}
	route := snapshot.Routes[0]
	traffic := &Traffic{Clients: clients, Namespace: "wafl-system"}
	id := ID(13, "wait-1")
	add := connect.NewRequest(&wf.TrafficAddRequest{Id: id, App: &wf.AppRef{Key: &wf.AppRef_Id{Id: route.App}}, Session: "proxy"})
	add.Header().Set("wafl-tenant", "demo")
	observation, err := traffic.Add(ctx, add)
	if err != nil || !observation.Msg.Activation || observation.Msg.Active != 1 {
		t.Fatalf("waiting registration: %v %v", observation, err)
	}
	if _, err = traffic.Add(ctx, add); err != nil {
		t.Fatal("idempotent Add", err)
	}
	other := connect.NewRequest(&wf.TrafficAddRequest{Id: ID(13, "wait-2"), App: add.Msg.App, Session: "proxy"})
	other.Header().Set("wafl-tenant", "demo")
	if _, err = traffic.Add(ctx, other); connect.CodeOf(err) != connect.CodeResourceExhausted {
		t.Fatal("waiting limit", err)
	}
	wake := connect.NewRequest(&wf.AppWakeRequest{Ref: add.Msg.App})
	wake.Header().Set("wafl-tenant", "demo")
	api := &AppAPI{Clients: clients}
	for i := 0; i < 2; i++ {
		if _, err = api.Wake(ctx, wake); err != nil {
			t.Fatal("Wake", err)
		}
	}
	current, err := clients.Kube.AppsV1().Deployments("demo").Get(ctx, d.Name, meta.GetOptions{})
	if err != nil || *current.Spec.Replicas != 1 {
		t.Fatal("Wake must request one execution", current, err)
	}
	closed, err := traffic.Report(ctx, connect.NewRequest(&wf.TrafficReportRequest{Ref: &wf.TrafficRef{Key: &wf.TrafficRef_Id{Id: id}}, Sequence: 1, Started: 1, Closed: true}))
	if err != nil || !closed.Msg.Closed || closed.Msg.Active != 0 {
		t.Fatal("waiting close", closed, err)
	}
	delete(a.Object["spec"].(map[string]any), "scaling")
	if _, err = clients.Dynamic.Resource(Apps).Namespace("demo").Update(ctx, a, meta.UpdateOptions{}); err != nil {
		t.Fatal(err)
	}
	snapshot, err = c.routing(ctx)
	if err != nil || !snapshot.Routes[0].Suspended {
		t.Fatal("fixed zero must remain suspended")
	}
	if _, err = api.Wake(ctx, wake); connect.CodeOf(err) != connect.CodeFailedPrecondition {
		t.Fatal("fixed zero activated", err)
	}
}
