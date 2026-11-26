package platform

import (
	"bytes"
	"context"
	"crypto/rand"
	"crypto/sha256"
	"encoding/base64"
	"encoding/hex"
	"encoding/json"
	"errors"
	"io"
	"net/http"
	"net/url"
	"regexp"
	"slices"
	"strconv"
	"strings"
	"time"
	"unicode/utf8"

	"connectrpc.com/connect"
	"github.com/lesomnus/protobuf-patch/patch"
	"github.com/lesomnus/protobuf-patch/patchproto"
	wf "github.com/lesomnus/wafl/kubernetes/gen/wafl"
	"github.com/lesomnus/wafl/kubernetes/gen/wafl/waflconnect"
	"google.golang.org/protobuf/encoding/protojson"
	"google.golang.org/protobuf/reflect/protoreflect"
	core "k8s.io/api/core/v1"
	apierrors "k8s.io/apimachinery/pkg/api/errors"
	meta "k8s.io/apimachinery/pkg/apis/meta/v1"
	"k8s.io/apimachinery/pkg/apis/meta/v1/unstructured"
	"k8s.io/apimachinery/pkg/util/validation"
)

// AppAPI translates the generated platform contract into Kubernetes storage.
// No Kubernetes object or API credential crosses this client-facing boundary.
type AppAPI struct{ Clients *Clients }

var appReadOnly = patch.MustNewReadOnly((&wf.App{}).ProtoReflect().Descriptor(), "id", "tenant", "alias", "domain", "revision", "state", "source", "status", "listeners")
var secretEnv = regexp.MustCompile(`^[A-Za-z_][A-Za-z0-9_]{0,127}$`)

func invalidApp(message string) error {
	return connect.NewError(connect.CodeInvalidArgument, errors.New(message))
}
func appError(e error) error {
	if e == nil {
		return nil
	}
	var rpc *connect.Error
	if errors.As(e, &rpc) {
		return e
	}
	code, message := connect.CodeUnavailable, "App storage is unavailable"
	switch {
	case apierrors.IsNotFound(e):
		code, message = connect.CodeNotFound, "App or tenant not found"
	case apierrors.IsAlreadyExists(e):
		code, message = connect.CodeAlreadyExists, "App alias already exists"
	case apierrors.IsConflict(e):
		code, message = connect.CodeAborted, "App changed; read it again before retrying"
	case apierrors.IsInvalid(e), apierrors.IsBadRequest(e):
		code, message = connect.CodeInvalidArgument, "invalid App declaration"
	case apierrors.IsForbidden(e):
		code, message = connect.CodePermissionDenied, "controller cannot access this tenant"
	}
	// Do not return Kubernetes request objects, Secret contents, or input values.
	return connect.NewError(code, errors.New(message))
}
func (a *AppAPI) tenant(ctx context.Context, header http.Header) (string, error) {
	name := header.Get("wafl-tenant")
	if name == "" {
		name = "default"
	}
	name = strings.TrimPrefix(name, "@")
	if id, err := hex.DecodeString(strings.ReplaceAll(name, "-", "")); err == nil && validID(id, 1) {
		list, e := a.Clients.Kube.CoreV1().Namespaces().List(ctx, meta.ListOptions{})
		if e != nil {
			return "", appError(e)
		}
		for _, ns := range list.Items {
			if ns.DeletionTimestamp == nil && bytes.Equal(id, ID(1, ns.Name)) {
				return ns.Name, nil
			}
		}
		return "", connect.NewError(connect.CodeNotFound, errors.New("tenant not found"))
	}
	if len(validation.IsDNS1123Label(name)) != 0 {
		return "", invalidApp("invalid tenant alias")
	}
	ns, e := a.Clients.Kube.CoreV1().Namespaces().Get(ctx, name, meta.GetOptions{})
	if e != nil {
		return "", appError(e)
	}
	if ns.DeletionTimestamp != nil {
		return "", connect.NewError(connect.CodeFailedPrecondition, errors.New("tenant is being removed"))
	}
	return name, nil
}
func appMatch(ref *wf.AppRef, value *unstructured.Unstructured) bool {
	if ref == nil {
		return false
	}
	switch key := ref.Key.(type) {
	case *wf.AppRef_Id:
		return bytes.Equal(key.Id, ID(10, string(value.GetUID())))
	case *wf.AppRef_Alias:
		return key.Alias == value.GetName()
	case *wf.AppRef_Domain:
		domain, _, _ := unstructured.NestedString(value.Object, "spec", "domain")
		return key.Domain == domain
	}
	return false
}
func validAppRef(ref *wf.AppRef) bool {
	if ref == nil {
		return false
	}
	switch key := ref.Key.(type) {
	case *wf.AppRef_Id:
		return validID(key.Id, 10)
	case *wf.AppRef_Alias:
		return len(validation.IsDNS1123Subdomain(key.Alias)) == 0
	case *wf.AppRef_Domain:
		return key.Domain != "" && len(key.Domain) <= 253
	}
	return false
}
func (a *AppAPI) get(ctx context.Context, ns string, ref *wf.AppRef) (*unstructured.Unstructured, error) {
	if !validAppRef(ref) {
		return nil, invalidApp("invalid App reference")
	}
	if alias, ok := ref.Key.(*wf.AppRef_Alias); ok {
		v, e := a.Clients.Dynamic.Resource(Apps).Namespace(ns).Get(ctx, alias.Alias, meta.GetOptions{})
		return v, appError(e)
	}
	list, e := a.Clients.Dynamic.Resource(Apps).Namespace(ns).List(ctx, meta.ListOptions{})
	if e != nil {
		return nil, appError(e)
	}
	for i := range list.Items {
		if appMatch(ref, &list.Items[i]) {
			return &list.Items[i], nil
		}
	}
	return nil, connect.NewError(connect.CodeNotFound, errors.New("App not found"))
}
func appValue(object *unstructured.Unstructured) (*wf.App, error) {
	v, e := decodeApp(object)
	if e != nil {
		return nil, connect.NewError(connect.CodeFailedPrecondition, errors.New("stored App declaration is invalid"))
	}
	v.Id = ID(10, string(object.GetUID()))
	v.Alias = object.GetName()
	v.Tenant = &wf.Tenant{Id: ID(1, object.GetNamespace()), Alias: object.GetNamespace()}
	v.Revision = uint64(object.GetGeneration())
	v.SecretValues = nil
	v.Source = nil
	v.Status = &wf.AppStatus{}
	desired, _, _ := unstructured.NestedInt64(object.Object, "status", "desired")
	readyCount, _, _ := unstructured.NestedInt64(object.Object, "status", "ready")
	v.State = &wf.ScaleState{Desired: uint32(max(0, desired))}
	v.Status.Ready = uint32(max(0, readyCount))
	conditions, _, _ := unstructured.NestedSlice(object.Object, "status", "conditions")
	for _, entry := range conditions {
		condition, ok := entry.(map[string]any)
		if !ok {
			continue
		}
		if condition["type"] == "Accepted" {
			v.Status.Accepted = condition["status"] == "True"
		}
		if condition["type"] == "Ready" {
			v.Status.Reason, _ = condition["reason"].(string)
			v.Status.Message, _ = condition["message"].(string)
		}
	}
	return v, nil
}
func appSpec(v *wf.App) (map[string]any, error) {
	// Scalar presence is implicit in the entity contract. Explicitly retain zero
	// replicas and zero surge when mapping to the CRD's defaulted JSON fields.
	source := map[string]any{"replicas": int64(v.Replicas)}
	if v.Rollout != nil {
		source["rollout"] = map[string]any{"surge": int64(v.Rollout.Surge)}
	}
	out, e := kubernetesJSON(v.ProtoReflect(), source)
	if e != nil {
		return nil, e
	}
	for key := range out {
		switch key {
		case "name", "domain", "manifest", "replicas", "scaling", "secrets", "readiness", "rollout", "listeners":
		default:
			delete(out, key)
		}
	}
	return out, nil
}
func checkValues(v *wf.App) error {
	if len(v.Secrets) > 64 || len(v.SecretValues) > 64 {
		return invalidApp("at most 64 Secret bindings are supported")
	}
	seen := map[string]bool{}
	for _, s := range v.Secrets {
		if s == nil || !secretEnv.MatchString(s.Env) || s.Env == "PORT" || seen[s.Env] {
			return invalidApp("invalid or repeated Secret environment name")
		}
		seen[s.Env] = true
		if _, exists := v.Manifest.Env[s.Env]; exists {
			return invalidApp("Secret environment conflicts with manifest.env")
		}
		ref := s.GetSecretKeyRef()
		if ref == nil || len(validation.IsDNS1123Subdomain(ref.Name)) != 0 || len(validation.IsConfigMapKey(ref.Key)) != 0 {
			return invalidApp("Secret sources must be references; upload local file values through secret_values")
		}
	}
	total := 0
	for name, value := range v.SecretValues {
		total += len(name) + len(value)
		if !secretEnv.MatchString(name) || name == "PORT" || len(value) > 16384 || !utf8.ValidString(value) || strings.ContainsRune(value, 0) {
			return invalidApp("invalid Secret environment value")
		}
		if _, exists := v.Manifest.Env[name]; exists {
			return invalidApp("Secret environment conflicts with manifest.env")
		}
		seen[name] = true
	}
	if total > 65536 {
		return invalidApp("Secret values exceed 64 KiB")
	}
	if len(seen) > 64 {
		return invalidApp("at most 64 Secret bindings are supported")
	}
	return nil
}
func pinRemote(ctx context.Context, m *wf.Manifest) error {
	if m == nil || m.Bundle == nil {
		return invalidApp("manifest.bundle is required")
	}
	if m.Bundle.Digest != "" {
		return nil
	}
	u, e := url.Parse(m.Bundle.GetUrl())
	if e != nil || u.Scheme != "http" && u.Scheme != "https" {
		return invalidApp("this bundle URL requires an explicit digest; upload local files first")
	}
	r, e := http.NewRequestWithContext(ctx, http.MethodGet, u.String(), nil)
	if e != nil {
		return invalidApp("invalid bundle URL")
	}
	response, e := (&http.Client{Timeout: 2 * time.Minute}).Do(r)
	if e != nil {
		return connect.NewError(connect.CodeUnavailable, errors.New("bundle download failed"))
	}
	defer response.Body.Close()
	if response.StatusCode != 200 {
		return connect.NewError(connect.CodeFailedPrecondition, errors.New("bundle download failed"))
	}
	h := sha256.New()
	n, e := io.Copy(h, io.LimitReader(response.Body, (1<<30)+1))
	if e != nil {
		return connect.NewError(connect.CodeUnavailable, errors.New("bundle download failed"))
	}
	if n > 1<<30 {
		return connect.NewError(connect.CodeResourceExhausted, errors.New("bundle exceeds 1 GiB"))
	}
	m.Bundle.Digest = "sha256:" + hex.EncodeToString(h.Sum(nil))
	return nil
}
func validateAppValue(ctx context.Context, v *wf.App) error {
	if v.Manifest == nil {
		return invalidApp("manifest is required")
	}
	if e := checkValues(v); e != nil {
		return e
	}
	if e := pinRemote(ctx, v.Manifest); e != nil {
		return e
	}
	spec, e := appSpec(v)
	if e != nil {
		return invalidApp("invalid App declaration")
	}
	if _, e = decodeApp(&unstructured.Unstructured{Object: map[string]any{"spec": spec}}); e != nil {
		return invalidApp("invalid App declaration: " + e.Error())
	}
	return nil
}
func (a *AppAPI) importValues(ctx context.Context, ns string, v *wf.App) error {
	keys := make([]string, 0, len(v.SecretValues))
	for key := range v.SecretValues {
		keys = append(keys, key)
	}
	slices.Sort(keys)
	for _, env := range keys {
		nonce := make([]byte, 16)
		if _, e := rand.Read(nonce); e != nil {
			return connect.NewError(connect.CodeInternal, errors.New("cannot allocate Secret source"))
		}
		name := "wafl-value-" + hex.EncodeToString(nonce)
		// Immutable random names reveal neither contents nor their hashes. Existing
		// App/Pod references protect sources from the controller's 10-minute GC.
		secret := &core.Secret{ObjectMeta: meta.ObjectMeta{Name: name, Namespace: ns, Labels: map[string]string{Managed: "true", Kind: "snapshot"}}, Immutable: Ptr(true), Data: map[string][]byte{"value": []byte(v.SecretValues[env])}}
		if _, e := a.Clients.Kube.CoreV1().Secrets(ns).Create(ctx, secret, meta.CreateOptions{}); e != nil {
			return appError(e)
		}
		binding := &wf.Secret{Env: env, Source: &wf.Secret_SecretKeyRef{SecretKeyRef: &wf.SecretKeyRef{Name: name, Key: "value"}}}
		found := false
		for i, s := range v.Secrets {
			if s.Env == env {
				v.Secrets[i] = binding
				found = true
				break
			}
		}
		if !found {
			v.Secrets = append(v.Secrets, binding)
		}
	}
	v.SecretValues = nil
	return nil
}
func (a *AppAPI) Add(ctx context.Context, r *connect.Request[wf.AppAddRequest]) (*connect.Response[wf.App], error) {
	ns, e := a.tenant(ctx, r.Header())
	if e != nil {
		return nil, e
	}
	m := r.Msg
	if len(m.Id) != 0 || m.Revision != 0 || m.State != nil || m.Source != nil || m.Status != nil {
		return nil, invalidApp("App identity and observed fields are assigned by the controller")
	}
	if m.Tenant != nil {
		match := m.Tenant.GetAlias() == ns || bytes.Equal(m.Tenant.GetId(), ID(1, ns))
		if !match {
			return nil, invalidApp("tenant reference differs from wafl-tenant scope")
		}
	}
	alias := m.GetAlias()
	if alias == "" {
		alias = strings.ReplaceAll(m.Domain, ".", "-")
	}
	if len(validation.IsDNS1123Subdomain(alias)) != 0 {
		return nil, invalidApp("invalid App alias")
	}
	if _, e := a.Clients.Dynamic.Resource(Apps).Namespace(ns).Get(ctx, alias, meta.GetOptions{}); e == nil {
		return nil, connect.NewError(connect.CodeAlreadyExists, errors.New("App alias already exists"))
	} else if !apierrors.IsNotFound(e) {
		return nil, appError(e)
	}
	data, _ := protojson.Marshal(m)
	v := &wf.App{}
	if e = (protojson.UnmarshalOptions{DiscardUnknown: true}).Unmarshal(data, v); e != nil {
		return nil, invalidApp("invalid App declaration")
	}
	v.Tenant = nil
	v.Alias = alias
	if e = validateAppValue(ctx, v); e != nil {
		return nil, e
	}
	if e = a.importValues(ctx, ns, v); e != nil {
		return nil, e
	}
	spec, e := appSpec(v)
	if e != nil {
		return nil, invalidApp("invalid App declaration")
	}
	object := &unstructured.Unstructured{Object: map[string]any{"apiVersion": Group + "/v1alpha1", "kind": "App", "metadata": map[string]any{"name": alias, "namespace": ns}, "spec": spec}}
	created, e := a.Clients.Dynamic.Resource(Apps).Namespace(ns).Create(ctx, object, meta.CreateOptions{})
	if e != nil {
		return nil, appError(e)
	}
	value, e := appValue(created)
	if e != nil {
		return nil, e
	}
	return connect.NewResponse(value), nil
}
func (a *AppAPI) Get(ctx context.Context, r *connect.Request[wf.AppGetRequest]) (*connect.Response[wf.App], error) {
	ns, e := a.tenant(ctx, r.Header())
	if e != nil {
		return nil, e
	}
	object, e := a.get(ctx, ns, r.Msg.Ref)
	if e != nil {
		return nil, e
	}
	value, e := appValue(object)
	if e != nil {
		return nil, e
	}
	if r.Msg.Select != nil {
		selectFields(value.ProtoReflect(), r.Msg.Select.ProtoReflect())
	}
	return connect.NewResponse(value), nil
}
func revision(header http.Header, object *unstructured.Unstructured) error {
	value := header.Get("wafl-revision")
	if value == "" {
		return nil
	}
	n, e := strconv.ParseUint(value, 10, 64)
	if e != nil {
		return invalidApp("invalid wafl-revision")
	}
	if n != uint64(object.GetGeneration()) {
		return connect.NewError(connect.CodeAborted, errors.New("App changed; read it again before retrying"))
	}
	return nil
}
func (a *AppAPI) change(ctx context.Context, header http.Header, ref *wf.AppRef, fn func(*wf.App) (*wf.App, error)) (*connect.Response[wf.App], error) {
	ns, e := a.tenant(ctx, header)
	if e != nil {
		return nil, e
	}
	object, e := a.get(ctx, ns, ref)
	if e != nil {
		return nil, e
	}
	if object.GetDeletionTimestamp() != nil {
		return nil, connect.NewError(connect.CodeFailedPrecondition, errors.New("App is being removed"))
	}
	if e = revision(header, object); e != nil {
		return nil, e
	}
	before, e := appValue(object)
	if e != nil {
		return nil, e
	}
	next, e := fn(before)
	if e != nil {
		return nil, e
	}
	if e = validateAppValue(ctx, next); e != nil {
		return nil, e
	}
	if e = a.importValues(ctx, ns, next); e != nil {
		return nil, e
	}
	spec, e := appSpec(next)
	if e != nil {
		return nil, invalidApp("invalid App declaration")
	}
	object.Object["spec"] = spec
	// resourceVersion is a CAS for the complete patch, including test operations.
	// Uncertain writes never delete newly created sources; unreferenced sources
	// are collected later, so an acknowledged or committed App cannot lose them.
	updated, e := a.Clients.Dynamic.Resource(Apps).Namespace(ns).Update(ctx, object, meta.UpdateOptions{})
	if e != nil {
		return nil, appError(e)
	}
	value, e := appValue(updated)
	if e != nil {
		return nil, e
	}
	return connect.NewResponse(value), nil
}
func (a *AppAPI) Patch(ctx context.Context, r *connect.Request[wf.AppPatchRequest]) (*connect.Response[wf.App], error) {
	return a.change(ctx, r.Header(), r.Msg.Ref, func(v *wf.App) (*wf.App, error) {
		m := r.Msg
		fields := m.ProtoReflect().Descriptor().Fields()
		target := v.ProtoReflect()
		var err error
		m.ProtoReflect().Range(func(f protoreflect.FieldDescriptor, value protoreflect.Value) bool {
			name := string(f.Name())
			if name == "ref" {
				return true
			}
			switch name {
			case "alias", "revision", "state", "state_null", "status", "status_null":
				err = invalidApp("App identity and observed fields are read-only")
				return false
			}
			if strings.HasSuffix(name, "_null") {
				if value.Bool() {
					field := target.Descriptor().Fields().ByName(protoreflect.Name(strings.TrimSuffix(name, "_null")))
					if field != nil {
						target.Clear(field)
					}
				}
				return true
			}
			if clear := fields.ByName(protoreflect.Name(name + "_null")); clear != nil && m.ProtoReflect().Get(clear).Bool() {
				return true
			}
			field := target.Descriptor().Fields().ByName(f.Name())
			if field == nil {
				err = invalidApp("unsupported App field")
				return false
			}
			target.Set(field, value)
			return true
		})
		return v, err
	})
}
func (a *AppAPI) Apply(ctx context.Context, r *connect.Request[wf.AppApplyRequest]) (*connect.Response[wf.App], error) {
	if e := appReadOnly.Check(r.Msg.Patch); e != nil {
		return nil, invalidApp("patch targets a read-only field or has an invalid document")
	}
	return a.change(ctx, r.Header(), r.Msg.Ref, func(v *wf.App) (*wf.App, error) {
		next, e := patchproto.Apply(v, r.Msg.Patch)
		if e != nil {
			return nil, connect.NewError(connect.CodeFailedPrecondition, errors.New("App patch could not be applied"))
		}
		return next, nil
	})
}
func (a *AppAPI) Erase(ctx context.Context, r *connect.Request[wf.AppRef]) (*connect.Response[wf.AppEraseResponse], error) {
	ns, e := a.tenant(ctx, r.Header())
	if e != nil {
		return nil, e
	}
	object, e := a.get(ctx, ns, r.Msg)
	if connect.CodeOf(e) == connect.CodeNotFound {
		return connect.NewResponse(&wf.AppEraseResponse{Erased: Ptr(false)}), nil
	}
	if e != nil {
		return nil, e
	}
	if e = revision(r.Header(), object); e != nil {
		return nil, e
	}
	e = a.Clients.Dynamic.Resource(Apps).Namespace(ns).Delete(ctx, object.GetName(), meta.DeleteOptions{PropagationPolicy: Ptr(meta.DeletePropagationForeground), Preconditions: &meta.Preconditions{UID: Ptr(object.GetUID()), ResourceVersion: Ptr(object.GetResourceVersion())}})
	if e != nil {
		return nil, appError(e)
	}
	return connect.NewResponse(&wf.AppEraseResponse{Erased: Ptr(true)}), nil
}
func (a *AppAPI) List(ctx context.Context, r *connect.Request[wf.AppListRequest]) (*connect.Response[wf.AppListResponse], error) {
	ns, e := a.tenant(ctx, r.Header())
	if e != nil {
		return nil, e
	}
	m := r.Msg
	if len(m.Filters) > 64 {
		return nil, invalidApp("too many App filters")
	}
	for _, f := range m.Filters {
		if f == nil || !validAppRef(f.Ref) {
			return nil, invalidApp("invalid App filter")
		}
	}
	scope := digest([]any{ns, m.Filters})
	var cursor trafficCursor
	if m.After != "" {
		b, e := base64.RawURLEncoding.DecodeString(m.After)
		if len(m.After) > 512 || e != nil || json.Unmarshal(b, &cursor) != nil || cursor.Scope != scope || cursor.Reverse != m.Reverse || !validID(cursor.ID, 10) {
			return nil, invalidApp("invalid App cursor")
		}
	}
	list, e := a.Clients.Dynamic.Resource(Apps).Namespace(ns).List(ctx, meta.ListOptions{})
	if e != nil {
		return nil, appError(e)
	}
	items := []*wf.App{}
	for i := range list.Items {
		object := &list.Items[i]
		matches := len(m.Filters) == 0
		for _, f := range m.Filters {
			if appMatch(f.Ref, object) {
				matches = true
				break
			}
		}
		if !matches {
			continue
		}
		v, e := appValue(object)
		if e != nil {
			return nil, e
		}
		if m.After != "" {
			cmp := bytes.Compare(v.Id, cursor.ID)
			if !m.Reverse && cmp <= 0 || m.Reverse && cmp >= 0 {
				continue
			}
		}
		items = append(items, v)
	}
	slices.SortFunc(items, func(x, y *wf.App) int {
		n := bytes.Compare(x.Id, y.Id)
		if m.Reverse {
			return -n
		}
		return n
	})
	size := int(m.Size)
	if size <= 0 {
		size = 100
	}
	size = min(size, 1000)
	out := &wf.AppListResponse{}
	if len(items) > size {
		b, _ := json.Marshal(trafficCursor{ID: items[size-1].Id, Reverse: m.Reverse, Scope: scope})
		out.Next = base64.RawURLEncoding.EncodeToString(b)
		items = items[:size]
	}
	out.Items = items
	return connect.NewResponse(out), nil
}
func (a *AppAPI) Tail(ctx context.Context, r *connect.Request[wf.AppTailRequest], out *connect.ServerStream[wf.Log]) error {
	ns, e := a.tenant(ctx, r.Header())
	if e != nil {
		return e
	}
	object, e := a.get(ctx, ns, r.Msg.Ref)
	if e != nil {
		return e
	}
	if r.Msg.Cursor != 0 && r.Msg.Instance == nil {
		return invalidApp("resuming logs requires an explicit instance")
	}
	if r.Msg.Instance != nil && !validID(r.Msg.Instance.GetId(), 7) {
		return invalidApp("instance must be a domain-7 ID")
	}
	pods, e := a.Clients.Kube.CoreV1().Pods(ns).List(ctx, meta.ListOptions{LabelSelector: AppUID + "=" + string(object.GetUID())})
	if e != nil {
		return appError(e)
	}
	slices.SortFunc(pods.Items, func(x, y core.Pod) int {
		if ready(&x) != ready(&y) {
			if ready(&x) {
				return -1
			}
			return 1
		}
		return strings.Compare(x.Name, y.Name)
	})
	for _, p := range pods.Items {
		id := podID(&p)
		if r.Msg.Instance != nil && !bytes.Equal(id, r.Msg.Instance.GetId()) {
			continue
		}
		endpoint := p.Annotations[Runner]
		if endpoint == "" {
			continue
		}
		stream, e := NewRunner(endpoint).Instances.Tail(ctx, connect.NewRequest(&wf.InstanceTailRequest{Ref: instanceRef(id), Follow: r.Msg.Follow, Cursor: r.Msg.Cursor, Limit: r.Msg.Limit}))
		if e != nil {
			return e
		}
		defer stream.Close()
		out.ResponseHeader().Set("wafl-instance", UUID(id))
		for stream.Receive() {
			if e = out.Send(stream.Msg()); e != nil {
				return e
			}
		}
		return stream.Err()
	}
	return connect.NewError(connect.CodeFailedPrecondition, errors.New("App has no execution with available logs"))
}

var _ waflconnect.AppServiceHandler = (*AppAPI)(nil)
