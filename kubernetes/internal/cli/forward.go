package cli

import (
	"context"
	"fmt"
	"io"
	"net/http"
	"strings"

	"github.com/lesomnus/wafl/kubernetes/internal/platform"
	"k8s.io/client-go/tools/portforward"
	"k8s.io/client-go/transport/spdy"
)

func forward(ctx context.Context, c *platform.Clients, pod string, port int) (string, func(), error) {
	parts := strings.SplitN(pod, "/", 2)
	if len(parts) != 2 {
		return "", nil, fmt.Errorf("invalid runner Pod reference")
	}
	u := c.Kube.CoreV1().RESTClient().Post().Namespace(parts[0]).Resource("pods").Name(parts[1]).SubResource("portforward").URL()
	transport, upgrader, e := spdy.RoundTripperFor(c.Config)
	if e != nil {
		return "", nil, e
	}
	dialer := spdy.NewDialer(upgrader, &http.Client{Transport: transport}, http.MethodPost, u)
	stop := make(chan struct{})
	ready := make(chan struct{})
	f, e := portforward.New(dialer, []string{fmt.Sprintf("0:%d", port)}, stop, ready, io.Discard, io.Discard)
	if e != nil {
		return "", nil, e
	}
	done := make(chan error, 1)
	go func() { done <- f.ForwardPorts() }()
	select {
	case <-ctx.Done():
		close(stop)
		return "", nil, ctx.Err()
	case e := <-done:
		close(stop)
		return "", nil, e
	case <-ready:
	}
	ports, e := f.GetPorts()
	if e != nil {
		close(stop)
		return "", nil, e
	}
	return fmt.Sprintf("http://127.0.0.1:%d", ports[0].Local), func() { close(stop) }, nil
}
