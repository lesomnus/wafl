package platform

import (
	"bytes"
	"context"
	"crypto/sha256"
	"encoding/json"
	"errors"
	"fmt"
	"io"
	"net/http"
	"net/http/httptest"
	"strings"
	"testing"
	"time"

	"connectrpc.com/connect"
	"github.com/lesomnus/protobuf-patch/patch"
	wf "github.com/lesomnus/wafl/kubernetes/gen/wafl"
	"github.com/lesomnus/wafl/kubernetes/gen/wafl/waflconnect"
	core "k8s.io/api/core/v1"
	apierrors "k8s.io/apimachinery/pkg/api/errors"
	meta "k8s.io/apimachinery/pkg/apis/meta/v1"
	"k8s.io/apimachinery/pkg/apis/meta/v1/unstructured"
	"k8s.io/apimachinery/pkg/runtime"
	"k8s.io/apimachinery/pkg/runtime/schema"
	"k8s.io/apimachinery/pkg/types"
	dynamicfake "k8s.io/client-go/dynamic/fake"
	"k8s.io/client-go/kubernetes/fake"
	ktesting "k8s.io/client-go/testing"
)

func apiFixture(t *testing.T) (*Clients, waflconnect.AppServiceClient) {
	t.Helper()
	k := fake.NewClientset(&core.Namespace{ObjectMeta: meta.ObjectMeta{Name: "demo"}}, &core.Namespace{ObjectMeta: meta.ObjectMeta{Name: "other"}})
	d := dynamicfake.NewSimpleDynamicClientWithCustomListKinds(runtime.NewScheme(), map[schema.GroupVersionResource]string{Apps: "AppList"})
	d.PrependReactor("create", "apps", func(action ktesting.Action) (bool, runtime.Object, error) {
		object := action.(ktesting.CreateAction).GetObject().(*unstructured.Unstructured)
		object.SetUID(types.UID("uid-" + object.GetNamespace() + "-" + object.GetName()))
		object.SetResourceVersion("1")
		object.SetGeneration(1)
		return false, nil, nil
	})
	d.PrependReactor("update", "apps", func(action ktesting.Action) (bool, runtime.Object, error) {
		object := action.(ktesting.UpdateAction).GetObject().(*unstructured.Unstructured)
		stored, e := d.Tracker().Get(Apps, object.GetNamespace(), object.GetName())
		if e != nil {
			return true, nil, e
		}
		old := stored.(*unstructured.Unstructured)
		if old.GetResourceVersion() != object.GetResourceVersion() {
			return true, nil, apierrors.NewConflict(Apps.GroupResource(), object.GetName(), errors.New("CAS conflict"))
		}
		object.SetGeneration(old.GetGeneration() + 1)
		object.SetResourceVersion(fmt.Sprint(object.GetGeneration()))
		return false, nil, nil
	})
	c := &Clients{Kube: k, Dynamic: d}
	mux := http.NewServeMux()
	mux.Handle(waflconnect.NewAppServiceHandler(&AppAPI{Clients: c}))
	server := httptest.NewServer(mux)
	t.Cleanup(server.Close)
	return c, waflconnect.NewAppServiceClient(server.Client(), server.URL, connect.WithProtoJSON())
}
func appRequest[T any](msg *T) *connect.Request[T] {
	r := connect.NewRequest(msg)
	r.Header().Set("wafl-tenant", "demo")
	return r
}
func addApp(t *testing.T, client waflconnect.AppServiceClient, alias string) *wf.App {
	t.Helper()
	_, spec := fixture()
	r, e := client.Add(context.Background(), appRequest(&wf.AppAddRequest{Alias: Ptr(alias), Domain: alias + ".example", Replicas: 1, Manifest: spec.Manifest, SecretValues: map[string]string{"PASSWORD": "canary-api-value"}}))
	if e != nil {
		t.Fatal(e)
	}
	return r.Msg
}
func TestAppAPIStoresOnlyReferencesAndFencesUpdates(t *testing.T) {
	c, client := apiFixture(t)
	ctx := context.Background()
	v := addApp(t, client, "example")
	if !validID(v.Id, 10) || v.Revision != 1 || v.Tenant.Alias != "demo" || len(v.Secrets) != 1 || len(v.SecretValues) != 0 {
		t.Fatal("invalid public App response")
	}
	stored, e := c.Dynamic.Resource(Apps).Namespace("demo").Get(ctx, "example", meta.GetOptions{})
	if e != nil {
		t.Fatal(e)
	}
	data, _ := json.Marshal(stored)
	if bytes.Contains(data, []byte("canary-api-value")) {
		t.Fatal("Secret escaped into App storage")
	}
	first := v.Secrets[0].GetSecretKeyRef().Name
	secret, e := c.Kube.CoreV1().Secrets("demo").Get(ctx, first, meta.GetOptions{})
	if e != nil || string(secret.Data["value"]) != "canary-api-value" || secret.Immutable == nil || !*secret.Immutable {
		t.Fatal("Secret source was not preserved")
	}
	ref := &wf.AppRef{Key: &wf.AppRef_Id{Id: v.Id}}
	update := appRequest(&wf.AppPatchRequest{Ref: ref, SecretValues: map[string]string{"PASSWORD": "canary-api-new"}})
	update.Header().Set("wafl-revision", "1")
	next, e := client.Patch(ctx, update)
	if e != nil {
		t.Fatal(e)
	}
	if next.Msg.Revision != 2 || next.Msg.Secrets[0].GetSecretKeyRef().Name == first || len(next.Msg.SecretValues) != 0 {
		t.Fatal("rotation did not create a private immutable source")
	}
	if _, e = client.Patch(ctx, update); connect.CodeOf(e) != connect.CodeAborted {
		t.Fatalf("stale update: %v", e)
	}
	sources, _ := c.Kube.CoreV1().Secrets("demo").List(ctx, meta.ListOptions{})
	if len(sources.Items) != 2 {
		t.Fatal("stale write created a Secret")
	}
	read, e := client.Get(ctx, appRequest(&wf.AppGetRequest{Ref: ref, Select: &wf.AppSelect{SecretValues: Ptr(true)}}))
	if e != nil || len(read.Msg.SecretValues) != 0 || len(read.Msg.Id) == 0 {
		t.Fatal("selection leaked Secret contents")
	}
	foreign := connect.NewRequest(&wf.AppGetRequest{Ref: ref})
	foreign.Header().Set("wafl-tenant", "other")
	if _, e = client.Get(ctx, foreign); connect.CodeOf(e) != connect.CodeNotFound {
		t.Fatal("App escaped tenant scope")
	}
	bad := appRequest(&wf.AppApplyRequest{Ref: ref, Patch: patch.MustNew("wafl.App", patch.Target(patch.Name("alias")).Assign(patch.Str("other")))})
	if _, e = client.Apply(ctx, bad); connect.CodeOf(e) != connect.CodeInvalidArgument {
		t.Fatal("immutable alias changed")
	}
	// One complete CAS also covers tests and whole-list removal.
	p := patch.MustNew("wafl.App", patch.Target(patch.Name("revision")).Test(patch.Uint64(2)), patch.Target(patch.Name("secrets")).Assign(patch.List()), patch.Target(patch.Name("replicas")).Assign(patch.Uint32(0)))
	cleared, e := client.Apply(ctx, appRequest(&wf.AppApplyRequest{Ref: ref, Patch: p}))
	if e != nil {
		t.Fatal(e)
	}
	if len(cleared.Msg.Secrets) != 0 || cleared.Msg.Replicas != 0 {
		t.Fatal("explicit zero/list clear was lost")
	}
	if _, e = client.Apply(ctx, appRequest(&wf.AppApplyRequest{Ref: ref, Patch: p})); connect.CodeOf(e) != connect.CodeFailedPrecondition {
		t.Fatal("failed patch test was ignored")
	}
	erased, e := client.Erase(ctx, appRequest(ref))
	if e != nil || !erased.Msg.GetErased() {
		t.Fatal(e)
	}
	erased, e = client.Erase(ctx, appRequest(ref))
	if e != nil || erased.Msg.GetErased() {
		t.Fatal("erase is not idempotent")
	}
}
func TestAppAPIListScopesCursorAndRejectsServerFilePaths(t *testing.T) {
	_, client := apiFixture(t)
	ctx := context.Background()
	one := addApp(t, client, "one")
	addApp(t, client, "two")
	page, e := client.List(ctx, appRequest(&wf.AppListRequest{Size: 1}))
	if e != nil || len(page.Msg.Items) != 1 || page.Msg.Next == "" {
		t.Fatal("missing first page")
	}
	next, e := client.List(ctx, appRequest(&wf.AppListRequest{Size: 1, After: page.Msg.Next}))
	if e != nil || len(next.Msg.Items) != 1 || bytes.Equal(next.Msg.Items[0].Id, page.Msg.Items[0].Id) {
		t.Fatal("bad next page")
	}
	foreign := connect.NewRequest(&wf.AppListRequest{Size: 1, After: page.Msg.Next})
	foreign.Header().Set("wafl-tenant", "other")
	if _, e = client.List(ctx, foreign); connect.CodeOf(e) != connect.CodeInvalidArgument {
		t.Fatal("cursor escaped tenant scope")
	}
	only, e := client.List(ctx, appRequest(&wf.AppListRequest{Filters: []*wf.AppFilter{{Ref: &wf.AppRef{Key: &wf.AppRef_Domain{Domain: one.Domain}}}}}))
	if e != nil || len(only.Msg.Items) != 1 || !bytes.Equal(only.Msg.Items[0].Id, one.Id) {
		t.Fatal("domain filter failed")
	}
	_, spec := fixture()
	_, e = client.Add(ctx, appRequest(&wf.AppAddRequest{Alias: Ptr("file"), Domain: "file.example", Replicas: 1, Manifest: spec.Manifest, Secrets: []*wf.Secret{{Env: "TOKEN", Source: &wf.Secret_File{File: "/server/private"}}}}))
	if connect.CodeOf(e) != connect.CodeInvalidArgument {
		t.Fatal("controller accepted a server-side file source")
	}
}

type tailRunner struct {
	waflconnect.UnimplementedInstanceServiceHandler
	cancelled chan struct{}
}

func (s *tailRunner) Tail(ctx context.Context, r *connect.Request[wf.InstanceTailRequest], out *connect.ServerStream[wf.Log]) error {
	if !r.Msg.Follow {
		return out.Send(&wf.Log{Sequence: 1, Data: []byte("guest log\n")})
	}
	if e := out.Send(&wf.Log{Sequence: 1, Data: []byte("guest log\n")}); e != nil {
		return e
	}
	<-ctx.Done()
	close(s.cancelled)
	return ctx.Err()
}
func TestAppTailRelaysRunnerStreamAndCancellation(t *testing.T) {
	c, client := apiFixture(t)
	app := addApp(t, client, "logs")
	ctx := context.Background()
	runner := &tailRunner{cancelled: make(chan struct{})}
	mux := http.NewServeMux()
	mux.Handle(waflconnect.NewInstanceServiceHandler(runner))
	server := httptest.NewServer(mux)
	defer server.Close()
	object, _ := c.Dynamic.Resource(Apps).Namespace("demo").Get(ctx, "logs", meta.GetOptions{})
	pod := &core.Pod{ObjectMeta: meta.ObjectMeta{Name: "execution", Namespace: "demo", UID: "pod-uid", Labels: map[string]string{AppUID: string(object.GetUID())}, Annotations: map[string]string{Runner: server.URL}}}
	if _, e := c.Kube.CoreV1().Pods("demo").Create(ctx, pod, meta.CreateOptions{}); e != nil {
		t.Fatal(e)
	}
	streamCtx, cancel := context.WithCancel(ctx)
	defer cancel()
	stream, e := client.Tail(streamCtx, appRequest(&wf.AppTailRequest{Ref: &wf.AppRef{Key: &wf.AppRef_Id{Id: app.Id}}, Follow: true}))
	if e != nil {
		t.Fatal(e)
	}
	if !stream.Receive() || string(stream.Msg().Data) != "guest log\n" || stream.ResponseHeader().Get("wafl-instance") != UUID(podID(pod)) {
		t.Fatal("log stream was not relayed")
	}
	cancel()
	_ = stream.Close()
	select {
	case <-runner.cancelled:
	case <-time.After(3 * time.Second):
		t.Fatal("client cancellation did not reach runner")
	}
}
func TestArtifactUploadChecksDigestBeforePublication(t *testing.T) {
	directory := t.TempDir()
	handler, e := AssetHandler(directory)
	if e != nil {
		t.Fatal(e)
	}
	store := httptest.NewServer(handler)
	defer store.Close()
	controller := &Controller{Assets: store.URL}
	mux := http.NewServeMux()
	mux.HandleFunc("PUT /bundles/{digest}", controller.upload)
	server := httptest.NewServer(mux)
	defer server.Close()
	body := "test bundle"
	sum := fmt.Sprintf("%x", sha256Sum([]byte(body)))
	request, _ := http.NewRequest(http.MethodPut, server.URL+"/bundles/"+sum, strings.NewReader(body))
	response, e := http.DefaultClient.Do(request)
	if e != nil {
		t.Fatal(e)
	}
	io.Copy(io.Discard, response.Body)
	response.Body.Close()
	if response.StatusCode != 201 {
		t.Fatal("upload rejected")
	}
	request, _ = http.NewRequest(http.MethodPut, server.URL+"/bundles/"+sum, strings.NewReader("different"))
	response, e = http.DefaultClient.Do(request)
	if e != nil {
		t.Fatal(e)
	}
	response.Body.Close()
	if response.StatusCode != 400 {
		t.Fatal("digest mismatch accepted")
	}
	response, e = http.Get(store.URL + "/" + sum + ".tar.gz")
	if e != nil {
		t.Fatal(e)
	}
	data, _ := io.ReadAll(response.Body)
	response.Body.Close()
	if string(data) != body {
		t.Fatal("failed upload replaced published content")
	}
}
func sha256Sum(v []byte) []byte { h := sha256.New(); h.Write(v); return h.Sum(nil) }
