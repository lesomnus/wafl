package platform

import (
	"context"
	"errors"
	"time"

	"connectrpc.com/connect"
	wf "github.com/lesomnus/wafl/kubernetes/gen/wafl"
	core "k8s.io/api/core/v1"
	apierrors "k8s.io/apimachinery/pkg/api/errors"
	meta "k8s.io/apimachinery/pkg/apis/meta/v1"
	"k8s.io/client-go/util/retry"
)

// Publish complete, secret-free snapshots. An endpoint is only a candidate:
// Traffic.Add must persist a gate and re-read the Pod before a proxy may dial.
func (c *Controller) routing(ctx context.Context) (*wf.Routing, error) {
	apps, err := c.Clients.activeApps(ctx)
	if err != nil {
		return nil, err
	}
	pods, err := c.Clients.Kube.CoreV1().Pods("").List(ctx, meta.ListOptions{LabelSelector: Managed + "=true"})
	if err != nil {
		return nil, err
	}
	orderApps(apps.Items)
	result := &wf.Routing{Version: 1}
	domains, ports := map[string]bool{}, map[uint32]bool{}
	for i := range apps.Items {
		a := &apps.Items[i]
		spec, err := decodeApp(a)
		if a.GetDeletionTimestamp() != nil || err != nil || !claimRoutes(spec, domains, ports) {
			continue
		}
		waiting, wait := uint32(128), uint64(30000)
		if spec.Scaling != nil {
			waiting, wait = spec.Scaling.Waiting, spec.Scaling.WaitMs
		}
		listeners := spec.Listeners
		if len(listeners) == 0 {
			listeners = []*wf.Listener{{Socket: "app"}}
		}
		for _, listener := range listeners {
			r := &wf.Route{App: ID(10, string(a.GetUID())), Tenant: a.GetNamespace(), Port: listener.Port, Protocol: spec.Manifest.Protocol, Suspended: spec.Scaling == nil && spec.Replicas == 0, Waiting: waiting, WaitMs: wait}
			if listener.Port == 0 {
				name := appName(a)
				r.Hosts = []string{spec.Domain, name, name + "." + a.GetNamespace(), name + "." + a.GetNamespace() + ".svc", name + "." + a.GetNamespace() + ".svc.cluster.local"}
			}
			for j := range pods.Items {
				p := &pods.Items[j]
				if r.Suspended || p.Labels[AppUID] != string(a.GetUID()) || !ready(p) {
					continue
				}
				address := socketAddress(p, listener.Socket)
				if listener.Port == 0 {
					address = p.Annotations[Endpoint]
				}
				if address != "" {
					r.Targets = append(r.Targets, &wf.RouteTarget{Placement: placementID(p), Address: address})
				}
			}
			result.Routes = append(result.Routes, r)
		}
	}
	return result, nil
}

func (a *AppAPI) Wake(ctx context.Context, request *connect.Request[wf.AppWakeRequest]) (*connect.Response[wf.App], error) {
	if !validID(request.Msg.GetRef().GetId(), 10) {
		return nil, invalidApp("Wake requires an App ID")
	}
	ns, err := a.tenant(ctx, request.Header())
	if err != nil {
		return nil, err
	}
	object, err := a.get(ctx, ns, request.Msg.Ref)
	if err != nil {
		return nil, err
	}
	spec, err := decodeApp(object)
	if err != nil {
		return nil, err
	}
	if object.GetDeletionTimestamp() != nil || spec.Scaling == nil && spec.Replicas == 0 {
		return nil, connect.NewError(connect.CodeFailedPrecondition, errors.New("App is suspended or deleting"))
	}
	if spec.Scaling != nil {
		err = retry.RetryOnConflict(retry.DefaultBackoff, func() error {
			// Fence App replacement and suspension throughout concurrent wakeups.
			fresh, e := a.get(ctx, ns, request.Msg.Ref)
			if e != nil {
				return e
			}
			current, e := decodeApp(fresh)
			if e != nil {
				return e
			}
			if fresh.GetDeletionTimestamp() != nil || current.Scaling == nil {
				return connect.NewError(connect.CodeFailedPrecondition, errors.New("App no longer accepts activation"))
			}
			api := a.Clients.Kube.AppsV1().Deployments(ns)
			d, e := api.Get(ctx, appName(fresh), meta.GetOptions{})
			if apierrors.IsNotFound(e) {
				// The leader may not have reconciled a newly added App yet.
				// The proxy keeps its waiting observer and repeats Wake while
				// awaiting a ready target, bounded by the startup deadline.
				return nil
			}
			if e != nil {
				return e
			}
			if d.DeletionTimestamp != nil {
				return errors.New("deployment is deleting")
			}
			if *d.Spec.Replicas > 0 {
				return nil
			}
			d.Spec.Replicas = Ptr(int32(max(1, current.Scaling.Min)))
			_, e = api.Update(ctx, d, meta.UpdateOptions{})
			return e
		})
		if err != nil {
			return nil, rpcError(err)
		}
	}
	value, err := appValue(object)
	if err != nil {
		return nil, err
	}
	return connect.NewResponse(value), nil
}

// App-only observers cover waiting demand; they never authorize an upstream
// connection. The placement observer must be admitted before this one closes.
func (t *Traffic) addWaiting(ctx context.Context, r *connect.Request[wf.TrafficAddRequest]) (*connect.Response[wf.Traffic], error) {
	api := &AppAPI{Clients: t.Clients}
	ns, err := api.tenant(ctx, r.Header())
	if err != nil {
		return nil, err
	}
	a, err := api.get(ctx, ns, r.Msg.App)
	if err != nil {
		return nil, err
	}
	spec, err := decodeApp(a)
	if err != nil {
		return nil, err
	}
	if a.GetDeletionTimestamp() != nil || spec.Scaling == nil && spec.Replicas == 0 {
		return nil, connect.NewError(connect.CodeFailedPrecondition, errors.New("App is suspended or deleting"))
	}
	v := &wf.Traffic{Id: r.Msg.GetId(), Session: r.Msg.GetSession(), Tenant: &wf.Tenant{Id: ID(1, ns)}, App: &wf.App{Id: ID(10, string(a.GetUID()))}, Activation: true, Active: 1, Started: 1, DateSeen: uint64(time.Now().UnixMilli())}
	cm := waitingRecord(t.Namespace, ns, a.GetName(), string(a.GetUID()), v)
	if err = t.createWaiting(ctx, cm, v); err != nil {
		return nil, err
	}
	_, v, err = t.get(ctx, v.Id)
	if err != nil {
		return nil, err
	}
	limit := uint32(128)
	if spec.Scaling != nil {
		limit = spec.Scaling.Waiting
	}
	observations, err := t.all(ctx, string(a.GetUID()))
	if err != nil {
		return nil, err
	}
	waiting := uint64(0)
	for _, o := range observations {
		if o.Activation && !o.Closed {
			waiting += uint64(o.Active)
		}
	}
	if waiting > uint64(limit) {
		_, _ = t.update(ctx, v.Id, func(_ *core.ConfigMap, v *wf.Traffic) error { v.Active = 0; v.Closed = true; return nil })
		return nil, connect.NewError(connect.CodeResourceExhausted, errors.New("activation queue full"))
	}
	return connect.NewResponse(v), nil
}

func waitingRecord(namespace, tenant, name, uid string, v *wf.Traffic) *core.ConfigMap {
	return &core.ConfigMap{ObjectMeta: meta.ObjectMeta{Name: trafficName(v.Id), Namespace: namespace, Labels: map[string]string{Managed: "true", Kind: "traffic", AppUID: uid}}, Data: map[string]string{"value": trafficData(v), "namespace": tenant, "app": name}}
}
func (t *Traffic) createWaiting(ctx context.Context, cm *core.ConfigMap, v *wf.Traffic) error {
	_, err := t.Clients.Kube.CoreV1().ConfigMaps(t.Namespace).Create(ctx, cm, meta.CreateOptions{})
	if apierrors.IsAlreadyExists(err) {
		_, old, e := t.get(ctx, v.Id)
		if e != nil {
			return e
		}
		if old.Session != v.Session || !old.Activation || UUID(old.App.GetId()) != UUID(v.App.Id) {
			return connect.NewError(connect.CodeFailedPrecondition, errors.New("observer identity differs"))
		}
		return nil
	}
	return err
}
