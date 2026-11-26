package platform

import (
	"context"
	"encoding/json"
	"slices"
	"strings"
	"time"

	core "k8s.io/api/core/v1"
	apierrors "k8s.io/apimachinery/pkg/api/errors"
	meta "k8s.io/apimachinery/pkg/apis/meta/v1"
	"k8s.io/client-go/util/retry"
)

// Kubernetes knows the physical runner Pod disappeared before the virtual-node
// heartbeat times out. Fail its sandboxes promptly; retain receipts until all
// observers acknowledge closure. No live runner reservation is reclaimed here.
func (c *Controller) reap(ctx context.Context) error {
	nodes, e := c.Clients.Kube.CoreV1().Nodes().List(ctx, meta.ListOptions{LabelSelector: Runtime + "=wasmtime"})
	if e != nil {
		return e
	}
	for _, node := range nodes.Items {
		ref := strings.SplitN(node.Annotations[RunnerPod], "/", 2)
		if len(ref) != 2 {
			continue
		}
		physical, e := c.Clients.Kube.CoreV1().Pods(ref[0]).Get(ctx, ref[1], meta.GetOptions{})
		gone := apierrors.IsNotFound(e) || e == nil && string(physical.UID) != node.Annotations[Group+"/runnerUID"]
		if e != nil && !gone {
			return e
		}
		if !gone {
			continue
		}
		if !node.Spec.Unschedulable {
			node.Spec.Unschedulable = true
			if _, e = c.Clients.Kube.CoreV1().Nodes().Update(ctx, &node, meta.UpdateOptions{}); e != nil {
				return e
			}
		}
		pods, e := c.Clients.Kube.CoreV1().Pods("").List(ctx, meta.ListOptions{FieldSelector: "spec.nodeName=" + node.Name})
		if e != nil {
			return e
		}
		for i := range pods.Items {
			p := &pods.Items[i]
			if !managed(p) {
				continue
			}
			p.Status.Phase = core.PodFailed
			p.Status.Reason = "RunnerRemoved"
			p.Status.Conditions = []core.PodCondition{{Type: core.PodReady, Status: core.ConditionFalse, Reason: "RunnerRemoved", LastTransitionTime: meta.Now()}}
			if _, e = c.Clients.Kube.CoreV1().Pods(p.Namespace).UpdateStatus(ctx, p, meta.UpdateOptions{}); e != nil {
				continue
			}
			_ = c.Clients.Kube.CoreV1().Pods(p.Namespace).Delete(ctx, p.Name, meta.DeleteOptions{Preconditions: &meta.Preconditions{UID: Ptr(p.UID)}})
		}
		receipts, e := c.Clients.Kube.CoreV1().ConfigMaps(c.Namespace).List(ctx, meta.ListOptions{LabelSelector: Kind + "=execution," + Group + "/node=" + node.Name})
		if e != nil {
			return e
		}
		pending := false
		cleanup := map[string]*core.Pod{}
		for i := range pods.Items {
			p := &pods.Items[i]
			if managed(p) {
				cleanup[string(p.UID)] = p
			}
		}
		for _, receipt := range receipts.Items {
			var p core.Pod
			if e = json.Unmarshal([]byte(receipt.Data["pod"]), &p); e != nil {
				return e
			}
			cleanup[string(p.UID)] = &p
		}
		for _, p := range cleanup {
			closed, e := c.Traffic.drained(ctx, string(p.UID))
			if e != nil {
				return e
			}
			if !closed {
				pending = true
				continue
			}
			provider := &Provider{Clients: c.Clients, Namespace: c.Namespace}
			if e = provider.finish(ctx, p); e != nil {
				pending = true
			}
		}
		if !pending {
			_ = c.Clients.Kube.CoreV1().Nodes().Delete(ctx, node.Name, meta.DeleteOptions{Preconditions: &meta.Preconditions{UID: Ptr(node.UID)}})
		}
	}
	// Acknowledged observer records are retained briefly so retries can see the
	// final response. Never collect an open gate because its heartbeat expired.
	items, e := c.Clients.Kube.CoreV1().ConfigMaps(c.Namespace).List(ctx, meta.ListOptions{LabelSelector: Kind + "=traffic"})
	if e != nil {
		return e
	}
	for i := range items.Items {
		cm := &items.Items[i]
		v, e := trafficValue(cm)
		if e != nil {
			return e
		}
		if v.Closed && v.Active == 0 && time.Now().UnixMilli()-int64(v.DateSeen) > 600000 {
			_ = c.Clients.Kube.CoreV1().ConfigMaps(c.Namespace).Delete(ctx, cm.Name, meta.DeleteOptions{Preconditions: &meta.Preconditions{ResourceVersion: Ptr(cm.ResourceVersion)}})
		}
	}
	return nil
}

func DrainNode(ctx context.Context, c *Clients, name string) error {
	e := retry.RetryOnConflict(retry.DefaultBackoff, func() error {
		node, e := c.Kube.CoreV1().Nodes().Get(ctx, name, meta.GetOptions{})
		if apierrors.IsNotFound(e) {
			return nil
		}
		if e != nil {
			return e
		}
		if node.Spec.Unschedulable {
			return nil
		}
		node.Spec.Unschedulable = true
		_, e = c.Kube.CoreV1().Nodes().Update(ctx, node, meta.UpdateOptions{})
		return e
	})
	if e != nil {
		return e
	}
	for {
		pods, e := c.Kube.CoreV1().Pods("").List(ctx, meta.ListOptions{FieldSelector: "spec.nodeName=" + name})
		if e != nil {
			return e
		}
		pending := false
		for _, p := range pods.Items {
			if !managed(&p) {
				continue
			}
			if slices.Contains(p.Finalizers, Finalizer) {
				pending = true
			}
			if p.DeletionTimestamp == nil {
				_ = c.Kube.CoreV1().Pods(p.Namespace).Delete(ctx, p.Name, meta.DeleteOptions{Preconditions: &meta.Preconditions{UID: Ptr(p.UID)}})
			}
		}
		if !pending {
			return nil
		}
		select {
		case <-ctx.Done():
			return ctx.Err()
		case <-time.After(time.Second):
		}
	}
}

// Keep snapshots referenced by live Pods or retained rollback templates. The
// grace window also covers a controller restart between snapshot and Deployment
// writes; a subsequent reconcile can recreate any unreferenced snapshot.
func (c *Controller) collectSnapshots(ctx context.Context) error {
	used := map[string]bool{}
	remember := func(namespace string, spec core.PodSpec) {
		for _, container := range spec.Containers {
			for _, env := range container.Env {
				if env.ValueFrom != nil && env.ValueFrom.SecretKeyRef != nil {
					used[namespace+"/"+env.ValueFrom.SecretKeyRef.Name] = true
				}
			}
		}
	}
	pods, e := c.Clients.Kube.CoreV1().Pods("").List(ctx, meta.ListOptions{LabelSelector: Managed + "=true"})
	if e != nil {
		return e
	}
	for _, p := range pods.Items {
		remember(p.Namespace, p.Spec)
	}
	sets, e := c.Clients.Kube.AppsV1().ReplicaSets("").List(ctx, meta.ListOptions{LabelSelector: Managed + "=true"})
	if e != nil {
		return e
	}
	for _, s := range sets.Items {
		remember(s.Namespace, s.Spec.Template.Spec)
	}
	deployments, e := c.Clients.Kube.AppsV1().Deployments("").List(ctx, meta.ListOptions{LabelSelector: Managed + "=true"})
	if e != nil {
		return e
	}
	for _, d := range deployments.Items {
		remember(d.Namespace, d.Spec.Template.Spec)
	}
	apps, e := c.Clients.Dynamic.Resource(Apps).Namespace("").List(ctx, meta.ListOptions{})
	if e != nil {
		return e
	}
	for _, a := range apps.Items {
		spec, e := decodeApp(&a)
		if e != nil {
			continue
		}
		for _, binding := range spec.Secrets {
			if ref := binding.GetSecretKeyRef(); ref != nil {
				used[a.GetNamespace()+"/"+ref.Name] = true
			}
		}
	}
	secrets, e := c.Clients.Kube.CoreV1().Secrets("").List(ctx, meta.ListOptions{LabelSelector: Kind + "=snapshot"})
	if e != nil {
		return e
	}
	for _, s := range secrets.Items {
		if !used[s.Namespace+"/"+s.Name] && time.Since(s.CreationTimestamp.Time) > 10*time.Minute {
			if e = c.Clients.Kube.CoreV1().Secrets(s.Namespace).Delete(ctx, s.Name, meta.DeleteOptions{Preconditions: &meta.Preconditions{ResourceVersion: Ptr(s.ResourceVersion)}}); e != nil && !apierrors.IsNotFound(e) && !apierrors.IsConflict(e) {
				return e
			}
		}
	}
	return nil
}
