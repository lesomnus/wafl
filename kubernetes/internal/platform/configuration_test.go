package platform

import (
	"context"
	"testing"

	core "k8s.io/api/core/v1"
	meta "k8s.io/apimachinery/pkg/apis/meta/v1"
	"k8s.io/apimachinery/pkg/runtime"
	"k8s.io/apimachinery/pkg/runtime/schema"
	dynamicfake "k8s.io/client-go/dynamic/fake"
	"k8s.io/client-go/kubernetes/fake"
)

func TestUnavailableAppRouting(t *testing.T) {
	for _, tc := range []struct {
		name     string
		replicas int64
		scaling  bool
		protocol string
		want     string
	}{
		{"fixed HTTP recovering", 2, false, "http", "http://controller:8083"},
		{"fixed gRPC recovering", 2, false, "h2c", "h2c://controller:8083"},
		{"suspended", 0, false, "http", ""},
		{"autoscaling zero", 0, true, "h2c", "h2c://controller:8083"},
	} {
		t.Run(tc.name, func(t *testing.T) {
			a, _ := fixture()
			spec := a.Object["spec"].(map[string]any)
			spec["replicas"] = tc.replicas
			spec["manifest"].(map[string]any)["protocol"] = tc.protocol
			if tc.scaling {
				spec["scaling"] = map[string]any{"min": int64(0), "max": int64(2)}
			}
			c := &Controller{Activate: "http://controller:8083", Clients: &Clients{
				Kube:    fake.NewClientset(&core.Namespace{ObjectMeta: meta.ObjectMeta{Name: "demo"}}),
				Dynamic: dynamicfake.NewSimpleDynamicClientWithCustomListKinds(runtime.NewScheme(), map[schema.GroupVersionResource]string{Apps: "AppList"}, a),
			}}
			config, err := c.configuration(context.Background())
			if err != nil {
				t.Fatal(err)
			}
			services := config["http"].(map[string]any)["services"].(map[string]any)
			servers := services[appName(a)].(map[string]any)["loadBalancer"].(map[string]any)["servers"].([]any)
			if tc.want == "" {
				if len(servers) != 0 {
					t.Fatal("a suspended App must not activate")
				}
			} else if len(servers) != 1 || servers[0].(map[string]string)["url"] != tc.want {
				t.Fatalf("requests should wait at %s: %v", tc.want, servers)
			}
		})
	}
}
