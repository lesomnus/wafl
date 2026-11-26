package platform

import (
	"bytes"
	"context"
	"io"
	"net"
	"testing"
	"time"

	wf "github.com/lesomnus/wafl/kubernetes/gen/wafl"
)

func tcpPair(t *testing.T) (*net.TCPConn, *net.TCPConn) {
	t.Helper()
	l, err := net.ListenTCP("tcp", &net.TCPAddr{IP: net.IPv4(127, 0, 0, 1)})
	if err != nil {
		t.Fatal(err)
	}
	defer l.Close()
	client, err := net.DialTCP("tcp", nil, l.Addr().(*net.TCPAddr))
	if err != nil {
		t.Fatal(err)
	}
	server, err := l.AcceptTCP()
	if err != nil {
		t.Fatal(err)
	}
	t.Cleanup(func() { client.Close(); server.Close() })
	client.SetDeadline(time.Now().Add(5 * time.Second))
	server.SetDeadline(time.Now().Add(5 * time.Second))
	return client, server
}

func TestTCPRelayServerFirstBinaryAndHalfClose(t *testing.T) {
	client, front := tcpPair(t)
	back, server := tcpPair(t)
	done := make(chan struct{})
	go func() { relayTCP(context.Background(), front, back); close(done) }()
	if _, err := server.Write([]byte("hello\n")); err != nil {
		t.Fatal(err)
	}
	hello := make([]byte, 6)
	if _, err := io.ReadFull(client, hello); err != nil || string(hello) != "hello\n" {
		t.Fatal("server-first greeting", err)
	}
	data := bytes.Repeat([]byte{0, 255, 13, 10, 128}, 100_000)
	sent := make(chan error, 1)
	go func() {
		_, err := client.Write(data)
		if err == nil {
			err = client.CloseWrite()
		}
		sent <- err
	}()
	read, err := io.ReadAll(server)
	if err != nil || !bytes.Equal(read, data) {
		t.Fatal("client half-close/data", err)
	}
	if err := <-sent; err != nil {
		t.Fatal(err)
	}
	select {
	case <-done:
		t.Fatal("relay closed before server finished its response")
	default:
	}
	go func() {
		_, err := server.Write(data)
		if err == nil {
			err = server.CloseWrite()
		}
		sent <- err
	}()
	read, err = io.ReadAll(client)
	if err != nil || !bytes.Equal(read, data) {
		t.Fatal("response after client EOF", err)
	}
	if err := <-sent; err != nil {
		t.Fatal(err)
	}
	select {
	case <-done:
	case <-time.After(time.Second):
		t.Fatal("relay leaked after both EOFs")
	}
}

func TestTCPRelayShutdownClosesBothDirections(t *testing.T) {
	client, front := tcpPair(t)
	back, server := tcpPair(t)
	ctx, cancel := context.WithCancel(context.Background())
	done := make(chan struct{})
	go func() { relayTCP(ctx, front, back); close(done) }()
	cancel()
	select {
	case <-done:
	case <-time.After(time.Second):
		t.Fatal("relay blocked on shutdown")
	}
	for _, conn := range []*net.TCPConn{client, server} {
		if _, err := conn.Read(make([]byte, 1)); err != io.EOF {
			t.Fatalf("shutdown did not close socket: %v", err)
		}
	}
}

func TestTCPSocketContractAndGeneratedAppAPI(t *testing.T) {
	_, client := apiFixture(t)
	_, spec := fixture()
	spec.Manifest.Port, spec.Manifest.Protocol = 0, ""
	spec.Manifest.Sockets = []*wf.Socket{{Name: "echo", Port: 8080}, {Name: "admin", Port: 8081}}
	req := &wf.AppAddRequest{Alias: Ptr("echo"), Manifest: spec.Manifest, Listeners: []*wf.Listener{{Socket: "echo", Port: TCPFirst}, {Socket: "admin", Port: TCPFirst + 1}}}
	result, err := client.Add(context.Background(), appRequest(req))
	if err != nil {
		t.Fatal(err)
	}
	if result.Msg.Domain != "" || len(result.Msg.Listeners) != 2 || result.Msg.Manifest.Protocol != "tcp" {
		t.Fatal("TCP App lost its routing contract", result.Msg)
	}
	legacy := &wf.Manifest{Protocol: "tcp", Port: 9000}
	ss, err := sockets(legacy)
	if err != nil || len(ss) != 1 || ss[0].Name != "app" || ss[0].Protocol != "" {
		t.Fatal("legacy normalization", err)
	}
	for _, ss := range [][]*wf.Socket{
		{{Name: "echo", Port: 8000, Transport: "udp"}},
		{{Name: "echo", Port: 8000}, {Name: "echo", Port: 8001}},
		{{Name: "echo", Port: 8000}, {Name: "admin", Port: 8000}},
		{{Name: "", Port: 8000}},
	} {
		if _, err := sockets(&wf.Manifest{Sockets: ss}); err == nil {
			t.Fatal("accepted invalid sockets", ss)
		}
	}
	if err := validateListeners(&wf.App{Manifest: legacy, Domain: "tcp.example"}); err == nil {
		t.Fatal("raw TCP requires explicit public ports")
	}
	ports := map[uint32]bool{}
	domains := map[string]bool{}
	if !claimRoutes(result.Msg, domains, ports) || claimRoutes(result.Msg, domains, ports) {
		t.Fatal("public ports can be claimed twice")
	}
	if !claimRoutes(&wf.App{Domain: "web.example"}, domains, ports) {
		t.Fatal("TCP claim interfered with HTTP domain")
	}
}
