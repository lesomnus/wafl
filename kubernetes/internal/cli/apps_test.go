package cli

import (
	"bytes"
	"context"
	"crypto/sha256"
	"fmt"
	"io"
	"net/http"
	"net/http/httptest"
	"os"
	"path/filepath"
	"testing"

	"connectrpc.com/connect"
	"github.com/lesomnus/protobuf-patch/patchproto"
	wf "github.com/lesomnus/wafl/kubernetes/gen/wafl"
	"github.com/lesomnus/wafl/kubernetes/gen/wafl/waflconnect"
	"google.golang.org/protobuf/proto"
)

type appStub struct {
	waflconnect.UnimplementedAppServiceHandler
	add   func(context.Context, *connect.Request[wf.AppAddRequest]) (*connect.Response[wf.App], error)
	get   func(context.Context, *connect.Request[wf.AppGetRequest]) (*connect.Response[wf.App], error)
	apply func(context.Context, *connect.Request[wf.AppApplyRequest]) (*connect.Response[wf.App], error)
}

func (s *appStub) Add(c context.Context, r *connect.Request[wf.AppAddRequest]) (*connect.Response[wf.App], error) {
	return s.add(c, r)
}
func (s *appStub) Get(c context.Context, r *connect.Request[wf.AppGetRequest]) (*connect.Response[wf.App], error) {
	return s.get(c, r)
}
func (s *appStub) Apply(c context.Context, r *connect.Request[wf.AppApplyRequest]) (*connect.Response[wf.App], error) {
	return s.apply(c, r)
}

func withoutKubernetes(t *testing.T) {
	t.Helper()
	t.Setenv("KUBECONFIG", "/unavailable/kubeconfig")
	t.Setenv("KUBERNETES_SERVICE_HOST", "127.0.0.1")
	t.Setenv("KUBERNETES_SERVICE_PORT", "1")
	t.Setenv("PATH", t.TempDir())
}
func execute(t *testing.T, endpoint string, args ...string) {
	t.Helper()
	command := Command()
	command.SetArgs(append([]string{"--endpoint", endpoint, "--tenant", "@demo"}, args...))
	if e := command.Execute(); e != nil {
		t.Fatal(e)
	}
}
func write(t *testing.T, path, contents string) {
	t.Helper()
	if e := os.WriteFile(path, []byte(contents), 0600); e != nil {
		t.Fatal(e)
	}
}
func testAppID() []byte { return []byte{0, 0, 0, 0, 0, 0, 0x80, 0, 0x80, 10, 0, 0, 0, 0, 0, 1} }

func TestAddUploadsFilesThroughControllerWithoutKubernetes(t *testing.T) {
	withoutKubernetes(t)
	directory := t.TempDir()
	bundle := "bundle bytes"
	sum := fmt.Sprintf("%x", sha256.Sum256([]byte(bundle)))
	write(t, filepath.Join(directory, "app.tar.gz"), bundle)
	write(t, filepath.Join(directory, "token"), "private-value\n")
	file := filepath.Join(directory, "app.yaml")
	write(t, file, "alias: example\ndomain: example.test\nmanifest:\n  bundle:\n    url: file:app.tar.gz\n  env:\n    memory_bytes: opaque\nsecrets:\n- env: TOKEN\n  file: token\n")
	added := make(chan *connect.Request[wf.AppAddRequest], 1)
	uploaded := make(chan []byte, 1)
	stub := &appStub{add: func(_ context.Context, r *connect.Request[wf.AppAddRequest]) (*connect.Response[wf.App], error) {
		added <- r
		return connect.NewResponse(&wf.App{Id: testAppID(), Alias: r.Msg.GetAlias()}), nil
	}}
	mux := http.NewServeMux()
	mux.Handle(waflconnect.NewAppServiceHandler(stub))
	mux.HandleFunc("PUT /bundles/{digest}", func(w http.ResponseWriter, r *http.Request) {
		if r.PathValue("digest") != sum || r.Header.Get("wafl-tenant") != "@demo" {
			http.Error(w, "wrong upload", 400)
			return
		}
		data, _ := io.ReadAll(r.Body)
		uploaded <- data
		w.Header().Set("Content-Type", "application/json")
		w.WriteHeader(201)
		fmt.Fprintf(w, `{"url":"http://assets.test/%s.tar.gz","digest":"sha256:%s"}`, sum, sum)
	})
	server := httptest.NewServer(mux)
	defer server.Close()
	execute(t, server.URL, "app", "add", file)
	request := <-added
	if request.Header().Get("wafl-tenant") != "demo" || request.Msg.Replicas != 1 || request.Msg.SecretValues["TOKEN"] != "private-value\n" || len(request.Msg.Secrets) != 0 || request.Msg.Manifest.Env["memory_bytes"] != "opaque" {
		t.Fatal("invalid App API request")
	}
	if request.Msg.Manifest.Bundle.GetUrl() != "http://assets.test/"+sum+".tar.gz" || request.Msg.Manifest.Bundle.Digest != "sha256:"+sum || string(<-uploaded) != bundle {
		t.Fatal("local bundle was not uploaded through controller")
	}
}
func TestPatchPreservesZerosAndOpaqueKeysAndFencesIdentity(t *testing.T) {
	withoutKubernetes(t)
	directory := t.TempDir()
	file := filepath.Join(directory, "patch.yaml")
	write(t, filepath.Join(directory, "new-token"), "next-value")
	write(t, file, "replicas: 0\nmanifest:\n  env:\n    old: null\n    memory_bytes: '0042'\nrollout:\n  surge: 0\nsecrets:\n- env: TOKEN\n  file: new-token\nreadiness: null\n")
	current := &wf.App{Id: testAppID(), Alias: "example", Revision: 8, Replicas: 3, Manifest: &wf.Manifest{Env: map[string]string{"old": "remove", "keep": "present"}}, Readiness: &wf.Probe{Kind: &wf.Probe_Http{Http: "/ready"}}, Secrets: []*wf.Secret{{Env: "OLD", Source: &wf.Secret_SecretKeyRef{SecretKeyRef: &wf.SecretKeyRef{Name: "source", Key: "value"}}}}}
	applied := make(chan *connect.Request[wf.AppApplyRequest], 1)
	nextValue := make(chan *wf.App, 1)
	stub := &appStub{
		get: func(_ context.Context, r *connect.Request[wf.AppGetRequest]) (*connect.Response[wf.App], error) {
			if r.Msg.Ref.GetAlias() != "example" {
				return nil, connect.NewError(connect.CodeNotFound, fmt.Errorf("wrong ref"))
			}
			return connect.NewResponse(proto.Clone(current).(*wf.App)), nil
		},
		apply: func(_ context.Context, r *connect.Request[wf.AppApplyRequest]) (*connect.Response[wf.App], error) {
			next, e := patchproto.Apply(current, r.Msg.Patch)
			if e != nil {
				return nil, e
			}
			applied <- r
			nextValue <- next
			out := proto.Clone(next).(*wf.App)
			out.SecretValues = nil
			return connect.NewResponse(out), nil
		},
	}
	mux := http.NewServeMux()
	mux.Handle(waflconnect.NewAppServiceHandler(stub))
	server := httptest.NewServer(mux)
	defer server.Close()
	execute(t, server.URL, "app", "patch", "@example", "--file", file)
	request := <-applied
	next := <-nextValue
	if !bytes.Equal(request.Msg.Ref.GetId(), current.Id) || request.Header().Get("wafl-revision") != "8" {
		t.Fatal("update did not bind both identity and revision")
	}
	if next.Replicas != 0 || next.Rollout.Surge != 0 || next.Readiness != nil || len(next.Secrets) != 0 || next.SecretValues["TOKEN"] != "next-value" {
		t.Fatal("zero, null, or Secret file conversion was lost")
	}
	if _, exists := next.Manifest.Env["old"]; exists || next.Manifest.Env["keep"] != "present" || next.Manifest.Env["memory_bytes"] != "0042" {
		t.Fatal("merge corrupted environment map")
	}
}
