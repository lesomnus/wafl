package platform

import (
	"context"
	"encoding/base64"
	"encoding/json"
	"errors"
	"log/slog"
	"net/http"
	"net/url"
	"os"
	"slices"
	"strconv"
	"sync"
	"time"

	"connectrpc.com/connect"
	wf "github.com/lesomnus/wafl/kubernetes/gen/wafl"
	"github.com/lesomnus/wafl/kubernetes/gen/wafl/waflconnect"
	"github.com/virtual-kubelet/virtual-kubelet/errdefs"
	"github.com/virtual-kubelet/virtual-kubelet/node"
	core "k8s.io/api/core/v1"
	apierrors "k8s.io/apimachinery/pkg/api/errors"
	"k8s.io/apimachinery/pkg/api/resource"
	meta "k8s.io/apimachinery/pkg/apis/meta/v1"
	"k8s.io/client-go/informers"
	"k8s.io/client-go/kubernetes/scheme"
	"k8s.io/client-go/tools/record"
	"k8s.io/client-go/util/retry"
)

type Provider struct {
	Clients         *Clients
	Runner          *RunnerClient
	Traffic         *Traffic
	Name, Namespace string
	ctx             context.Context
	mu              sync.Mutex
	pods            map[string]*core.Pod
	notify          func(*core.Pod)
	notifyNode      func(*core.Node)
	node            *core.Node
}

func NewProvider(c *Clients, name, url, namespace string) *Provider {
	return &Provider{Clients: c, Runner: NewRunner(url), Traffic: &Traffic{Clients: c, Namespace: namespace}, Name: name, Namespace: namespace, pods: map[string]*core.Pod{}}
}
func podKey(p *core.Pod) string        { return p.Namespace + "/" + p.Name }
func executionName(p *core.Pod) string { return "wafl-execution-" + string(p.UID) }
func (p *Provider) NotifyPods(_ context.Context, f func(*core.Pod)) {
	p.mu.Lock()
	p.notify = f
	p.mu.Unlock()
}
func (p *Provider) NotifyNodeStatus(_ context.Context, f func(*core.Node)) {
	p.mu.Lock()
	p.notifyNode = f
	p.mu.Unlock()
}
func (p *Provider) Ping(ctx context.Context) error {
	r, e := p.Runner.Status(ctx)
	if e != nil {
		return e
	}
	if !r.Accepting || !slices.Contains(r.Capabilities, "halt-instance-v1") {
		return errors.New("runner does not accept Kubernetes executions")
	}
	p.mu.Lock()
	n := p.node.DeepCopy()
	cb := p.notifyNode
	p.mu.Unlock()
	if n != nil {
		n.Status.Capacity = core.ResourceList{core.ResourcePods: *resource.NewQuantity(int64(r.Capacity.Instances), resource.DecimalSI), Slots: *resource.NewQuantity(int64(r.Capacity.Running), resource.DecimalSI), Memory: *resource.NewQuantity(int64(r.Capacity.MemoryBytes), resource.DecimalSI)}
		n.Status.Allocatable = n.Status.Capacity.DeepCopy()
		if cb != nil {
			cb(n)
		}
	}
	return nil
}
func (p *Provider) GetPod(_ context.Context, ns, name string) (*core.Pod, error) {
	p.mu.Lock()
	defer p.mu.Unlock()
	v := p.pods[ns+"/"+name]
	if v == nil {
		return nil, errdefs.NotFound("Pod not found")
	}
	return v.DeepCopy(), nil
}
func (p *Provider) GetPodStatus(ctx context.Context, ns, name string) (*core.PodStatus, error) {
	v, e := p.GetPod(ctx, ns, name)
	if e != nil {
		return nil, e
	}
	return &v.Status, nil
}
func (p *Provider) GetPods(context.Context) ([]*core.Pod, error) {
	p.mu.Lock()
	defer p.mu.Unlock()
	v := []*core.Pod{}
	for _, pod := range p.pods {
		v = append(v, pod.DeepCopy())
	}
	return v, nil
}
func (p *Provider) CreatePod(ctx context.Context, pod *core.Pod) error {
	if _, e := validatePod(pod); e != nil {
		return errdefs.InvalidInput(e.Error())
	}
	data, _ := json.Marshal(pod)
	cm := &core.ConfigMap{ObjectMeta: meta.ObjectMeta{Name: executionName(pod), Namespace: p.Namespace, Labels: map[string]string{Managed: "true", Kind: "execution", Group + "/node": p.Name, PodUID: string(pod.UID)}}, Data: map[string]string{"pod": string(data)}}
	_, e := p.Clients.Kube.CoreV1().ConfigMaps(p.Namespace).Create(ctx, cm, meta.CreateOptions{})
	if e != nil && !apierrors.IsAlreadyExists(e) {
		return e
	}
	p.launch(pod)
	return nil
}
func (p *Provider) launch(pod *core.Pod) {
	p.mu.Lock()
	old := p.pods[podKey(pod)]
	if old != nil && old.UID == pod.UID {
		p.mu.Unlock()
		return
	}
	p.pods[podKey(pod)] = pod.DeepCopy()
	p.mu.Unlock()
	go p.run(pod.DeepCopy())
}
func (p *Provider) UpdatePod(ctx context.Context, pod *core.Pod) error { return p.CreatePod(ctx, pod) }
func (p *Provider) DeletePod(ctx context.Context, pod *core.Pod) error {
	// The durable receipt also lets a replacement provider finish cleanup after
	// a force deletion or an agent restart. The worker observes deletion directly.
	return p.CreatePod(ctx, pod)
}
func (p *Provider) publish(pod *core.Pod, phase core.PodPhase, reason string, healthy bool, instance *wf.Instance, restarts int32) {
	p.mu.Lock()
	old := p.pods[podKey(pod)]
	next := pod.DeepCopy()
	now := meta.Now()
	next.Status = core.PodStatus{Phase: phase, Reason: reason, StartTime: &now}
	if old != nil && old.Status.StartTime != nil {
		next.Status.StartTime = old.Status.StartTime
	}
	condition := core.PodCondition{Type: core.PodReady, Status: core.ConditionFalse, Reason: reason, LastTransitionTime: now}
	if healthy {
		condition.Status = core.ConditionTrue
	}
	if old != nil {
		for _, c := range old.Status.Conditions {
			if c.Type == condition.Type && c.Status == condition.Status && c.Reason == condition.Reason {
				condition.LastTransitionTime = c.LastTransitionTime
			}
		}
	}
	next.Status.Conditions = []core.PodCondition{condition, {Type: core.PodInitialized, Status: core.ConditionTrue, LastTransitionTime: *next.Status.StartTime}}
	cs := core.ContainerStatus{Name: "app", Image: pod.Spec.Containers[0].Image, ImageID: pod.Spec.Containers[0].Image, Ready: healthy, RestartCount: restarts, Started: Ptr(phase == core.PodRunning)}
	if instance != nil {
		cs.ContainerID = "wasm://" + UUID(instance.Id) + "/" + strconv.FormatUint(instance.Generation, 10)
	}
	switch phase {
	case core.PodRunning:
		cs.State.Running = &core.ContainerStateRunning{StartedAt: *next.Status.StartTime}
	case core.PodFailed, core.PodSucceeded:
		cs.State.Terminated = &core.ContainerStateTerminated{Reason: reason, FinishedAt: now, ExitCode: 1}
	default:
		cs.State.Waiting = &core.ContainerStateWaiting{Reason: reason}
	}
	next.Status.ContainerStatuses = []core.ContainerStatus{cs}
	changed := true
	if old != nil {
		a, _ := json.Marshal(old.Status)
		b, _ := json.Marshal(next.Status)
		changed = string(a) != string(b)
	}
	p.pods[podKey(pod)] = next
	cb := p.notify
	p.mu.Unlock()
	if changed && cb != nil {
		cb(next.DeepCopy())
	}
}
func (p *Provider) receipt(ctx context.Context, pod *core.Pod) (*core.ConfigMap, error) {
	return p.Clients.Kube.CoreV1().ConfigMaps(p.Namespace).Get(ctx, executionName(pod), meta.GetOptions{})
}
func (p *Provider) session(ctx context.Context, pod *core.Pod, r *wf.Runner) ([]byte, error) {
	var session []byte
	err := retry.RetryOnConflict(retry.DefaultBackoff, func() error {
		cm, e := p.receipt(ctx, pod)
		if e != nil {
			return e
		}
		if v := cm.Data["session"]; v != "" {
			session, e = base64.StdEncoding.DecodeString(v)
			return e
		}
		session = r.Id
		cm.Data["session"] = B64(session)
		_, e = p.Clients.Kube.CoreV1().ConfigMaps(p.Namespace).Update(ctx, cm, meta.UpdateOptions{})
		return e
	})
	return session, err
}
func (p *Provider) annotate(ctx context.Context, pod *core.Pod, i *wf.Instance) error {
	return retry.RetryOnConflict(retry.DefaultBackoff, func() error {
		v, e := p.Clients.Kube.CoreV1().Pods(pod.Namespace).Get(ctx, pod.Name, meta.GetOptions{})
		if e != nil {
			return e
		}
		if v.UID != pod.UID {
			return errors.New("Pod UID changed")
		}
		if v.Annotations == nil {
			v.Annotations = map[string]string{}
		}
		a := map[string]string{Instance: UUID(i.Id), Session: UUID(i.Session), Endpoint: i.GetEndpoint().GetAddress(), Runner: p.Runner.URL, Execution: strconv.FormatUint(i.Generation, 10)}
		if physical := os.Getenv("POD_NAME"); physical != "" {
			a[RunnerPod] = p.Namespace + "/" + physical
		}
		changed := false
		for k, val := range a {
			if v.Annotations[k] != val {
				v.Annotations[k] = val
				changed = true
			}
		}
		if v.Labels[Placement] != UUID(placementID(pod)) {
			v.Labels[Placement] = UUID(placementID(pod))
			changed = true
		}
		if !changed {
			return nil
		}
		_, e = p.Clients.Kube.CoreV1().Pods(pod.Namespace).Update(ctx, v, meta.UpdateOptions{})
		return e
	})
}
func (p *Provider) secretValues(ctx context.Context, pod *core.Pod) (map[string]string, error) {
	out := map[string]string{}
	cache := map[string]*core.Secret{}
	size := 0
	for _, e := range pod.Spec.Containers[0].Env {
		r := e.ValueFrom.SecretKeyRef
		s := cache[r.Name]
		if s == nil {
			var err error
			s, err = p.Clients.Kube.CoreV1().Secrets(pod.Namespace).Get(ctx, r.Name, meta.GetOptions{})
			if err != nil {
				return nil, errors.New("cannot read immutable Secret snapshot")
			}
			if s.Immutable == nil || !*s.Immutable || s.Labels[AppUID] != pod.Labels[AppUID] {
				return nil, errors.New("Secret is not an App snapshot")
			}
			cache[r.Name] = s
		}
		value, ok := s.Data[r.Key]
		if !ok {
			return nil, errors.New("snapshot key missing")
		}
		size += len(value) + len(e.Name)
		if size > 65536 {
			return nil, errors.New("secret environment exceeds limit")
		}
		out[e.Name] = string(value)
	}
	return out, nil
}
func (p *Provider) finish(ctx context.Context, pod *core.Pod) error {
	err := retry.RetryOnConflict(retry.DefaultBackoff, func() error {
		v, e := p.Clients.Kube.CoreV1().Pods(pod.Namespace).Get(ctx, pod.Name, meta.GetOptions{})
		if apierrors.IsNotFound(e) {
			return nil
		}
		if e != nil {
			return e
		}
		if v.UID != pod.UID {
			return nil
		}
		v.Finalizers = slices.DeleteFunc(v.Finalizers, func(s string) bool { return s == Finalizer })
		_, e = p.Clients.Kube.CoreV1().Pods(pod.Namespace).Update(ctx, v, meta.UpdateOptions{})
		return e
	})
	if err != nil {
		return err
	}
	// A kubelet must complete the API deletion too: expiration of the grace
	// period alone does not remove a Pod from the API server.
	err = p.Clients.Kube.CoreV1().Pods(pod.Namespace).Delete(ctx, pod.Name, meta.DeleteOptions{GracePeriodSeconds: Ptr(int64(0)), Preconditions: &meta.Preconditions{UID: Ptr(pod.UID)}})
	if err != nil && !apierrors.IsNotFound(err) && !apierrors.IsConflict(err) {
		return err
	}
	err = p.Clients.Kube.CoreV1().ConfigMaps(p.Namespace).Delete(ctx, executionName(pod), meta.DeleteOptions{})
	if apierrors.IsNotFound(err) {
		err = nil
	}
	return err
}
func (p *Provider) cleanup(ctx context.Context, pod *core.Pod, session []byte, deadline time.Time) (bool, error) {
	r, e := p.Runner.Status(ctx)
	if e != nil {
		return false, e
	}
	if UUID(r.Id) != UUID(session) {
		// An observed new process session is stronger than a heartbeat timeout.
		// Its durable journal still refuses reuse until every old gate is closed.
		done, e := p.Traffic.drained(ctx, string(pod.UID))
		if e != nil || !done {
			return false, e
		}
		client := waflconnect.NewReservationServiceClient(http.DefaultClient, p.Runner.URL, connect.WithProtoJSON())
		list, e := client.List(ctx, connect.NewRequest(&wf.ReservationListRequest{}))
		if e != nil {
			return false, e
		}
		for _, claim := range list.Msg.Items {
			if UUID(claim.Instance) == UUID(podID(pod)) && UUID(claim.Session) == UUID(session) {
				_, e = client.Release(ctx, request(&wf.ReservationReleaseRequest{Ref: &wf.ReservationRef{Key: &wf.ReservationRef_Id{Id: claim.Id}}, Token: claim.Token, Withdrawn: true}, r.Id, 0))
				if e != nil {
					return false, e
				}
			}
		}
		return true, nil
	}
	if e = p.Runner.Seal(ctx, podID(pod), session); e != nil {
		return false, e
	}
	i, e := p.Runner.Get(ctx, podID(pod))
	if connect.CodeOf(e) == connect.CodeNotFound {
		return true, nil
	}
	if e != nil {
		return false, e
	}
	if !i.Draining {
		v, e := p.Runner.Instances.Drain(ctx, request(&wf.InstanceDrainRequest{Ref: instanceRef(i.Id)}, session, i.Revision))
		if e != nil {
			return false, e
		}
		i = v.Msg
	}
	if !deadline.IsZero() && time.Now().After(deadline) && i.Phase != wf.Phase_PHASE_STOPPED {
		v, e := p.Runner.Instances.Halt(ctx, request(&wf.InstanceStopRequest{Ref: instanceRef(i.Id)}, session, i.Revision))
		if e != nil {
			return false, e
		}
		i = v.Msg
	}
	done, e := p.Traffic.drained(ctx, string(pod.UID))
	if e != nil || !done {
		return false, e
	}
	if i.GetEndpoint().GetAddress() != "" || i.Generation > 0 {
		v, e := p.Runner.Instances.Release(ctx, request(&wf.InstanceReleaseRequest{Ref: instanceRef(i.Id), Generation: i.Generation}, session, i.Revision))
		if e != nil {
			return false, e
		}
		i = v.Msg
	}
	_, e = p.Runner.Instances.Erase(ctx, request(instanceRef(i.Id), session, i.Revision))
	return e == nil, e
}

func (p *Provider) run(original *core.Pod) {
	ctx := p.ctx
	defer func() {
		p.mu.Lock()
		if old := p.pods[podKey(original)]; old != nil && old.UID == original.UID {
			delete(p.pods, podKey(original))
		}
		p.mu.Unlock()
	}()
	id := podID(original)
	var session []byte
	var nextStart time.Time
	var failedDeadline time.Time
	var failures int
	var successes, bad int32
	var nextProbe time.Time
	healthy := false
	ticker := time.NewTicker(500 * time.Millisecond)
	defer ticker.Stop()
	for {
		select {
		case <-ctx.Done():
			return
		case <-ticker.C:
		}
		step, cancel := context.WithTimeout(ctx, 15*time.Second)
		pod, e := p.Clients.Kube.CoreV1().Pods(original.Namespace).Get(step, original.Name, meta.GetOptions{})
		absent := apierrors.IsNotFound(e) || e == nil && pod.UID != original.UID
		if e != nil && !absent {
			cancel()
			continue
		}
		if absent {
			pod = original.DeepCopy()
			pod.DeletionTimestamp = Ptr(meta.Now())
		}
		r, e := p.Runner.Status(step)
		if e != nil {
			p.publish(pod, core.PodRunning, "RunnerUnavailable", false, nil, int32(failures))
			cancel()
			continue
		}
		if session == nil {
			session, e = p.session(step, pod, r)
			if e != nil {
				cancel()
				continue
			}
		}
		lost := UUID(session) != UUID(r.Id)
		if pod.DeletionTimestamp != nil || lost {
			reason := "Draining"
			phase := core.PodRunning
			deadline := time.Time{}
			if pod.DeletionTimestamp != nil {
				deadline = pod.DeletionTimestamp.Time
			}
			if lost {
				reason = "RunnerRestarted"
				phase = core.PodFailed
			}
			if !deadline.IsZero() && time.Now().After(deadline) {
				reason = "TerminationGraceExpired"
				phase = core.PodFailed
			}
			p.publish(pod, phase, reason, false, nil, int32(failures))
			done, e := p.cleanup(step, pod, session, deadline)
			if e == nil && done {
				p.publish(pod, core.PodFailed, "ExecutionStopped", false, nil, int32(failures))
				e = p.finish(step, pod)
				if e == nil {
					cancel()
					return
				}
			}
			cancel()
			continue
		}
		i, e := p.Runner.Get(step, id)
		if connect.CodeOf(e) == connect.CodeNotFound && time.Now().After(nextStart) {
			m, err := validatePod(pod)
			if err != nil {
				p.publish(pod, core.PodPending, "UnsupportedPod", false, nil, int32(failures))
				cancel()
				continue
			}
			values, err := p.secretValues(step, pod)
			if err != nil {
				p.publish(pod, core.PodPending, "CreateContainerConfigError", false, nil, int32(failures))
				cancel()
				continue
			}
			p.publish(pod, core.PodPending, "Preparing", false, nil, int32(failures))
			cancel()
			start, stop := context.WithTimeout(ctx, 2*time.Minute)
			v, err := p.Runner.Instances.Add(start, request(&wf.InstanceAddRequest{Id: id, Alias: Ptr("pod-" + string(pod.UID)), Tenant: &wf.TenantRef{Key: &wf.TenantRef_Id{Id: ID(1, pod.Namespace)}}, Manifest: m, Secrets: values}, session, 0))
			stop()
			if err != nil {
				failures++
				nextStart = time.Now().Add(time.Duration(min(60, 1<<min(failures, 6))) * time.Second)
				p.publish(pod, core.PodPending, "CreateContainerError", false, nil, int32(failures))
				continue
			}
			i = v.Msg
			continue // Recheck deletion/session before starting a newly added instance.
		}
		if e != nil {
			cancel()
			continue
		}
		if i.Phase == wf.Phase_PHASE_FAILED {
			if failedDeadline.IsZero() {
				failedDeadline = time.Now().Add(10 * time.Second)
			}
			if time.Now().Before(failedDeadline) {
				p.publish(pod, core.PodPending, "ExecutionBackoff", false, i, int32(failures))
				cancel()
				continue
			}
			// A failed sandbox is terminal. ReplicaSet replaces the Pod; its old
			// address remains quarantined until the failed Pod's gates have closed.
			p.publish(pod, core.PodFailed, "ExecutionFailed", false, i, int32(failures))
			_, e = p.Clients.Kube.CoreV1().Pods(pod.Namespace).Get(step, pod.Name, meta.GetOptions{})
			if e == nil {
				_ = p.Clients.Kube.CoreV1().Pods(pod.Namespace).Delete(step, pod.Name, meta.DeleteOptions{Preconditions: &meta.Preconditions{UID: Ptr(pod.UID)}})
			}
			cancel()
			continue
		}
		if i.Phase == wf.Phase_PHASE_STOPPED && !i.Draining {
			p.publish(pod, core.PodPending, "Starting", false, i, int32(failures))
			cancel()
			start, stop := context.WithTimeout(ctx, 2*time.Minute)
			_, _ = p.Runner.Instances.Start(start, request(&wf.InstanceStartRequest{Ref: instanceRef(id)}, session, i.Revision))
			stop()
			continue
		}
		if i.Phase == wf.Phase_PHASE_RUNNING && !i.Draining {
			if e = p.annotate(step, pod, i); e != nil {
				cancel()
				continue
			}
			if time.Now().Before(nextProbe) {
				cancel()
				continue
			}
			nextProbe = time.Now().Add(time.Duration(max(1, pod.Spec.Containers[0].ReadinessProbe.PeriodSeconds)) * time.Second)
			if e = probe(step, pod, i.GetEndpoint().GetAddress()); e == nil {
				successes++
				bad = 0
				if successes >= pod.Spec.Containers[0].ReadinessProbe.SuccessThreshold {
					healthy = true
				}
			} else {
				bad++
				if bad == 1 {
					slog.Warn("readiness failed", "pod", podKey(pod), "error", e)
				}
				successes = 0
				if bad >= pod.Spec.Containers[0].ReadinessProbe.FailureThreshold {
					healthy = false
				}
			}
			pod.Annotations[Endpoint] = i.GetEndpoint().GetAddress()
			reason := "Ready"
			if !healthy {
				reason = "ReadinessFailed"
			}
			p.publish(pod, core.PodRunning, reason, healthy, i, int32(failures))
		}
		cancel()
	}
}

func (p *Provider) Serve(ctx context.Context) error {
	p.ctx = ctx
	for {
		check, cancel := context.WithTimeout(ctx, 5*time.Second)
		r, e := p.Runner.Status(check)
		if e == nil {
			e = p.addressAvailable(check)
		}
		cancel()
		if e == nil && slices.Contains(r.Capabilities, "halt-instance-v1") {
			break
		}
		select {
		case <-ctx.Done():
			return ctx.Err()
		case <-time.After(time.Second):
		}
	}
	p.node = &core.Node{ObjectMeta: meta.ObjectMeta{Name: p.Name, Labels: map[string]string{Runtime: "wasmtime", "kubernetes.io/os": "linux", "kubernetes.io/arch": "wasm32", "node.kubernetes.io/exclude-from-external-load-balancers": "true"}, Annotations: map[string]string{Runner: p.Runner.URL}}, Spec: core.NodeSpec{Taints: []core.Taint{{Key: Runtime, Value: "wasmtime", Effect: core.TaintEffectNoSchedule}}}, Status: core.NodeStatus{Conditions: []core.NodeCondition{{Type: core.NodeReady, Status: core.ConditionTrue, Reason: "RunnerReady", LastHeartbeatTime: meta.Now(), LastTransitionTime: meta.Now()}}, NodeInfo: core.NodeSystemInfo{OperatingSystem: "linux", Architecture: "wasm32", KubeletVersion: "v1.37.1"}}}
	if physical := os.Getenv("POD_NAME"); physical != "" {
		pod, e := p.Clients.Kube.CoreV1().Pods(p.Namespace).Get(ctx, physical, meta.GetOptions{})
		if e != nil {
			return e
		}
		p.node.Annotations[RunnerPod] = p.Namespace + "/" + physical
		p.node.Annotations[Group+"/runnerUID"] = string(pod.UID)
	}
	r, e := p.Runner.Status(ctx)
	if e != nil {
		return e
	}
	p.node.Status.Capacity = core.ResourceList{core.ResourcePods: *resource.NewQuantity(int64(r.Capacity.Instances), resource.DecimalSI), Slots: *resource.NewQuantity(int64(r.Capacity.Running), resource.DecimalSI), Memory: *resource.NewQuantity(int64(r.Capacity.MemoryBytes), resource.DecimalSI)}
	p.node.Status.Allocatable = p.node.Status.Capacity.DeepCopy()
	factory := informers.NewSharedInformerFactoryWithOptions(p.Clients.Kube, 0, informers.WithTweakListOptions(func(o *meta.ListOptions) { o.FieldSelector = "spec.nodeName=" + p.Name }))
	other := informers.NewSharedInformerFactory(p.Clients.Kube, 0)
	broadcaster := record.NewBroadcaster()
	defer broadcaster.Shutdown()
	pc, e := node.NewPodController(node.PodControllerConfig{PodClient: p.Clients.Kube.CoreV1(), PodInformer: factory.Core().V1().Pods(), ConfigMapInformer: other.Core().V1().ConfigMaps(), SecretInformer: other.Core().V1().Secrets(), ServiceInformer: other.Core().V1().Services(), EventRecorder: broadcaster.NewRecorder(scheme.Scheme, core.EventSource{Component: "wafl"}), Provider: p, SkipDownwardAPIResolution: true})
	if e != nil {
		return e
	}
	nc, e := node.NewNodeController(p, p.node, p.Clients.Kube.CoreV1().Nodes(), node.WithNodeEnableLeaseV1(p.Clients.Kube.CoordinationV1().Leases(core.NamespaceNodeLease), 30), node.WithNodePingInterval(2*time.Second), node.WithNodePingTimeout(5*time.Second))
	if e != nil {
		return e
	}
	// Recover receipts before accepting new Pod work, including force-deleted Pods.
	items, e := p.Clients.Kube.CoreV1().ConfigMaps(p.Namespace).List(ctx, meta.ListOptions{LabelSelector: Kind + "=execution," + Group + "/node=" + p.Name})
	if e != nil {
		return e
	}
	for _, cm := range items.Items {
		var pod core.Pod
		if e = json.Unmarshal([]byte(cm.Data["pod"]), &pod); e != nil {
			return e
		}
		p.launch(&pod)
	}
	factory.Start(ctx.Done())
	errs := make(chan error, 2)
	go func() { errs <- nc.Run(ctx) }()
	go func() { errs <- pc.Run(ctx, 4) }()
	select {
	case e := <-errs:
		return e
	case <-ctx.Done():
		return nil
	}
}

// CNI may reuse a deleted Pod's IP. A fresh runner has a fresh journal, so it
// must not allocate sockets while any previous virtual node still owns that
// address. The controller removes that node only after its gates are closed.
func (p *Provider) addressAvailable(ctx context.Context) error {
	current, e := url.Parse(p.Runner.URL)
	if e != nil {
		return e
	}
	physicalUID := ""
	if physical := os.Getenv("POD_NAME"); physical != "" {
		pod, e := p.Clients.Kube.CoreV1().Pods(p.Namespace).Get(ctx, physical, meta.GetOptions{})
		if e != nil {
			return e
		}
		physicalUID = string(pod.UID)
	}
	nodes, e := p.Clients.Kube.CoreV1().Nodes().List(ctx, meta.ListOptions{LabelSelector: Runtime + "=wasmtime"})
	if e != nil {
		return e
	}
	for _, n := range nodes.Items {
		if n.Name == p.Name {
			if physicalUID != "" && n.Annotations[Group+"/runnerUID"] != physicalUID {
				return errors.New("virtual node belongs to a previous runner Pod")
			}
			continue
		}
		old, e := url.Parse(n.Annotations[Runner])
		if e == nil && old.Hostname() == current.Hostname() {
			return errors.New("runner address is quarantined by a previous virtual node")
		}
	}
	return nil
}

// Keep compile-time conformance to the actual Virtual Kubelet interfaces.
var _ node.PodLifecycleHandler = (*Provider)(nil)
var _ node.PodNotifier = (*Provider)(nil)
var _ node.NodeProvider = (*Provider)(nil)
