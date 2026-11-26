package platform

import (
	"context"
	"encoding/hex"
	"errors"
	"fmt"
	"io"
	"log/slog"
	"net"
	"net/url"
	"strconv"
	"sync"
	"sync/atomic"
	"time"

	wf "github.com/lesomnus/wafl/kubernetes/gen/wafl"
	core "k8s.io/api/core/v1"
	apierrors "k8s.io/apimachinery/pkg/api/errors"
	meta "k8s.io/apimachinery/pkg/apis/meta/v1"
	"k8s.io/apimachinery/pkg/apis/meta/v1/unstructured"
	"k8s.io/client-go/util/retry"
)

var tcpSequence atomic.Uint64

func (c *Controller) tcpApp(ctx context.Context, port uint32) (*unstructured.Unstructured, *wf.App, string, error) {
	list, err := c.Clients.activeApps(ctx)
	if err != nil {
		return nil, nil, "", err
	}
	orderApps(list.Items)
	domains, ports := map[string]bool{}, map[uint32]bool{}
	for i := range list.Items {
		a := &list.Items[i]
		spec, err := decodeApp(a)
		if err != nil || !claimRoutes(spec, domains, ports) {
			continue
		}
		for _, listener := range spec.Listeners {
			if listener.Port == port {
				return a, spec, listener.Socket, nil
			}
		}
	}
	return nil, nil, "", errors.New("TCP listener is not assigned")
}

// The front proxy only selects a public port. Each accepted connection is
// registered durably before its target Pod is re-read. This is the same drain
// fence as HTTP activation, and works for server-first and encrypted protocols.
func (c *Controller) tcpTarget(ctx context.Context, a *unstructured.Unstructured, spec *wf.App, socket string) (string, func(), error) {
	if spec.Scaling == nil && spec.Replicas == 0 {
		return "", func() {}, errors.New("App is suspended")
	}
	id := randomID()
	started := time.Now()
	observer := &wf.Traffic{Id: id, App: &wf.App{Id: ID(10, string(a.GetUID()))}, Session: hex.EncodeToString(id), Activation: true, Active: 1, Started: 1, DateSeen: uint64(started.UnixMilli())}
	cm := &core.ConfigMap{ObjectMeta: meta.ObjectMeta{Name: trafficName(id), Namespace: c.Namespace, Labels: map[string]string{Managed: "true", Kind: "traffic", AppUID: string(a.GetUID())}}, Data: map[string]string{"value": trafficData(observer)}}
	if _, err := c.Clients.Kube.CoreV1().ConfigMaps(c.Namespace).Create(ctx, cm, meta.CreateOptions{}); err != nil {
		return "", func() {}, err
	}
	done := make(chan struct{})
	completed := make(chan struct{})
	var once sync.Once
	finish := func() {
		once.Do(func() { close(done) })
		select {
		case <-completed:
		case <-time.After(5 * time.Second):
		}
	}
	go func() {
		defer close(completed)
		ticker := time.NewTicker(500 * time.Millisecond)
		defer ticker.Stop()
		closed := false
		deadline := time.Time{}
		for {
			if !closed {
				select {
				case <-done:
					closed = true
					deadline = time.Now().Add(time.Minute)
				case <-ticker.C:
				}
			}
			step, cancel := context.WithTimeout(context.Background(), 5*time.Second)
			_, err := c.Traffic.update(step, id, func(_ *core.ConfigMap, v *wf.Traffic) error {
				v.Sequence++
				v.DateSeen = uint64(time.Now().UnixMilli())
				v.BusyMs = uint64(time.Since(started).Milliseconds())
				if closed {
					v.Active = 0
					v.Closed = true
				}
				return nil
			})
			cancel()
			if closed && (err == nil || time.Now().After(deadline)) {
				if err != nil {
					slog.Error("TCP observer closure needs recovery", "observer", UUID(id), "error", err)
				}
				return
			}
			if closed {
				time.Sleep(time.Second)
			}
		}
	}()
	fail := func(err error) (string, func(), error) { finish(); return "", func() {}, err }
	limit, wait := uint32(128), 30*time.Second
	if spec.Scaling != nil {
		limit = spec.Scaling.Waiting
		wait = time.Duration(spec.Scaling.WaitMs) * time.Millisecond
	}
	queue, cancel := context.WithTimeout(ctx, wait)
	defer cancel()
	queued, err := c.Clients.Kube.CoreV1().ConfigMaps(c.Namespace).List(queue, meta.ListOptions{LabelSelector: Kind + "=traffic," + AppUID + "=" + string(a.GetUID())})
	if err != nil {
		return fail(err)
	}
	waiting := 0
	for i := range queued.Items {
		v, e := trafficValue(&queued.Items[i])
		if e == nil && v.Activation && !v.Closed && queued.Items[i].Labels[PodUID] == "" {
			waiting++
		}
	}
	if waiting > int(limit) {
		return fail(errors.New("TCP activation queue full"))
	}
	if spec.Scaling != nil {
		err = retry.RetryOnConflict(retry.DefaultBackoff, func() error {
			api := c.Clients.Kube.AppsV1().Deployments(a.GetNamespace())
			d, e := api.Get(queue, appName(a), meta.GetOptions{})
			if e != nil {
				return e
			}
			if d.DeletionTimestamp != nil {
				return errors.New("App is deleting")
			}
			if *d.Spec.Replicas > 0 {
				return nil
			}
			d.Spec.Replicas = Ptr(int32(max(1, spec.Scaling.Min)))
			_, e = api.Update(queue, d, meta.UpdateOptions{})
			return e
		})
		if err != nil {
			return fail(err)
		}
	}
	offset := tcpSequence.Add(1)
	for {
		fresh, e := c.Clients.Dynamic.Resource(Apps).Namespace(a.GetNamespace()).Get(queue, a.GetName(), meta.GetOptions{})
		if apierrors.IsNotFound(e) || e == nil && (fresh.GetUID() != a.GetUID() || fresh.GetDeletionTimestamp() != nil) {
			return fail(errors.New("App was removed"))
		}
		if e != nil {
			return fail(e)
		}
		if e == nil {
			latest, e := decodeApp(fresh)
			if e != nil {
				return fail(e)
			}
			if latest.Scaling == nil && latest.Replicas == 0 {
				return fail(errors.New("App is suspended"))
			}
		}
		pods, e := c.Clients.Kube.CoreV1().Pods(a.GetNamespace()).List(queue, meta.ListOptions{LabelSelector: AppUID + "=" + string(a.GetUID())})
		if e == nil {
			for j := range pods.Items {
				p := &pods.Items[(uint64(j)+offset)%uint64(len(pods.Items))]
				address := socketAddress(p, socket)
				if !ready(p) || address == "" {
					continue
				}
				_, e = c.Traffic.update(queue, id, func(cm *core.ConfigMap, v *wf.Traffic) error {
					cm.Labels[PodUID] = string(p.UID)
					cm.Data["pod"] = p.Name
					cm.Data["namespace"] = p.Namespace
					v.Placement = &wf.Placement{Id: placementID(p)}
					return nil
				})
				if e != nil {
					continue
				}
				current, e := c.Clients.Kube.CoreV1().Pods(p.Namespace).Get(queue, p.Name, meta.GetOptions{})
				if e == nil && current.UID == p.UID && ready(current) && socketAddress(current, socket) == address {
					return address, finish, nil
				}
				// No connection has been opened. Remove the rejected candidate binding;
				// it must not hold an unrelated retiring Pod or disappear from queue counts.
				_, _ = c.Traffic.update(queue, id, func(cm *core.ConfigMap, v *wf.Traffic) error {
					delete(cm.Labels, PodUID)
					delete(cm.Data, "pod")
					delete(cm.Data, "namespace")
					v.Placement = nil
					return nil
				})
			}
		}
		select {
		case <-queue.Done():
			return fail(queue.Err())
		case <-time.After(100 * time.Millisecond):
		}
	}
}

// Copy both directions independently: an EOF closes only the peer's write half.
// A reset/error or shutdown closes both directions. No payload sniffing,
// whole-message buffering, replay, or retry after the upstream dial succeeds.
func relayTCP(ctx context.Context, client, backend *net.TCPConn) {
	defer client.Close()
	defer backend.Close()
	stop := context.AfterFunc(ctx, func() { client.Close(); backend.Close() })
	defer stop()
	result := make(chan error, 2)
	copyHalf := func(dst, src *net.TCPConn) { _, err := io.Copy(dst, src); _ = dst.CloseWrite(); result <- err }
	go copyHalf(backend, client)
	go copyHalf(client, backend)
	if err := <-result; err != nil {
		client.Close()
		backend.Close()
	}
	<-result
}

func (c *Controller) tcpConnection(ctx context.Context, client *net.TCPConn, port uint32) {
	defer client.Close()
	stop := context.AfterFunc(ctx, func() { client.Close() })
	defer stop()
	lookup, cancel := context.WithTimeout(ctx, 5*time.Second)
	app, spec, socket, err := c.tcpApp(lookup, port)
	cancel()
	if err != nil {
		return
	}
	address, finish, err := c.tcpTarget(ctx, app, spec, socket)
	if err != nil {
		return
	}
	defer finish()
	backend, err := (&net.Dialer{Timeout: 5 * time.Second}).DialContext(ctx, "tcp", address)
	if err != nil {
		return
	}
	relayTCP(ctx, client, backend.(*net.TCPConn))
}

func (c *Controller) serveTCP(ctx context.Context) (func(), error) {
	ctx, cancel := context.WithCancel(ctx)
	listeners := []*net.TCPListener{}
	var accepts, connections sync.WaitGroup
	closeAll := func() {
		cancel()
		for _, l := range listeners {
			_ = l.Close()
		}
		accepts.Wait() // No further connection Add can race with Wait below.
		done := make(chan struct{})
		go func() { connections.Wait(); close(done) }()
		select {
		case <-done:
		case <-time.After(6 * time.Second):
			slog.Error("TCP shutdown could not confirm all observer closures")
		}
	}
	// A bounded set is installed on the controller Service and Traefik as well.
	// Admission is bounded per controller; per-App waiting is persisted above.
	budget := make(chan struct{}, 1024)
	for port := TCPFirst; port <= TCPLast; port++ {
		listener, err := net.ListenTCP("tcp", &net.TCPAddr{Port: port})
		if err != nil {
			closeAll()
			return nil, fmt.Errorf("TCP listener %d: %w", port, err)
		}
		listeners = append(listeners, listener)
	}
	for index, listener := range listeners {
		port := uint32(TCPFirst + index)
		accepts.Add(1)
		go func() {
			defer accepts.Done()
			for {
				client, err := listener.AcceptTCP()
				if err != nil {
					return
				}
				select {
				case budget <- struct{}{}:
					connections.Add(1)
					go func() { defer connections.Done(); defer func() { <-budget }(); c.tcpConnection(ctx, client, port) }()
				default:
					_ = client.Close()
				}
			}
		}()
	}
	return closeAll, nil
}

func (c *Controller) tcpConfiguration() map[string]any {
	routers, services := map[string]any{}, map[string]any{}
	origin, err := url.Parse(c.Activate)
	if err != nil || origin.Hostname() == "" {
		return nil
	}
	// Stable front routes do not keep stale App identities. The receiving relay
	// resolves the current App at accept time, and fences its UID through startup.
	for port := TCPFirst; port <= TCPLast; port++ {
		name := "tcp-" + strconv.Itoa(port)
		routers[name] = map[string]any{"entryPoints": []string{name}, "rule": "HostSNI(`*`)", "service": name}
		services[name] = map[string]any{"loadBalancer": map[string]any{"serversTransport": "wafl-tcp", "servers": []any{map[string]string{"address": net.JoinHostPort(origin.Hostname(), strconv.Itoa(port))}}}}
	}
	// Traefik's default 100ms termination deadline truncates protocols that send
	// a response after the client's write-half EOF. The App drain deadline bounds
	// retirement; an ordinary live half-closed connection may finish naturally.
	return map[string]any{"routers": routers, "services": services, "serversTransports": map[string]any{"wafl-tcp": map[string]any{"terminationDelay": "-1ms"}}}
}
