package platform

import (
	"context"
	"net/http"
	"strconv"

	"connectrpc.com/connect"
	wf "github.com/lesomnus/wafl/kubernetes/gen/wafl"
)

func commandHeaders(header http.Header) error {
	session, e := catalogSession(header)
	if e != nil {
		return e
	}
	values := header.Values("wafl-revision")
	if len(values) > 1 {
		return catalogError(connect.CodeInvalidArgument, "duplicate wafl-revision")
	}
	if len(values) == 1 {
		if len(session) == 0 {
			return catalogError(connect.CodeInvalidArgument, "wafl-revision requires wafl-session")
		}
		value := values[0]
		for _, c := range value {
			if c < '0' || c > '9' {
				return catalogError(connect.CodeInvalidArgument, "invalid wafl-revision")
			}
		}
		if _, e := strconv.ParseUint(value, 10, 64); e != nil {
			return catalogError(connect.CodeInvalidArgument, "invalid wafl-revision")
		}
	}
	return nil
}
func forwardRequest[T any](value *T, header http.Header, session []byte) *connect.Request[T] {
	r := request(value, session, 0)
	if value := header.Get("wafl-revision"); value != "" {
		r.Header().Set("wafl-revision", value)
	}
	return r
}
func selectedRunner(ctx context.Context, c *Catalog, h http.Header) (observation[*wf.Runner], *catalogResult[*wf.Runner], error) {
	var zero observation[*wf.Runner]
	if e := commandHeaders(h); e != nil {
		return zero, nil, e
	}
	session, e := catalogSession(h)
	if e != nil {
		return zero, nil, e
	}
	if len(session) == 0 {
		return zero, nil, catalogError(connect.CodeInvalidArgument, "this operation requires wafl-session to select a receiving runner")
	}
	result, e := collect(ctx, c, h, "Runner.Select", func(_ context.Context, _ member, r *wf.Runner) ([]*wf.Runner, error) { return []*wf.Runner{r}, nil })
	if e != nil {
		return zero, nil, e
	}
	item, e := unique(result)
	return item, result, e
}

// Mutation routing resolves once, forwards exactly once, and pins the receiving
// process. It never refreshes a revision/token or retries on another process.
func instanceCommand[T any](ctx context.Context, a *InstanceAPI, h http.Header, ref *wf.InstanceRef, call func(*RunnerClient, []byte, *wf.InstanceRef) (*connect.Response[T], error)) (*connect.Response[T], error) {
	if e := commandHeaders(h); e != nil {
		return nil, e
	}
	if ref != nil && ref.GetAlias() != "" && h.Get("wafl-session") == "" {
		return nil, catalogError(connect.CodeInvalidArgument, "alias mutations through a controller require wafl-session")
	}
	result, e := a.locate(ctx, h, ref)
	if e != nil {
		return nil, e
	}
	item, e := unique(result)
	if e != nil {
		return nil, e
	}
	response, e := call(a.Catalog.client(item.member), item.runner.Id, instanceRef(item.value.Id))
	if e != nil {
		return nil, e
	}
	if value, ok := any(response.Msg).(*wf.Instance); ok {
		value.Secrets = nil
	}
	return observedResponse(response.Msg, result), nil
}
func (a *InstanceAPI) Add(ctx context.Context, r *connect.Request[wf.InstanceAddRequest]) (*connect.Response[wf.Instance], error) {
	item, result, e := selectedRunner(ctx, a.Catalog, r.Header())
	if e != nil {
		return nil, e
	}
	response, e := a.Catalog.client(item.member).Instances.Add(ctx, forwardRequest(r.Msg, r.Header(), item.runner.Id))
	if e != nil {
		return nil, e
	}
	response.Msg.Secrets = nil
	return observedResponse(response.Msg, result), nil
}
func (a *InstanceAPI) Start(ctx context.Context, r *connect.Request[wf.InstanceStartRequest]) (*connect.Response[wf.Instance], error) {
	return instanceCommand(ctx, a, r.Header(), r.Msg.Ref, func(c *RunnerClient, s []byte, ref *wf.InstanceRef) (*connect.Response[wf.Instance], error) {
		m := clone(r.Msg)
		m.Ref = ref
		return c.Instances.Start(ctx, forwardRequest(m, r.Header(), s))
	})
}
func (a *InstanceAPI) Stop(ctx context.Context, r *connect.Request[wf.InstanceStopRequest]) (*connect.Response[wf.Instance], error) {
	return instanceCommand(ctx, a, r.Header(), r.Msg.Ref, func(c *RunnerClient, s []byte, ref *wf.InstanceRef) (*connect.Response[wf.Instance], error) {
		m := clone(r.Msg)
		m.Ref = ref
		return c.Instances.Stop(ctx, forwardRequest(m, r.Header(), s))
	})
}
func (a *InstanceAPI) Halt(ctx context.Context, r *connect.Request[wf.InstanceStopRequest]) (*connect.Response[wf.Instance], error) {
	return instanceCommand(ctx, a, r.Header(), r.Msg.Ref, func(c *RunnerClient, s []byte, ref *wf.InstanceRef) (*connect.Response[wf.Instance], error) {
		m := clone(r.Msg)
		m.Ref = ref
		return c.Instances.Halt(ctx, forwardRequest(m, r.Header(), s))
	})
}
func (a *InstanceAPI) Drain(ctx context.Context, r *connect.Request[wf.InstanceDrainRequest]) (*connect.Response[wf.Instance], error) {
	return instanceCommand(ctx, a, r.Header(), r.Msg.Ref, func(c *RunnerClient, s []byte, ref *wf.InstanceRef) (*connect.Response[wf.Instance], error) {
		m := clone(r.Msg)
		m.Ref = ref
		return c.Instances.Drain(ctx, forwardRequest(m, r.Header(), s))
	})
}
func (a *InstanceAPI) Release(ctx context.Context, r *connect.Request[wf.InstanceReleaseRequest]) (*connect.Response[wf.Instance], error) {
	if r.Header().Get("wafl-session") == "" || r.Header().Get("wafl-revision") == "" {
		return nil, catalogError(connect.CodeInvalidArgument, "Release requires wafl-session and wafl-revision")
	}
	return instanceCommand(ctx, a, r.Header(), r.Msg.Ref, func(c *RunnerClient, s []byte, ref *wf.InstanceRef) (*connect.Response[wf.Instance], error) {
		m := clone(r.Msg)
		m.Ref = ref
		return c.Instances.Release(ctx, forwardRequest(m, r.Header(), s))
	})
}
func (a *InstanceAPI) Erase(ctx context.Context, r *connect.Request[wf.InstanceRef]) (*connect.Response[wf.InstanceEraseResponse], error) {
	response, e := instanceCommand(ctx, a, r.Header(), r.Msg, func(c *RunnerClient, s []byte, ref *wf.InstanceRef) (*connect.Response[wf.InstanceEraseResponse], error) {
		return c.Instances.Erase(ctx, forwardRequest(ref, r.Header(), s))
	})
	if connect.CodeOf(e) == connect.CodeNotFound {
		return connect.NewResponse(&wf.InstanceEraseResponse{Erased: Ptr(false)}), nil
	}
	return response, e
}
func (a *InstanceAPI) Seal(ctx context.Context, r *connect.Request[wf.InstanceSealRequest]) (*connect.Response[wf.InstanceSealResponse], error) {
	item, result, e := selectedRunner(ctx, a.Catalog, r.Header())
	if e != nil {
		return nil, e
	}
	response, e := a.Catalog.client(item.member).Instances.Seal(ctx, forwardRequest(r.Msg, r.Header(), item.runner.Id))
	if e != nil {
		return nil, e
	}
	return observedResponse(response.Msg, result), nil
}
func (a *ReservationAPI) Release(ctx context.Context, r *connect.Request[wf.ReservationReleaseRequest]) (*connect.Response[wf.ReservationReleaseResponse], error) {
	if !validReservationRef(r.Msg.Ref) {
		return nil, invalidCatalog()
	}
	// The session selects the receiving runner, never the reservation's dead
	// owner. Preserve the caller's token and withdrawal confirmation verbatim.
	item, result, e := selectedRunner(ctx, a.Catalog, r.Header())
	if e != nil {
		return nil, e
	}
	response, e := a.Catalog.client(item.member).Reservations.Release(ctx, forwardRequest(r.Msg, r.Header(), item.runner.Id))
	if e != nil {
		return nil, e
	}
	return observedResponse(response.Msg, result), nil
}
