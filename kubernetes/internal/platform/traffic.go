package platform

import (
	"context"
	"errors"
	"fmt"
	"regexp"
	"time"

	"connectrpc.com/connect"
	wf "github.com/lesomnus/wafl/kubernetes/gen/wafl"
	"github.com/lesomnus/wafl/kubernetes/gen/wafl/waflconnect"
	"google.golang.org/protobuf/encoding/protojson"
	"google.golang.org/protobuf/reflect/protoreflect"
	core "k8s.io/api/core/v1"
	apierrors "k8s.io/apimachinery/pkg/api/errors"
	meta "k8s.io/apimachinery/pkg/apis/meta/v1"
	"k8s.io/client-go/util/retry"
)

const Kind = Group + "/kind"
const PodUID = Group + "/pod"
const Placement = Group + "/placement"

type Traffic struct {
	waflconnect.UnimplementedTrafficServiceHandler
	Clients   *Clients
	Namespace string
}

func trafficName(id []byte) string { return "wafl-traffic-" + UUID(id) }
func trafficValue(cm *core.ConfigMap) (*wf.Traffic, error) {
	var t wf.Traffic
	err := protojson.Unmarshal([]byte(cm.Data["value"]), &t)
	return &t, err
}
func trafficData(t *wf.Traffic) string { b, _ := protojson.Marshal(t); return string(b) }
func (t *Traffic) get(ctx context.Context, id []byte) (*core.ConfigMap, *wf.Traffic, error) {
	if !validID(id, 13) {
		return nil, nil, connect.NewError(connect.CodeInvalidArgument, errors.New("invalid observer ID"))
	}
	cm, err := t.Clients.Kube.CoreV1().ConfigMaps(t.Namespace).Get(ctx, trafficName(id), meta.GetOptions{})
	if apierrors.IsNotFound(err) {
		return nil, nil, connect.NewError(connect.CodeNotFound, errors.New("observer not found"))
	}
	if err != nil {
		return nil, nil, rpcError(err)
	}
	v, err := trafficValue(cm)
	return cm, v, err
}
func (t *Traffic) update(ctx context.Context, id []byte, fn func(*core.ConfigMap, *wf.Traffic) error) (*wf.Traffic, error) {
	var result *wf.Traffic
	err := retry.RetryOnConflict(retry.DefaultBackoff, func() error {
		cm, v, e := t.get(ctx, id)
		if e != nil {
			return e
		}
		if e = fn(cm, v); e != nil {
			return e
		}
		cm.Data["value"] = trafficData(v)
		_, e = t.Clients.Kube.CoreV1().ConfigMaps(t.Namespace).Update(ctx, cm, meta.UpdateOptions{})
		result = v
		return e
	})
	return result, err
}
func (t *Traffic) pod(ctx context.Context, cm *core.ConfigMap) (*core.Pod, error) {
	p, e := t.Clients.Kube.CoreV1().Pods(cm.Data["namespace"]).Get(ctx, cm.Data["pod"], meta.GetOptions{})
	if e != nil {
		return nil, e
	}
	if string(p.UID) != cm.Labels[PodUID] {
		return nil, apierrors.NewNotFound(core.Resource("pods"), cm.Data["pod"])
	}
	return p, nil
}
func (t *Traffic) Add(ctx context.Context, r *connect.Request[wf.TrafficAddRequest]) (*connect.Response[wf.Traffic], error) {
	id := r.Msg.GetId()
	placement := r.Msg.GetPlacement().GetId()
	session := r.Msg.GetSession()
	allowed := true
	r.Msg.ProtoReflect().Range(func(f protoreflect.FieldDescriptor, _ protoreflect.Value) bool {
		if f.Name() != "id" && f.Name() != "placement" && f.Name() != "session" && f.Name() != "app" {
			allowed = false
		}
		return true
	})
	waiting := r.Msg.App != nil && r.Msg.Placement == nil && validID(r.Msg.App.GetId(), 10)
	if !allowed || !validID(id, 13) || !(waiting || r.Msg.App == nil && validID(placement, 11)) || !regexp.MustCompile(`^[A-Za-z0-9-]{1,128}$`).MatchString(session) {
		return nil, connect.NewError(connect.CodeInvalidArgument, errors.New("invalid observer identity"))
	}
	if waiting {
		return t.addWaiting(ctx, r)
	}
	pods, err := t.Clients.Kube.CoreV1().Pods("").List(ctx, meta.ListOptions{LabelSelector: Placement + "=" + UUID(placement)})
	if err != nil {
		return nil, rpcError(err)
	}
	if len(pods.Items) != 1 {
		return nil, connect.NewError(connect.CodeFailedPrecondition, errors.New("execution no longer accepts observers"))
	}
	p := &pods.Items[0]
	v := &wf.Traffic{Id: id, Session: session, Tenant: &wf.Tenant{Id: ID(1, p.Namespace)}, Placement: &wf.Placement{Id: placement}, App: &wf.App{Id: ID(10, p.Labels[AppUID])}, DateSeen: uint64(time.Now().UnixMilli())}
	cm := &core.ConfigMap{ObjectMeta: meta.ObjectMeta{Name: trafficName(id), Namespace: t.Namespace, Labels: map[string]string{Managed: "true", Kind: "traffic", PodUID: string(p.UID), AppUID: p.Labels[AppUID]}}, Data: map[string]string{"value": trafficData(v), "namespace": p.Namespace, "pod": p.Name}}
	_, err = t.Clients.Kube.CoreV1().ConfigMaps(t.Namespace).Create(ctx, cm, meta.CreateOptions{})
	if apierrors.IsAlreadyExists(err) {
		cm, v, err = t.get(ctx, id)
		if err == nil && (v.Session != session || UUID(v.Placement.GetId()) != UUID(placement)) {
			err = connect.NewError(connect.CodeFailedPrecondition, errors.New("observer identity differs"))
		}
	}
	if err != nil {
		return nil, err
	}
	// Persist registration BEFORE a fresh Pod read. Deletion first stops new
	// admissions, then lists registrations; neither side can miss an admitted gate.
	p, err = t.pod(ctx, cm)
	if apierrors.IsNotFound(err) || err == nil && !ready(p) {
		v, err = t.update(ctx, id, func(_ *core.ConfigMap, v *wf.Traffic) error { v.Draining = true; return nil })
	}
	if err != nil {
		return nil, rpcError(err)
	}
	return connect.NewResponse(v), nil
}
func (t *Traffic) Get(ctx context.Context, r *connect.Request[wf.TrafficGetRequest]) (*connect.Response[wf.Traffic], error) {
	_, v, e := t.get(ctx, r.Msg.GetRef().GetId())
	if e != nil {
		return nil, e
	}
	if r.Msg.Select != nil {
		selectFields(v.ProtoReflect(), r.Msg.Select.ProtoReflect())
	}
	return connect.NewResponse(v), nil
}
func (t *Traffic) List(ctx context.Context, r *connect.Request[wf.TrafficListRequest]) (*connect.Response[wf.TrafficListResponse], error) {
	items, e := t.Clients.Kube.CoreV1().ConfigMaps(t.Namespace).List(ctx, meta.ListOptions{LabelSelector: Kind + "=traffic"})
	if e != nil {
		return nil, rpcError(e)
	}
	result := &wf.TrafficListResponse{}
	for i := range items.Items {
		v, e := trafficValue(&items.Items[i])
		if e != nil {
			return nil, e
		}
		result.Items = append(result.Items, v)
	}
	return trafficPage(r.Msg, result.Items)
}
func (t *Traffic) Report(ctx context.Context, r *connect.Request[wf.TrafficReportRequest]) (*connect.Response[wf.Traffic], error) {
	m := r.Msg
	v, e := t.update(ctx, m.GetRef().GetId(), func(cm *core.ConfigMap, v *wf.Traffic) error {
		if v.Activation && (m.Active > 1 || m.Started != 1 || m.Closed && m.Active != 0) {
			return connect.NewError(connect.CodeFailedPrecondition, errors.New("waiting observer tracks exactly one request"))
		}
		if m.Sequence <= v.Sequence {
			return nil
		}
		if v.Closed && v.Active == 0 {
			return nil
		}
		if m.Started < v.Started || m.BusyMs < v.BusyMs || uint64(m.Active) > m.Started || v.Closed && (!m.Closed || m.Active > v.Active || m.Started != v.Started) {
			return connect.NewError(connect.CodeFailedPrecondition, errors.New("observer counters cannot move backwards"))
		}
		if !v.Activation {
			p, e := t.pod(ctx, cm)
			if apierrors.IsNotFound(e) {
				v.Draining = true
			} else if e != nil {
				return e
			} else if !ready(p) {
				v.Draining = true
			}
		}
		v.Sequence = m.Sequence
		v.Active = m.Active
		v.Started = m.Started
		v.BusyMs = m.BusyMs
		v.Closed = m.Closed
		v.DateSeen = uint64(time.Now().UnixMilli())
		return nil
	})
	if e != nil {
		return nil, e
	}
	return connect.NewResponse(v), nil
}
func (t *Traffic) Close(ctx context.Context, r *connect.Request[wf.TrafficCloseRequest]) (*connect.Response[wf.Traffic], error) {
	if !r.Msg.Withdrawn {
		return nil, connect.NewError(connect.CodeFailedPrecondition, errors.New("withdrawn confirmation is required"))
	}
	v, e := t.update(ctx, r.Msg.GetRef().GetId(), func(_ *core.ConfigMap, v *wf.Traffic) error {
		v.Draining = true
		v.Closed = true
		v.Active = 0
		return nil
	})
	if e != nil {
		return nil, e
	}
	return connect.NewResponse(v), nil
}
func (t *Traffic) drained(ctx context.Context, uid string) (bool, error) {
	items, e := t.Clients.Kube.CoreV1().ConfigMaps(t.Namespace).List(ctx, meta.ListOptions{LabelSelector: Kind + "=traffic," + PodUID + "=" + uid})
	if e != nil {
		return false, e
	}
	for i := range items.Items {
		v, e := trafficValue(&items.Items[i])
		if e != nil {
			return false, e
		}
		if !v.Closed || v.Active != 0 {
			return false, nil
		}
	}
	return true, nil
}
func (t *Traffic) all(ctx context.Context, uid string) ([]*wf.Traffic, error) {
	items, e := t.Clients.Kube.CoreV1().ConfigMaps(t.Namespace).List(ctx, meta.ListOptions{LabelSelector: Kind + "=traffic," + AppUID + "=" + uid})
	if e != nil {
		return nil, e
	}
	out := []*wf.Traffic{}
	for i := range items.Items {
		v, e := trafficValue(&items.Items[i])
		if e != nil {
			return nil, fmt.Errorf("invalid traffic record")
		}
		out = append(out, v)
	}
	return out, nil
}
