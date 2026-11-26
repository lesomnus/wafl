package platform

import (
	"context"
	"errors"
	"testing"

	core "k8s.io/api/core/v1"
	apierrors "k8s.io/apimachinery/pkg/api/errors"
	meta "k8s.io/apimachinery/pkg/apis/meta/v1"
	"k8s.io/apimachinery/pkg/runtime"
	"k8s.io/apimachinery/pkg/runtime/schema"
	"k8s.io/client-go/kubernetes/fake"
	ktesting "k8s.io/client-go/testing"
)

func TestReapCancelledUnscheduledPod(t *testing.T) {
	pending := &core.Pod{ObjectMeta: meta.ObjectMeta{Name: "pending", Namespace: "demo", UID: "pending", ResourceVersion: "1", Labels: map[string]string{Managed: "true"}, DeletionTimestamp: Ptr(meta.Now()), Finalizers: []string{Finalizer}}}
	assigned := pending.DeepCopy()
	assigned.Name, assigned.UID, assigned.Spec.NodeName = "assigned", "assigned", "live-runner"
	live := pending.DeepCopy()
	live.Name, live.UID, live.DeletionTimestamp = "live", "live", nil
	foreign := pending.DeepCopy()
	foreign.Name, foreign.UID, foreign.Labels = "foreign", "foreign", nil
	k := fake.NewClientset(pending, assigned, live, foreign)
	c := &Controller{Clients: &Clients{Kube: k}}
	if err := c.reapUnscheduled(context.Background()); err != nil {
		t.Fatal(err)
	}
	if _, err := k.CoreV1().Pods("demo").Get(context.Background(), "pending", meta.GetOptions{}); !apierrors.IsNotFound(err) {
		t.Fatalf("cancelled Pod remains: %v", err)
	}
	for _, name := range []string{"assigned", "live", "foreign"} {
		if _, err := k.CoreV1().Pods("demo").Get(context.Background(), name, meta.GetOptions{}); err != nil {
			t.Fatalf("unrelated Pod %s changed: %v", name, err)
		}
	}
}

func TestUnscheduledCleanupFencesConcurrentBinding(t *testing.T) {
	pod := &core.Pod{ObjectMeta: meta.ObjectMeta{Name: "pending", Namespace: "demo", UID: "pending", ResourceVersion: "1", Labels: map[string]string{Managed: "true"}, DeletionTimestamp: Ptr(meta.Now()), Finalizers: []string{Finalizer, "example.com/another-owner"}}}
	k := fake.NewClientset(pod)
	k.PrependReactor("update", "pods", func(action ktesting.Action) (bool, runtime.Object, error) {
		p := action.(ktesting.UpdateAction).GetObject().(*core.Pod)
		if p.ResourceVersion != "1" || len(p.Finalizers) != 1 || p.Finalizers[0] != "example.com/another-owner" {
			t.Fatal("cleanup lost CAS or another owner's finalizer")
		}
		return true, nil, apierrors.NewConflict(schema.GroupResource{Resource: "pods"}, p.Name, errors.New("scheduler bound the Pod"))
	})
	c := &Controller{Clients: &Clients{Kube: k}}
	if err := c.reapUnscheduled(context.Background()); err != nil {
		t.Fatal(err)
	}
	for _, action := range k.Actions() {
		if action.GetVerb() == "delete" {
			t.Fatal("stale unscheduled snapshot deleted a concurrently bound Pod")
		}
	}
}
