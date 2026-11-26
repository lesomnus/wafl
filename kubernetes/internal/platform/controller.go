package platform

import (
	"context"
	"encoding/json"
	"errors"
	"fmt"
	"log/slog"
	"net/http"
	"reflect"
	"sort"
	"strings"
	"time"

	"connectrpc.com/connect"
	"github.com/lesomnus/wafl/kubernetes/gen/wafl/waflconnect"
	apps "k8s.io/api/apps/v1"
	core "k8s.io/api/core/v1"
	discovery "k8s.io/api/discovery/v1"
	apierrors "k8s.io/apimachinery/pkg/api/errors"
	meta "k8s.io/apimachinery/pkg/apis/meta/v1"
	"k8s.io/apimachinery/pkg/apis/meta/v1/unstructured"
	"k8s.io/apimachinery/pkg/util/intstr"
	"k8s.io/client-go/tools/leaderelection"
	"k8s.io/client-go/tools/leaderelection/resourcelock"
)

type Controller struct {
	Clients                                                                              *Clients
	Namespace, Listen, ActivationListen, Control, Activate, Entrypoint, Identity, Assets string
	Traffic                                                                              *Traffic
	RunnerTimeout                                                                        time.Duration
	RunnerParallel                                                                       int
}

func (c *Controller) reconcile(ctx context.Context) error {
	if e := c.reap(ctx); e != nil {
		return e
	}
	list, e := c.Clients.activeApps(ctx)
	if e != nil {
		return e
	}
	orderApps(list.Items)
	domains := map[string]bool{}
	ports := map[uint32]bool{}
	for index := range list.Items {
		a := &list.Items[index]
		if a.GetDeletionTimestamp() != nil {
			continue
		}
		spec, err := decodeApp(a)
		var d *apps.Deployment
		if err != nil {
			e = err
		} else if !claimRoutes(spec, domains, ports) {
			e = errors.New("domain or TCP listener is already assigned to another App")
		} else {
			d, _, e = c.Clients.reconcileDeployment(ctx, a)
		}
		if e != nil {
			_ = c.status(ctx, a, nil, e)
			continue
		}
		if e = c.endpoints(ctx, a, d); e != nil {
			slog.Error("publish endpoints", "app", a.GetName(), "error", e)
		}
		if e = c.scale(ctx, a, d); e != nil {
			slog.Error("scale", "app", a.GetName(), "error", e)
		}
		_ = c.status(ctx, a, d, nil)
	}
	return c.collectSnapshots(ctx)
}
func orderApps(items []unstructured.Unstructured) {
	sort.Slice(items, func(i, j int) bool {
		a, b := items[i].GetCreationTimestamp(), items[j].GetCreationTimestamp()
		if a.Equal(&b) {
			return string(items[i].GetUID()) < string(items[j].GetUID())
		}
		return a.Before(&b)
	})
}
func (c *Controller) status(ctx context.Context, a *unstructured.Unstructured, d *apps.Deployment, err error) error {
	status := map[string]any{"observedGeneration": a.GetGeneration(), "desired": int64(0), "ready": int64(0), "revision": int64(0)}
	accepted, readyValue, reason, message := "True", "False", "Progressing", "waiting for ready executions"
	if err != nil {
		accepted, reason, message = "False", "Rejected", err.Error()
	} else if d != nil {
		status["deployment"], status["service"] = d.Name, d.Name
		status["desired"] = int64(*d.Spec.Replicas)
		status["ready"] = int64(d.Status.ReadyReplicas)
		status["revision"] = d.Generation
		if d.Status.ObservedGeneration >= d.Generation && d.Status.UpdatedReplicas == *d.Spec.Replicas && d.Status.AvailableReplicas >= *d.Spec.Replicas {
			readyValue, reason, message = "True", "Available", "desired executions are available"
			if *d.Spec.Replicas == 0 {
				reason, message = "ScaledToZero", "first request activates the App"
				if _, found, _ := unstructured.NestedMap(a.Object, "spec", "scaling"); !found {
					reason, message = "Suspended", "App has zero fixed replicas"
				}
			}
		}
	}
	conditions := []any{}
	for _, v := range []struct{ kind, value string }{{"Accepted", accepted}, {"Ready", readyValue}} {
		date := time.Now().UTC().Format(time.RFC3339)
		old, _, _ := unstructured.NestedSlice(a.Object, "status", "conditions")
		for _, entry := range old {
			m := entry.(map[string]any)
			if m["type"] == v.kind && m["status"] == v.value && m["reason"] == reason {
				date, _ = m["lastTransitionTime"].(string)
			}
		}
		conditions = append(conditions, map[string]any{"type": v.kind, "status": v.value, "reason": reason, "message": message, "observedGeneration": a.GetGeneration(), "lastTransitionTime": date})
	}
	status["conditions"] = conditions
	old, _ := json.Marshal(a.Object["status"])
	next, _ := json.Marshal(status)
	if string(old) == string(next) {
		return nil
	}
	copy := a.DeepCopy()
	copy.Object["status"] = status
	_, e := c.Clients.Dynamic.Resource(Apps).Namespace(a.GetNamespace()).UpdateStatus(ctx, copy, meta.UpdateOptions{})
	return e
}
func (c *Controller) endpoints(ctx context.Context, a *unstructured.Unstructured, d *apps.Deployment) error {
	service := &core.Service{ObjectMeta: meta.ObjectMeta{Name: d.Name, Namespace: a.GetNamespace(), Labels: map[string]string{Managed: "true", AppUID: string(a.GetUID())}, OwnerReferences: []meta.OwnerReference{owner(a)}}, Spec: core.ServiceSpec{Ports: []core.ServicePort{{Name: "app", Port: 80, TargetPort: intstr.FromInt(80)}}}}
	spec, err := decodeApp(a)
	if err != nil {
		return err
	}
	if len(spec.Listeners) > 0 {
		service.Spec.Ports = nil
		ss, _ := sockets(spec.Manifest)
		for _, listener := range spec.Listeners {
			for _, socket := range ss {
				if socket.Name == listener.Socket {
					service.Spec.Ports = append(service.Spec.Ports, core.ServicePort{Name: socket.Name, Port: int32(socket.Port), TargetPort: intstr.FromInt32(int32(listener.Port))})
				}
			}
		}
	}
	services := c.Clients.Kube.CoreV1().Services(a.GetNamespace())
	_, e := services.Create(ctx, service, meta.CreateOptions{})
	if apierrors.IsAlreadyExists(e) {
		old, err := services.Get(ctx, service.Name, meta.GetOptions{})
		if err != nil {
			return err
		}
		for i := range service.Spec.Ports {
			service.Spec.Ports[i].Protocol = core.ProtocolTCP
		}
		if !reflect.DeepEqual(old.Spec.Ports, service.Spec.Ports) {
			old.Spec.Ports = service.Spec.Ports
			_, e = services.Update(ctx, old, meta.UpdateOptions{})
		} else {
			e = nil
		}
	}
	if e != nil {
		return e
	}
	// ClusterIP clients use the same gateway admission as external requests.
	// HTTP uses its plugin; TCP uses the connection relay on the controller.
	pods, e := c.Clients.Kube.CoreV1().Pods(c.Namespace).List(ctx, meta.ListOptions{LabelSelector: "app.kubernetes.io/component=traefik,app.kubernetes.io/part-of=wafl"})
	if e != nil {
		return e
	}
	wanted := map[string]bool{}
	for index := range pods.Items {
		p := &pods.Items[index]
		if p.DeletionTimestamp != nil || p.Status.PodIP == "" {
			continue
		}
		healthy := false
		for _, condition := range p.Status.Conditions {
			if condition.Type == core.PodReady && condition.Status == core.ConditionTrue {
				healthy = true
			}
		}
		if !healthy {
			continue
		}
		host := p.Status.PodIP
		number := int32(8080)
		name := d.Name + "-" + digest(string(p.UID))[:12]
		wanted[name] = true
		slice := &discovery.EndpointSlice{ObjectMeta: meta.ObjectMeta{Name: name, Namespace: a.GetNamespace(), Labels: map[string]string{discovery.LabelServiceName: d.Name, discovery.LabelManagedBy: Group, AppUID: string(a.GetUID())}, OwnerReferences: []meta.OwnerReference{owner(a)}}, AddressType: discovery.AddressTypeIPv4, Ports: []discovery.EndpointPort{{Name: Ptr("app"), Port: Ptr(int32(number)), Protocol: Ptr(core.ProtocolTCP)}}, Endpoints: []discovery.Endpoint{{Addresses: []string{host}, Conditions: discovery.EndpointConditions{Ready: Ptr(true)}, TargetRef: &core.ObjectReference{Kind: "Pod", Namespace: p.Namespace, Name: p.Name, UID: p.UID}}}}
		if len(spec.Listeners) > 0 {
			slice.Ports = nil
			for _, listener := range spec.Listeners {
				slice.Ports = append(slice.Ports, discovery.EndpointPort{Name: Ptr(listener.Socket), Port: Ptr(int32(listener.Port)), Protocol: Ptr(core.ProtocolTCP)})
			}
		}
		api := c.Clients.Kube.DiscoveryV1().EndpointSlices(a.GetNamespace())
		old, e := api.Get(ctx, name, meta.GetOptions{})
		if apierrors.IsNotFound(e) {
			_, e = api.Create(ctx, slice, meta.CreateOptions{})
		} else if e == nil {
			slice.ResourceVersion = old.ResourceVersion
			if !reflect.DeepEqual(old.Endpoints, slice.Endpoints) || !reflect.DeepEqual(old.Ports, slice.Ports) {
				_, e = api.Update(ctx, slice, meta.UpdateOptions{})
			}
		}
		if e != nil {
			return e
		}
	}
	slices, e := c.Clients.Kube.DiscoveryV1().EndpointSlices(a.GetNamespace()).List(ctx, meta.ListOptions{LabelSelector: AppUID + "=" + string(a.GetUID())})
	if e != nil {
		return e
	}
	for _, s := range slices.Items {
		if !wanted[s.Name] {
			_ = c.Clients.Kube.DiscoveryV1().EndpointSlices(a.GetNamespace()).Delete(ctx, s.Name, meta.DeleteOptions{})
		}
	}
	return nil
}
func (c *Controller) configuration(ctx context.Context) (map[string]any, error) {
	apps, e := c.Clients.activeApps(ctx)
	if e != nil {
		return nil, e
	}
	orderApps(apps.Items)
	pods, e := c.Clients.Kube.CoreV1().Pods("").List(ctx, meta.ListOptions{LabelSelector: Managed + "=true"})
	if e != nil {
		return nil, e
	}
	routers, services, middlewares := map[string]any{}, map[string]any{}, map[string]any{}
	domains := map[string]bool{}
	ports := map[uint32]bool{}
	for index := range apps.Items {
		a := &apps.Items[index]
		if a.GetDeletionTimestamp() != nil {
			continue
		}
		spec, e := decodeApp(a)
		if e != nil || !claimRoutes(spec, domains, ports) {
			continue
		}
		if spec.Manifest.Protocol == "tcp" {
			continue
		}
		name := appName(a)
		hosts := []string{spec.Domain, name, name + "." + a.GetNamespace(), name + "." + a.GetNamespace() + ".svc", name + "." + a.GetNamespace() + ".svc.cluster.local"}
		rules := []string{}
		for _, host := range hosts {
			rules = append(rules, fmt.Sprintf("Host(`%s`)", host))
		}
		routers[name] = map[string]any{"rule": strings.Join(rules, " || "), "entryPoints": []string{c.Entrypoint}, "service": name}
		backends := []any{}
		for j := range pods.Items {
			p := &pods.Items[j]
			if p.Labels[AppUID] != string(a.GetUID()) || !ready(p) {
				continue
			}
			leaf := "wafl-pod-" + string(p.UID)
			middlewares[leaf] = map[string]any{"plugin": map[string]any{"wafl": map[string]any{"control": c.Control, "activate": c.Activate, "placement": B64(placementID(p)), "intervalMs": 500}}}
			services[leaf] = map[string]any{"loadBalancer": map[string]any{"servers": []any{map[string]string{"url": endpointURL(p)}}, "passHostHeader": true}, "middlewares": []string{leaf}}
			backends = append(backends, map[string]any{"name": leaf, "weight": 1})
		}
		if len(backends) > 0 {
			services[name] = map[string]any{"weighted": map[string]any{"services": backends}}
		} else {
			servers := []any{}
			// Fixed replicas can also temporarily have no ready execution during
			// runner replacement. Queue them through the same bounded activator.
			if spec.Scaling != nil || spec.Replicas > 0 {
				url := c.Activate
				if spec.Manifest.Protocol == "h2c" {
					url = strings.Replace(url, "http://", "h2c://", 1)
				}
				servers = append(servers, map[string]string{"url": url})
			}
			services[name] = map[string]any{"loadBalancer": map[string]any{"servers": servers, "passHostHeader": true}}
		}
	}
	if len(routers) == 0 {
		if tcp := c.tcpConfiguration(); tcp != nil {
			return map[string]any{"tcp": tcp}, nil
		}
		return map[string]any{}, nil
	}
	httpConfig := map[string]any{"routers": routers, "services": services}
	if len(middlewares) > 0 {
		httpConfig["middlewares"] = middlewares
	}
	result := map[string]any{"http": httpConfig}
	if tcp := c.tcpConfiguration(); tcp != nil {
		result["tcp"] = tcp
	}
	return result, nil
}
func (c *Controller) Serve(ctx context.Context) error {
	ctx, cancel := context.WithCancel(ctx)
	defer cancel()
	if c.RunnerTimeout < 0 || c.RunnerParallel < 0 || c.RunnerParallel > 128 {
		return errors.New("runner query timeout must be nonnegative and parallelism must be between 0 and 128; zero uses the default")
	}
	c.Traffic = &Traffic{Clients: c.Clients, Namespace: c.Namespace}
	closeTCP, err := c.serveTCP(ctx)
	if err != nil {
		return err
	}
	defer closeTCP()
	catalog := &Catalog{Clients: c.Clients, Namespace: c.Namespace, Timeout: c.RunnerTimeout, Parallel: c.RunnerParallel}
	mux := http.NewServeMux()
	mux.Handle(waflconnect.NewRunnerServiceHandler(&RunnerAPI{Catalog: catalog}))
	mux.Handle(waflconnect.NewInstanceServiceHandler(&InstanceAPI{Catalog: catalog}, connect.WithReadMaxBytes(2<<20)))
	mux.Handle(waflconnect.NewReservationServiceHandler(&ReservationAPI{Catalog: catalog}))
	mux.Handle(waflconnect.NewTrafficServiceHandler(c.Traffic))
	mux.Handle(waflconnect.NewAppServiceHandler(&AppAPI{Clients: c.Clients}, connect.WithReadMaxBytes(2<<20)))
	mux.HandleFunc("PUT /bundles/{digest}", c.upload)
	mux.HandleFunc("GET /health", func(w http.ResponseWriter, r *http.Request) {
		check, cancel := context.WithTimeout(r.Context(), time.Second)
		defer cancel()
		if _, e := c.Clients.Kube.CoordinationV1().Leases(c.Namespace).Get(check, "wafl-controller", meta.GetOptions{}); e != nil {
			http.Error(w, "Kubernetes control state unavailable", 503)
			return
		}
		w.WriteHeader(200)
	})
	mux.HandleFunc("GET /traefik", func(w http.ResponseWriter, r *http.Request) {
		v, e := c.configuration(r.Context())
		if e != nil {
			http.Error(w, "configuration unavailable", 503)
			return
		}
		w.Header().Set("Content-Type", "application/json")
		w.Header().Set("Cache-Control", "no-store")
		_ = json.NewEncoder(w).Encode(v)
	})
	server := &http.Server{Addr: c.Listen, Handler: mux, ReadHeaderTimeout: 10 * time.Second, Protocols: new(http.Protocols)}
	server.Protocols.SetHTTP1(true)
	server.Protocols.SetUnencryptedHTTP2(true)
	activation := &http.Server{Addr: c.ActivationListen, Handler: http.HandlerFunc(c.activate), ReadHeaderTimeout: 10 * time.Second, Protocols: server.Protocols}
	errors := make(chan error, 2)
	go func() { errors <- server.ListenAndServe() }()
	go func() { errors <- activation.ListenAndServe() }()
	defer server.Close()
	defer activation.Close()
	go leaderelection.RunOrDie(ctx, leaderelection.LeaderElectionConfig{Lock: &resourcelock.LeaseLock{LeaseMeta: meta.ObjectMeta{Name: "wafl-controller", Namespace: c.Namespace}, Client: c.Clients.Kube.CoordinationV1(), LockConfig: resourcelock.ResourceLockConfig{Identity: c.Identity}}, LeaseDuration: 15 * time.Second, RenewDeadline: 10 * time.Second, RetryPeriod: 2 * time.Second, ReleaseOnCancel: true, Callbacks: leaderelection.LeaderCallbacks{
		OnStartedLeading: func(ctx context.Context) {
			ticker := time.NewTicker(time.Second)
			defer ticker.Stop()
			for {
				if e := c.reconcile(ctx); e != nil {
					slog.Error("reconcile", "error", e)
				}
				select {
				case <-ctx.Done():
					return
				case <-ticker.C:
				}
			}
		}, OnStoppedLeading: func() {
			if ctx.Err() == nil {
				errors <- fmt.Errorf("controller leadership lost")
			}
		}}})
	select {
	case <-ctx.Done():
		return nil
	case e := <-errors:
		return e
	}
}
