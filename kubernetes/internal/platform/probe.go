package platform

import (
	"bytes"
	"context"
	"encoding/binary"
	"errors"
	"io"
	"net"
	"net/http"
	"strings"
	"time"

	wf "github.com/lesomnus/wafl/kubernetes/gen/wafl"
	"google.golang.org/protobuf/encoding/protowire"
	core "k8s.io/api/core/v1"
)

var h2transport = func() *http.Transport {
	t := http.DefaultTransport.(*http.Transport).Clone()
	t.Protocols = new(http.Protocols)
	t.Protocols.SetUnencryptedHTTP2(true)
	return t
}()

func probe(ctx context.Context, pod *core.Pod, address string) error {
	p := pod.Spec.Containers[0].ReadinessProbe
	ctx, cancel := context.WithTimeout(ctx, time.Duration(max(1, p.TimeoutSeconds))*time.Second)
	defer cancel()
	if p.HTTPGet != nil {
		r, e := http.NewRequestWithContext(ctx, http.MethodGet, "http://"+address+p.HTTPGet.Path, nil)
		if e != nil {
			return e
		}
		client := &http.Client{CheckRedirect: func(*http.Request, []*http.Request) error { return http.ErrUseLastResponse }}
		response, e := client.Do(r)
		if e != nil {
			return e
		}
		defer response.Body.Close()
		_, _ = io.Copy(io.Discard, io.LimitReader(response.Body, 4096))
		if response.StatusCode < 200 || response.StatusCode >= 400 {
			return errors.New("HTTP readiness failed")
		}
		return nil
	}
	if p.GRPC != nil {
		message := []byte{}
		if p.GRPC.Service != nil && *p.GRPC.Service != "" {
			message = protowire.AppendTag(message, 1, protowire.BytesType)
			message = protowire.AppendString(message, *p.GRPC.Service)
		}
		body := make([]byte, 5)
		binary.BigEndian.PutUint32(body[1:], uint32(len(message)))
		body = append(body, message...)
		r, e := http.NewRequestWithContext(ctx, http.MethodPost, "http://"+address+"/grpc.health.v1.Health/Check", bytes.NewReader(body))
		if e != nil {
			return e
		}
		r.Header.Set("Content-Type", "application/grpc")
		r.Header.Set("TE", "trailers")
		response, e := (&http.Client{Transport: h2transport}).Do(r)
		if e != nil {
			return e
		}
		defer response.Body.Close()
		data, e := io.ReadAll(io.LimitReader(response.Body, 65537))
		if e != nil {
			return e
		}
		if response.StatusCode != 200 || !strings.HasPrefix(response.Header.Get("Content-Type"), "application/grpc") || response.Trailer.Get("Grpc-Status") != "0" && response.Header.Get("Grpc-Status") != "0" || len(data) < 5 || data[0] != 0 || int(binary.BigEndian.Uint32(data[1:5])) != len(data)-5 {
			return errors.New("invalid gRPC health response")
		}
		data = data[5:]
		for len(data) > 0 {
			n, typ, size := protowire.ConsumeTag(data)
			if size < 0 {
				return errors.New("invalid gRPC health message")
			}
			data = data[size:]
			if n == 1 && typ == protowire.VarintType {
				value, size := protowire.ConsumeVarint(data)
				if size < 0 || value != 1 {
					return errors.New("gRPC service is not serving")
				}
				return nil
			}
			size = protowire.ConsumeFieldValue(n, typ, data)
			if size < 0 {
				return errors.New("invalid gRPC health field")
			}
			data = data[size:]
		}
		return errors.New("gRPC status missing")
	}
	c, e := (&net.Dialer{}).DialContext(ctx, "tcp", address)
	if e == nil {
		_ = c.Close()
	}
	return e
}

func probeInstance(ctx context.Context, pod *core.Pod, i *wf.Instance) error {
	if len(i.Endpoints) == 0 || pod.Spec.Containers[0].ReadinessProbe.TCPSocket == nil {
		return probe(ctx, pod, i.GetEndpoint().GetAddress())
	}
	for _, endpoint := range i.Endpoints {
		if err := probe(ctx, pod, endpoint.Address); err != nil {
			return err
		}
	}
	return nil
}
