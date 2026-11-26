package tests

import (
	"bytes"
	"encoding/base64"
	"encoding/hex"
	"encoding/json"
	"strings"

	"github.com/lesomnus/wafl/kubernetes/internal/platform"
	core "k8s.io/api/core/v1"
	meta "k8s.io/apimachinery/pkg/apis/meta/v1"
)

// Real Rust CLI -> Go controller -> Rust runner, without client Kubernetes
// credentials. One unreachable member must never break the healthy rows.
func (s *suite) catalog(p core.Pod) {
	s.t.Helper()
	offline := "offline-" + s.ns
	_, e := s.c.Kube.CoreV1().Nodes().Create(s.ctx, &core.Node{
		ObjectMeta: meta.ObjectMeta{Name: offline, Labels: map[string]string{platform.Runtime: "wasmtime", "kubernetes.io/os": "wasi", "kubernetes.io/arch": "wasm32"}, Annotations: map[string]string{platform.Group + "/namespace": "wafl-system", platform.Runner: "http://127.0.0.1:1"}},
		Spec:       core.NodeSpec{Unschedulable: true, Taints: []core.Taint{{Key: platform.Runtime, Value: "wasmtime", Effect: core.TaintEffectNoSchedule}}},
	}, meta.CreateOptions{})
	if e != nil {
		s.t.Fatal(e)
	}
	defer func() { _ = s.c.Kube.CoreV1().Nodes().Delete(s.ctx, offline, meta.DeleteOptions{}) }()
	query := func(args ...string) []map[string]any {
		s.t.Helper()
		stdout, stderr, err := s.clientOutput(append([]string{"--json"}, args...)...)
		if err != nil {
			s.t.Fatalf("catalog %v failed: %v\n%s", args, err, stderr)
		}
		if !strings.Contains(string(stderr), offline) {
			s.t.Fatalf("catalog %v omitted stderr warning: %s", args, stderr)
		}
		if bytes.Contains(stdout, []byte("cli-secret")) {
			s.t.Fatal("catalog exposed Secret value")
		}
		rows := []map[string]any{}
		for _, line := range bytes.Split(bytes.TrimSpace(stdout), []byte{'\n'}) {
			if len(line) == 0 {
				continue
			}
			var row map[string]any
			if err := json.Unmarshal(line, &row); err != nil {
				s.t.Fatalf("catalog stdout is not JSON: %s", stdout)
			}
			rows = append(rows, row)
		}
		return rows
	}
	instance, session, address := p.Annotations[platform.Instance], p.Annotations[platform.Session], p.Annotations[platform.Endpoint]
	encoded := func(id string) string {
		b, err := hex.DecodeString(strings.ReplaceAll(id, "-", ""))
		if err != nil {
			s.t.Fatal(err)
		}
		return base64.StdEncoding.EncodeToString(b)
	}
	found := false
	for _, row := range query("instance", "list") {
		if row["id"] == encoded(instance) {
			found = true
		}
	}
	if !found {
		s.t.Fatal("healthy instance missing from partial catalog")
	}
	if len(query("runner", "list")) == 0 {
		s.t.Fatal("healthy runners missing")
	}
	for _, args := range [][]string{
		{"runner", "get", session},
		{"runner", "get", "@" + p.Spec.NodeName},
		{"instance", "get", instance},
		{"--session", session, "instance", "start", instance},
		{"reservation", "get", address},
	} {
		if len(query(args...)) != 1 {
			s.t.Fatalf("catalog %v did not return one record", args)
		}
	}
	found = false
	for _, row := range query("reservation", "list") {
		if row["address"] == address {
			found = true
		}
	}
	if !found {
		s.t.Fatal("healthy reservation missing from partial catalog")
	}
	stdout, stderr, err := s.clientOutput("instance", "logs", instance)
	if err != nil || !bytes.Contains(stdout, []byte("guest boot")) || !strings.Contains(string(stderr), offline) {
		s.t.Fatalf("catalog logs/warning missing: %v\n%s\n%s", err, stdout, stderr)
	}
	_, _, err = s.clientOutput("--session", "00000000-0000-8000-8008-000000000099", "instance", "start", instance)
	if err == nil {
		s.t.Fatal("stale session command succeeded")
	}
}
