// Package platform implements the Kubernetes control plane. Runner RPCs and App
// fields come from the same protobuf descriptors as the standalone Rust runner.
package platform

import (
	"context"
	"crypto/sha256"
	"encoding/base64"
	"encoding/hex"
	"encoding/json"
	"errors"
	"fmt"
	"net"
	"net/http"
	"strings"
	"time"

	"connectrpc.com/connect"
	wf "github.com/lesomnus/wafl/kubernetes/gen/wafl"
	"github.com/lesomnus/wafl/kubernetes/gen/wafl/waflconnect"
	"google.golang.org/protobuf/encoding/protojson"
	"google.golang.org/protobuf/proto"
	core "k8s.io/api/core/v1"
	meta "k8s.io/apimachinery/pkg/apis/meta/v1"
	"k8s.io/apimachinery/pkg/apis/meta/v1/unstructured"
	"k8s.io/apimachinery/pkg/runtime/schema"
	"k8s.io/client-go/dynamic"
	"k8s.io/client-go/kubernetes"
	"k8s.io/client-go/rest"
	"k8s.io/client-go/tools/clientcmd"
)

const Group = "wafl.opt.td"
const Managed = Group + "/managed"
const AppUID = Group + "/app"
const Revision = Group + "/revision"
const Manifest = Group + "/manifest"
const Instance = Group + "/instance"
const Session = Group + "/session"
const Endpoint = Group + "/endpoint"
const Runner = Group + "/runner"
const RunnerPod = Group + "/runnerPod"
const Execution = Group + "/execution"
const Finalizer = Group + "/execution"
const Runtime = Group + "/runtime"
const Slots = core.ResourceName(Group + "/slots")
const Memory = core.ResourceName(Group + "/memory")

var Apps = schema.GroupVersionResource{Group: Group, Version: "v1alpha1", Resource: "apps"}

type Clients struct {
	Kube    kubernetes.Interface
	Dynamic dynamic.Interface
	Config  *rest.Config
}

func Open(kubeconfig, contextName string) (*Clients, error) {
	rules := clientcmd.NewDefaultClientConfigLoadingRules()
	if kubeconfig != "" {
		rules.ExplicitPath = kubeconfig
	}
	overrides := &clientcmd.ConfigOverrides{CurrentContext: contextName}
	config, err := clientcmd.NewNonInteractiveDeferredLoadingClientConfig(rules, overrides).ClientConfig()
	if err != nil && kubeconfig == "" && contextName == "" {
		config, err = rest.InClusterConfig()
	}
	if err != nil {
		return nil, err
	}
	config.QPS = 50
	config.Burst = 100
	config.Timeout = 15 * time.Second
	k, err := kubernetes.NewForConfig(config)
	if err != nil {
		return nil, err
	}
	d, err := dynamic.NewForConfig(config)
	return &Clients{Kube: k, Dynamic: d, Config: config}, err
}
func Ptr[T any](v T) *T { return &v }

func (c *Clients) activeApps(ctx context.Context) (*unstructured.UnstructuredList, error) {
	list, e := c.Dynamic.Resource(Apps).Namespace("").List(ctx, meta.ListOptions{})
	if e != nil {
		return nil, e
	}
	ns, e := c.Kube.CoreV1().Namespaces().List(ctx, meta.ListOptions{})
	if e != nil {
		return nil, e
	}
	active := map[string]bool{}
	for _, n := range ns.Items {
		active[n.Name] = n.DeletionTimestamp == nil
	}
	items := list.Items[:0]
	for _, a := range list.Items {
		if active[a.GetNamespace()] && a.GetDeletionTimestamp() == nil {
			items = append(items, a)
		}
	}
	list.Items = items
	orderApps(list.Items)
	return list, nil
}
func digest(v any) string {
	b, _ := json.Marshal(v)
	s := sha256.Sum256(b)
	return hex.EncodeToString(s[:])
}
func ID(domain byte, key string) []byte {
	s := sha256.Sum256([]byte(key))
	id := append([]byte(nil), s[:16]...)
	id[6] = (id[6] & 15) | 0x80
	id[8] = 0x80
	id[9] = domain
	return id
}
func UUID(id []byte) string {
	if len(id) != 16 {
		return ""
	}
	h := hex.EncodeToString(id)
	return h[:8] + "-" + h[8:12] + "-" + h[12:16] + "-" + h[16:20] + "-" + h[20:]
}
func B64(id []byte) string { return base64.StdEncoding.EncodeToString(id) }
func validID(id []byte, domain byte) bool {
	return len(id) == 16 && id[6]>>4 == 8 && id[8]&0xc0 == 0x80 && id[9] == domain
}
func instanceRef(id []byte) *wf.InstanceRef { return &wf.InstanceRef{Key: &wf.InstanceRef_Id{Id: id}} }
func podID(p *core.Pod) []byte              { return ID(7, string(p.UID)) }
func placementID(p *core.Pod) []byte        { return ID(11, string(p.UID)) }
func managed(p *core.Pod) bool              { return p.Labels[Managed] == "true" }
func ready(p *core.Pod) bool {
	if p.DeletionTimestamp != nil || p.Status.Phase != core.PodRunning || p.Annotations[Endpoint] == "" {
		return false
	}
	for _, c := range p.Status.Conditions {
		if c.Type == core.PodReady {
			return c.Status == core.ConditionTrue
		}
	}
	return false
}
func decodeApp(a *unstructured.Unstructured) (*wf.App, error) {
	data, err := json.Marshal(a.Object["spec"])
	if err != nil {
		return nil, err
	}
	var spec wf.App
	if err := protojson.Unmarshal(data, &spec); err != nil {
		return nil, err
	}
	if spec.Manifest == nil || spec.Manifest.Bundle == nil || spec.Manifest.Bundle.GetUrl() == "" {
		return nil, errors.New("manifest.bundle.url is required")
	}
	if spec.Manifest.Port == 0 {
		spec.Manifest.Port = 8080
	}
	if spec.Manifest.Protocol == "" {
		spec.Manifest.Protocol = "http"
	}
	if spec.Manifest.MemoryBytes == 0 {
		spec.Manifest.MemoryBytes = 64 << 20
	}
	if spec.Manifest.Runtime == "" {
		spec.Manifest.Runtime = "wasmtime"
	}
	if spec.Manifest.Abi == "" {
		spec.Manifest.Abi = "wasip2"
	}
	if spec.Scaling == nil {
		if _, ok := a.Object["spec"].(map[string]any)["replicas"]; !ok {
			spec.Replicas = 1
		}
	} else {
		s := spec.Scaling
		if s.Max == 0 || s.Min > s.Max {
			return nil, errors.New("scaling requires 0 <= min <= max, max > 0")
		}
		if s.Target == 0 {
			s.Target = 16
		}
		if s.UpMs == 0 {
			s.UpMs = 1000
		}
		if s.DownMs == 0 {
			s.DownMs = 10000
		}
		if s.IdleMs == 0 {
			s.IdleMs = 60000
		}
		if s.Waiting == 0 {
			s.Waiting = 128
		}
		if s.WaitMs == 0 {
			s.WaitMs = 30000
		}
	}
	if spec.Replicas > 1000 || spec.Scaling.GetMax() > 1000 {
		return nil, errors.New("at most 1000 replicas are supported")
	}
	if spec.Rollout == nil {
		spec.Rollout = &wf.Rollout{Surge: 1, DrainMs: 30000}
	}
	if spec.Rollout.DrainMs == 0 {
		spec.Rollout.DrainMs = 30000
	}
	if spec.Rollout.Surge == 0 && spec.Rollout.Unavailable == 0 {
		return nil, errors.New("rollout surge and unavailable cannot both be zero")
	}
	if len(spec.Manifest.Bundle.Digest) != 71 || !strings.HasPrefix(spec.Manifest.Bundle.Digest, "sha256:") {
		return nil, errors.New("bundle.digest must pin the archive; wafl app add computes it for local files and HTTP bundles")
	}
	if _, err := hex.DecodeString(spec.Manifest.Bundle.Digest[7:]); err != nil {
		return nil, errors.New("invalid bundle digest")
	}
	return &spec, nil
}
func manifestJSON(m *wf.Manifest) string {
	b, _ := protojson.MarshalOptions{UseProtoNames: false}.Marshal(m)
	// Protobuf deliberately randomizes insignificant JSON whitespace between
	// binaries. Canonicalize before hashing immutable Pod templates.
	var value any
	_ = json.Unmarshal(b, &value)
	b, _ = json.Marshal(value)
	return string(b)
}
func podManifest(p *core.Pod) (*wf.Manifest, error) {
	var m wf.Manifest
	err := protojson.Unmarshal([]byte(p.Annotations[Manifest]), &m)
	return &m, err
}
func endpointURL(p *core.Pod) string {
	m, err := podManifest(p)
	if err != nil {
		return ""
	}
	scheme := "http"
	if m.Protocol == "h2c" {
		scheme = "h2c"
	}
	return scheme + "://" + p.Annotations[Endpoint]
}
func appName(a *unstructured.Unstructured) string { return "wafl-" + digest(string(a.GetUID()))[:20] }
func owner(a *unstructured.Unstructured) meta.OwnerReference {
	return meta.OwnerReference{APIVersion: Group + "/v1alpha1", Kind: "App", Name: a.GetName(), UID: a.GetUID(), Controller: Ptr(true), BlockOwnerDeletion: Ptr(true)}
}
func rpcError(err error) error {
	if err == nil {
		return nil
	}
	return connect.NewError(connect.CodeUnavailable, errors.New("Kubernetes state is unavailable"))
}

type RunnerClient struct {
	URL       string
	Instances waflconnect.InstanceServiceClient
	Runners   waflconnect.RunnerServiceClient
}

func NewRunner(url string) *RunnerClient {
	h := &http.Client{Transport: &http.Transport{DialContext: (&net.Dialer{Timeout: 5 * time.Second}).DialContext, IdleConnTimeout: time.Minute}}
	return &RunnerClient{url, waflconnect.NewInstanceServiceClient(h, url, connect.WithProtoJSON()), waflconnect.NewRunnerServiceClient(h, url, connect.WithProtoJSON())}
}
func request[T any](msg *T, session []byte, revision uint64) *connect.Request[T] {
	r := connect.NewRequest(msg)
	if len(session) > 0 {
		r.Header().Set("wafl-session", UUID(session))
	}
	if revision > 0 {
		r.Header().Set("wafl-revision", fmt.Sprint(revision))
	}
	return r
}
func (r *RunnerClient) Status(ctx context.Context) (*wf.Runner, error) {
	v, e := r.Runners.List(ctx, connect.NewRequest(&wf.RunnerListRequest{}))
	if e != nil {
		return nil, e
	}
	if len(v.Msg.Items) != 1 {
		return nil, errors.New("runner returned no session")
	}
	return v.Msg.Items[0], nil
}
func (r *RunnerClient) Get(ctx context.Context, id []byte) (*wf.Instance, error) {
	v, e := r.Instances.Get(ctx, connect.NewRequest(&wf.InstanceGetRequest{Ref: instanceRef(id)}))
	if e != nil {
		return nil, e
	}
	return v.Msg, nil
}
func (r *RunnerClient) Seal(ctx context.Context, id, session []byte) error {
	_, e := r.Instances.Seal(ctx, request(&wf.InstanceSealRequest{Id: id}, session, 0))
	return e
}
func clone[T proto.Message](v T) T { return proto.Clone(v).(T) }
