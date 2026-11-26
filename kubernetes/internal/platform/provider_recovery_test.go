package platform

import (
	"context"
	"net/http"
	"net/http/httptest"
	"sync/atomic"
	"testing"
	"time"

	"connectrpc.com/connect"
	wf "github.com/lesomnus/wafl/kubernetes/gen/wafl"
	"github.com/lesomnus/wafl/kubernetes/gen/wafl/waflconnect"
	core "k8s.io/api/core/v1"
	meta "k8s.io/apimachinery/pkg/apis/meta/v1"
	"k8s.io/client-go/kubernetes/fake"
)

type pagedRecovery struct {
	waflconnect.UnimplementedReservationServiceHandler
	current, previous, instance []byte
	fail                        bool
	pages                       atomic.Int32
	releases                    chan capturedCommand
}

func (s *pagedRecovery) List(_ context.Context, r *connect.Request[wf.ReservationListRequest]) (*connect.Response[wf.ReservationListResponse], error) {
	if r.Header().Get("wafl-session") != UUID(s.current) {
		return nil, catalogError(connect.CodeAborted, "session changed")
	}
	s.pages.Add(1)
	if r.Msg.After == "" {
		// A full page of other executions precedes the sockets to be recovered.
		items := []*wf.Reservation{}
		for n := range 100 {
			items = append(items, &wf.Reservation{Id: catalogID(9, byte(n)), Instance: ID(7, "another-pod"), Session: s.previous})
		}
		return connect.NewResponse(&wf.ReservationListResponse{Items: items, Next: "second"}), nil
	}
	if s.fail {
		return nil, catalogError(connect.CodeUnavailable, "next page unavailable")
	}
	if r.Msg.After != "second" {
		return nil, catalogError(connect.CodeInvalidArgument, "invalid cursor")
	}
	return connect.NewResponse(&wf.ReservationListResponse{Items: []*wf.Reservation{
		{Id: catalogID(9, 101), Instance: s.instance, Session: s.previous, Token: "socket-one"},
		{Id: catalogID(9, 102), Instance: s.instance, Session: s.previous, Token: "socket-two"},
		{Id: catalogID(9, 103), Instance: s.instance, Session: s.current, Token: "new-session"},
	}}), nil
}

func (s *pagedRecovery) Release(_ context.Context, r *connect.Request[wf.ReservationReleaseRequest]) (*connect.Response[wf.ReservationReleaseResponse], error) {
	s.releases <- capturedCommand{session: r.Header().Get("wafl-session"), id: r.Msg.Ref.GetId(), token: r.Msg.Token, withdrawn: r.Msg.Withdrawn}
	return connect.NewResponse(&wf.ReservationReleaseResponse{Released: true}), nil
}

func TestRecoveryReadsEveryReservationPageAndPreservesOtherOwners(t *testing.T) {
	for _, fail := range []bool{false, true} {
		t.Run(map[bool]string{false: "complete", true: "incomplete"}[fail], func(t *testing.T) {
			pod := &core.Pod{ObjectMeta: meta.ObjectMeta{Name: "old", Namespace: "demo", UID: "previous-pod"}}
			s := &pagedRecovery{current: ID(8, "new"), previous: ID(8, "old"), instance: podID(pod), fail: fail, releases: make(chan capturedCommand, 3)}
			mux := http.NewServeMux()
			mux.Handle(waflconnect.NewRunnerServiceHandler(&catalogRunnerStub{g: &catalogGuest{runner: &wf.Runner{Id: s.current}}}))
			mux.Handle(waflconnect.NewReservationServiceHandler(s))
			server := httptest.NewServer(mux)
			defer server.Close()
			clients := &Clients{Kube: fake.NewClientset()}
			p := NewProvider(clients, "node", server.URL, "system")
			done, err := p.cleanup(context.Background(), pod, s.previous, time.Time{})
			if s.pages.Load() != 2 {
				t.Fatal("did not read the second reservation page")
			}
			if fail {
				if done || err == nil || len(s.releases) != 0 {
					t.Fatal("incomplete list was acknowledged as cleaned up")
				}
				return
			}
			if !done || err != nil || len(s.releases) != 2 {
				t.Fatalf("cleanup: %v, %v, releases=%d", done, err, len(s.releases))
			}
			for _, token := range []string{"socket-one", "socket-two"} {
				command := <-s.releases
				if command.token != token || command.session != UUID(s.current) || !command.withdrawn {
					t.Fatal("release lost its session/allocation fence")
				}
			}
		})
	}
}
