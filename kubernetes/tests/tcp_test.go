package tests

import (
	"bytes"
	"encoding/json"
	"fmt"
	"io"
	"net"
	"net/url"
	"os"
	"path/filepath"
	"strconv"
	"testing"
	"time"

	wf "github.com/lesomnus/wafl/kubernetes/gen/wafl"
	"github.com/lesomnus/wafl/kubernetes/internal/platform"
	"google.golang.org/protobuf/encoding/protojson"
	meta "k8s.io/apimachinery/pkg/apis/meta/v1"
)

func tcpConnect(address, greeting string) (*net.TCPConn, error) {
	conn, err := net.DialTimeout("tcp", address, 3*time.Second)
	if err != nil {
		return nil, err
	}
	c := conn.(*net.TCPConn)
	c.SetDeadline(time.Now().Add(40 * time.Second))
	buf := make([]byte, len(greeting))
	if _, err = io.ReadFull(c, buf); err != nil || string(buf) != greeting {
		c.Close()
		return nil, fmt.Errorf("server-first greeting %q, expected %q: %v", buf, greeting, err)
	}
	c.SetDeadline(time.Time{})
	return c, nil
}

func tcpFinish(t *testing.T, c *net.TCPConn) {
	t.Helper()
	defer c.Close()
	c.SetDeadline(time.Now().Add(10 * time.Second))
	data := bytes.Repeat([]byte{0, 128, 255, 13, 10}, 16_384)
	if _, err := c.Write(data); err != nil {
		t.Fatal(err)
	}
	if err := c.CloseWrite(); err != nil {
		t.Fatal(err)
	}
	response, err := io.ReadAll(c)
	if err != nil || !bytes.Equal(response, append(data, []byte("done\n")...)) {
		t.Fatalf("binary echo and response after half-close: %d bytes, %v", len(response), err)
	}
}

func (s *suite) tcp(t *testing.T) {
	t.Run("tcp_first_connection_rollout_scaling_and_half_close", func(t *testing.T) {
		s := *s
		s.t = t
		if s.cli == "" || os.Getenv("WAFL_TEST_TCP_PORT") == "" {
			t.Skip("run scripts/test-kubernetes.sh with its published TCP port pool")
		}
		base, err := strconv.Atoi(os.Getenv("WAFL_TEST_TCP_PORT"))
		if err != nil {
			t.Fatal(err)
		}
		host := "127.0.0.1"
		if u, err := url.Parse(os.Getenv("DOCKER_HOST")); err == nil && u.Scheme == "tcp" {
			host = u.Hostname()
		}
		address := func(n int) string { return net.JoinHostPort(host, strconv.Itoa(base+n)) }
		s.control = os.Getenv("WAFL_TEST_CONTROL")
		if s.control == "" {
			s.control = s.forward("svc/wafl-controller", 8082)
		}
		bundle := s.bundle("wafl-example-tcp")
		spec := map[string]any{
			"alias": "tcp",
			"manifest": map[string]any{
				"bundle":  map[string]any{"url": "file:" + s.localBundles[bundle["digest"].(string)]},
				"env":     map[string]any{"GREETING": "old", "FIN_DELAY_MS": "250"},
				"sockets": []any{map[string]any{"name": "echo", "port": 8080}, map[string]any{"name": "admin", "port": 8081}},
			},
			"listeners": []any{map[string]any{"socket": "echo", "port": 10000}, map[string]any{"socket": "admin", "port": 10001}},
			"scaling":   map[string]any{"min": 0, "max": 2, "target": 2, "upMs": 500, "downMs": 1000, "idleMs": 3000},
			"rollout":   map[string]any{"surge": 1, "drainMs": 30000},
		}
		file := filepath.Join(t.TempDir(), "app.yaml")
		data, _ := json.Marshal(spec)
		if err := os.WriteFile(file, data, 0600); err != nil {
			t.Fatal(err)
		}
		s.client("app", "add", file)
		a, err := s.c.Dynamic.Resource(platform.Apps).Namespace(s.ns).Get(s.ctx, "tcp", meta.GetOptions{})
		if err != nil {
			t.Fatal(err)
		}
		eventually(t, "TCP App registered at zero", 30*time.Second, func() bool {
			ds, err := s.c.Kube.AppsV1().Deployments(s.ns).List(s.ctx, meta.ListOptions{LabelSelector: platform.AppUID + "=" + string(a.GetUID())})
			return err == nil && len(ds.Items) == 1 && *ds.Items[0].Spec.Replicas == 0 && len(s.pods(a)) == 0
		})
		// No client bytes precede this greeting: server-first protocols must also
		// trigger activation. Real published NodePorts avoid port-forward's stream
		// abstraction, so FIN travels through the actual Pingora TCP path.
		old, err := tcpConnect(address(0), "old:8080\n")
		if err != nil {
			t.Fatal(err)
		}
		defer old.Close()
		oldPod := s.ready(a)
		second, err := tcpConnect(address(1), "old:8081\n")
		if err != nil {
			t.Fatal(err)
		}
		tcpFinish(t, second)
		time.Sleep(4 * time.Second)
		if len(s.pods(a)) != 1 {
			t.Fatal("idle open TCP connection did not keep exactly one execution")
		}
		// Roll out a new version while keeping the old connection alive.
		patch := filepath.Join(t.TempDir(), "patch.yaml")
		if err := os.WriteFile(patch, []byte("manifest:\n  env:\n    GREETING: new\n"), 0600); err != nil {
			t.Fatal(err)
		}
		s.client("app", "patch", "@tcp", "--file", patch)
		eventually(t, "old TCP execution drains", 45*time.Second, func() bool {
			p, err := s.c.Kube.CoreV1().Pods(s.ns).Get(s.ctx, oldPod.Name, meta.GetOptions{})
			return err == nil && p.DeletionTimestamp != nil
		})
		// Deletion requests route withdrawal; it is not an acknowledgement from
		// every proxy. Wait for its closed gate while the original connection
		// stays active, then require all new connections to use the replacement.
		eventually(t, "proxy acknowledged TCP withdrawal without dropping the stream", 10*time.Second, func() bool {
			records, err := s.c.Kube.CoreV1().ConfigMaps("wafl-system").List(s.ctx, meta.ListOptions{LabelSelector: platform.Kind + "=traffic," + platform.PodUID + "=" + string(oldPod.UID)})
			if err != nil || len(records.Items) == 0 {
				return false
			}
			active := uint32(0)
			for _, record := range records.Items {
				var observation wf.Traffic
				if protojson.Unmarshal([]byte(record.Data["value"]), &observation) != nil || !observation.Closed {
					return false
				}
				active += observation.Active
			}
			return active == 1
		})
		current, err := tcpConnect(address(0), "new:8080\n")
		if err != nil {
			t.Fatal(err)
		}
		defer current.Close()
		tcpFinish(t, old)
		s.gone(oldPod)
		// Existing Service and endpoint annotations publish both named ports.
		services, err := s.c.Kube.CoreV1().Services(s.ns).List(s.ctx, meta.ListOptions{LabelSelector: platform.AppUID + "=" + string(a.GetUID())})
		if err != nil || len(services.Items) != 1 || len(services.Items[0].Spec.Ports) != 2 {
			t.Fatal("TCP Service ports", err)
		}
		// Concurrent open connections count toward scaling and stop at max=2.
		connections := []*net.TCPConn{current}
		for n := 0; n < 5; n++ {
			conn, err := tcpConnect(address(n%2), fmt.Sprintf("new:%d\n", 8080+n%2))
			if err != nil {
				t.Fatal(err)
			}
			defer conn.Close()
			connections = append(connections, conn)
		}
		s.executions(a, 2)
		for _, conn := range connections {
			tcpFinish(t, conn)
		}
		s.executions(a, 0)
		restarted, err := tcpConnect(address(1), "new:8081\n")
		if err != nil {
			t.Fatal(err)
		}
		tcpFinish(t, restarted)
		// A fixed zero is suspension, not automatic activation.
		s.patch("tcp", map[string]any{"scaling": nil, "replicas": int64(0)})
		s.executions(a, 0)
		conn, err := net.DialTimeout("tcp", address(0), 3*time.Second)
		if err != nil {
			t.Fatal(err)
		}
		conn.SetReadDeadline(time.Now().Add(5 * time.Second))
		_, err = conn.Read(make([]byte, 1))
		conn.Close()
		if err == nil {
			t.Fatal("suspended App accepted a connection")
		}
		if timeout, ok := err.(net.Error); ok && timeout.Timeout() {
			t.Fatal("suspended connection was not closed")
		}
		if len(s.pods(a)) != 0 {
			t.Fatal("fixed-zero TCP App activated")
		}
		s.client("app", "erase", "@tcp")
	})
}
