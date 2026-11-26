package platform

import (
	"context"
	"crypto/rand"
	"encoding/hex"
	"errors"
	"fmt"
	"net/http"
	"net/http/httputil"
	"net/url"
	"strconv"
	"strings"
	"time"

	wf "github.com/lesomnus/wafl/kubernetes/gen/wafl"
	core "k8s.io/api/core/v1"
	meta "k8s.io/apimachinery/pkg/apis/meta/v1"
	"k8s.io/apimachinery/pkg/apis/meta/v1/unstructured"
	"k8s.io/client-go/util/retry"
)

func randomID() []byte {
	b := make([]byte, 16)
	if _, e := rand.Read(b); e != nil {
		panic(e)
	}
	b[6] = (b[6] & 15) | 0x80
	b[8] = 0x80
	b[9] = 13
	return b
}
func (c *Controller) app(ctx context.Context, host string) (*unstructured.Unstructured, *wf.App, error) {
	if u, e := url.Parse("http://" + host); e == nil {
		host = strings.ToLower(u.Hostname())
	}
	list, e := c.Clients.activeApps(ctx)
	if e != nil {
		return nil, nil, e
	}
	orderApps(list.Items)
	for i := range list.Items {
		a := &list.Items[i]
		if a.GetDeletionTimestamp() != nil {
			continue
		}
		domain, _, _ := unstructured.NestedString(a.Object, "spec", "domain")
		service := appName(a) + "." + a.GetNamespace() + ".svc"
		if host == domain || host == appName(a) || host == appName(a)+"."+a.GetNamespace() || host == service || host == service+".cluster.local" {
			s, e := decodeApp(a)
			return a, s, e
		}
	}
	return nil, nil, errors.New("App not found")
}
func activationError(w http.ResponseWriter, r *http.Request, status int, message string) {
	if strings.HasPrefix(r.Header.Get("Content-Type"), "application/grpc") {
		code := "14"
		if status == http.StatusGatewayTimeout {
			code = "4"
		}
		if status == http.StatusTooManyRequests {
			code = "8"
		}
		w.Header().Set("Content-Type", "application/grpc")
		w.Header().Set("Grpc-Status", code)
		w.Header().Set("Grpc-Message", message)
		w.WriteHeader(200)
		return
	}
	http.Error(w, message, status)
}
func grpcTimeout(value string) (time.Duration, error) {
	if len(value) < 2 || len(value) > 9 {
		return 0, errors.New("invalid grpc-timeout")
	}
	n, e := strconv.ParseUint(value[:len(value)-1], 10, 32)
	if e != nil {
		return 0, e
	}
	units := map[byte]time.Duration{'H': time.Hour, 'M': time.Minute, 'S': time.Second, 'm': time.Millisecond, 'u': time.Microsecond, 'n': time.Nanosecond}
	u, ok := units[value[len(value)-1]]
	if !ok {
		return 0, errors.New("invalid grpc-timeout")
	}
	if n > uint64((1<<63-1)/int64(u)) {
		return time.Duration(1<<63 - 1), nil
	}
	return time.Duration(n) * u, nil
}
func (c *Controller) activate(w http.ResponseWriter, r *http.Request) {
	if r.Method == http.MethodConnect || r.Header.Get("Upgrade") != "" {
		activationError(w, r, 501, "protocol upgrade is unsupported")
		return
	}
	a, spec, e := c.app(r.Context(), r.Host)
	if e != nil {
		activationError(w, r, 404, "App unavailable")
		return
	}
	ctx := r.Context()
	cancel := func() {}
	if v := r.Header.Get("Grpc-Timeout"); v != "" {
		d, e := grpcTimeout(v)
		if e != nil {
			activationError(w, r, 400, "invalid grpc-timeout")
			return
		}
		ctx, cancel = context.WithTimeout(ctx, d)
	}
	defer cancel()
	id := randomID()
	started := time.Now()
	observer := &wf.Traffic{Id: id, App: &wf.App{Id: ID(10, string(a.GetUID()))}, Session: hex.EncodeToString(id), Activation: true, Active: 1, Started: 1, DateSeen: uint64(started.UnixMilli())}
	cm := &core.ConfigMap{ObjectMeta: meta.ObjectMeta{Name: trafficName(id), Namespace: c.Namespace, Labels: map[string]string{Managed: "true", Kind: "traffic", AppUID: string(a.GetUID())}}, Data: map[string]string{"value": trafficData(observer)}}
	if _, e = c.Clients.Kube.CoreV1().ConfigMaps(c.Namespace).Create(ctx, cm, meta.CreateOptions{}); e != nil {
		activationError(w, r, 503, "activation unavailable")
		return
	}
	finish := make(chan struct{})
	defer close(finish)
	go func() {
		ticker := time.NewTicker(500 * time.Millisecond)
		defer ticker.Stop()
		done := false
		for {
			if !done {
				select {
				case <-finish:
					done = true
				case <-ticker.C:
				}
			}
			step, cancel := context.WithTimeout(context.Background(), 5*time.Second)
			_, e := c.Traffic.update(step, id, func(_ *core.ConfigMap, v *wf.Traffic) error {
				v.Sequence++
				v.BusyMs = uint64(time.Since(started).Milliseconds())
				v.DateSeen = uint64(time.Now().UnixMilli())
				if done {
					v.Active = 0
					v.Closed = true
				}
				return nil
			})
			cancel()
			if done && e == nil {
				return
			}
			if done {
				time.Sleep(time.Second)
			}
		}
	}()
	limit := uint32(128)
	wait := 30 * time.Second
	if spec.Scaling != nil {
		limit = spec.Scaling.Waiting
		wait = time.Duration(spec.Scaling.WaitMs) * time.Millisecond
	}
	queued, e := c.Clients.Kube.CoreV1().ConfigMaps(c.Namespace).List(ctx, meta.ListOptions{LabelSelector: Kind + "=traffic," + AppUID + "=" + string(a.GetUID())})
	if e != nil {
		activationError(w, r, 503, "activation unavailable")
		return
	}
	waiting := 0
	for i := range queued.Items {
		v, e := trafficValue(&queued.Items[i])
		if e == nil && v.Activation && !v.Closed && queued.Items[i].Labels[PodUID] == "" {
			waiting++
		}
	}
	if waiting > int(limit) {
		activationError(w, r, 429, "activation queue is full")
		return
	}
	if spec.Scaling != nil {
		e = retry.RetryOnConflict(retry.DefaultBackoff, func() error {
			api := c.Clients.Kube.AppsV1().Deployments(a.GetNamespace())
			d, e := api.Get(ctx, appName(a), meta.GetOptions{})
			if e != nil {
				return e
			}
			if *d.Spec.Replicas > 0 {
				return nil
			}
			d.Spec.Replicas = Ptr(int32(max(1, spec.Scaling.Min)))
			_, e = api.Update(ctx, d, meta.UpdateOptions{})
			return e
		})
		if e != nil {
			activationError(w, r, 503, "activation unavailable")
			return
		}
	}
	queue, queueCancel := context.WithTimeout(ctx, wait)
	defer queueCancel()
	var selected *core.Pod
	for {
		pods, e := c.Clients.Kube.CoreV1().Pods(a.GetNamespace()).List(queue, meta.ListOptions{LabelSelector: AppUID + "=" + string(a.GetUID())})
		if e == nil {
			for i := range pods.Items {
				p := &pods.Items[i]
				if !ready(p) {
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
				fresh, e := c.Clients.Kube.CoreV1().Pods(p.Namespace).Get(queue, p.Name, meta.GetOptions{})
				if e == nil && fresh.UID == p.UID && ready(fresh) {
					selected = fresh
					break
				}
			}
		}
		if selected != nil {
			break
		}
		if spec.Scaling == nil && spec.Replicas == 0 {
			activationError(w, r, 503, "App has zero fixed replicas")
			return
		}
		select {
		case <-queue.Done():
			activationError(w, r, 504, "App startup deadline exceeded")
			return
		case <-time.After(100 * time.Millisecond):
		}
	}
	address, e := url.Parse("http://" + selected.Annotations[Endpoint])
	if e != nil {
		activationError(w, r, 503, "invalid execution endpoint")
		return
	}
	proxy := httputil.NewSingleHostReverseProxy(address)
	original := proxy.Director
	proxy.Director = func(out *http.Request) {
		host := out.Host
		original(out)
		out.Host = host
		out.Header.Del("Wafl-Instance")
		if deadline, ok := ctx.Deadline(); ok && strings.HasPrefix(out.Header.Get("Content-Type"), "application/grpc") {
			out.Header.Set("Grpc-Timeout", fmt.Sprint(max(1, time.Until(deadline).Milliseconds()))+"m")
		}
	}
	if spec.Manifest.Protocol == "h2c" {
		proxy.Transport = h2transport
	}
	proxy.FlushInterval = -1
	proxy.ErrorHandler = func(w http.ResponseWriter, r *http.Request, _ error) {
		activationError(w, r, 503, "execution unavailable")
	}
	proxy.ServeHTTP(w, r.WithContext(ctx))
}
