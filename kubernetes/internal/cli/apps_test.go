package cli

import (
	"k8s.io/apimachinery/pkg/apis/meta/v1/unstructured"
	"testing"
)

func TestNormalizationPreservesExplicitDefaultsAndOpaqueEnvironment(t *testing.T) {
	a := &unstructured.Unstructured{Object: map[string]any{"spec": map[string]any{
		"domain": "example.test", "replicas": int64(0),
		"manifest": map[string]any{"bundle": map[string]any{"url": "https://example.test/app.tar.gz"}, "memory_bytes": "67108864", "env": map[string]any{"timeoutMs": "123", "sizeBytes": "456"}},
		"rollout":  map[string]any{"surge": int64(0), "unavailable": int64(1), "drain_ms": "30000"},
	}}}
	if e := normalize(a); e != nil {
		t.Fatal(e)
	}
	spec := a.Object["spec"].(map[string]any)
	rollout := spec["rollout"].(map[string]any)
	manifest := spec["manifest"].(map[string]any)
	if rollout["surge"] != int64(0) || spec["replicas"] != int64(0) || rollout["drainMs"] != int64(30000) || manifest["memoryBytes"] != int64(67108864) {
		t.Fatalf("normalization changed explicit defaults: %v", spec)
	}
	env := manifest["env"].(map[string]any)
	if env["timeoutMs"] != "123" || env["sizeBytes"] != "456" {
		t.Fatal("environment value was interpreted as a duration")
	}
}
