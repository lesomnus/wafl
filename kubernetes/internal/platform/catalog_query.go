package platform

import (
	"bytes"
	"context"
	"net/http"
	"net/netip"
	"regexp"
	"slices"

	"connectrpc.com/connect"
	wf "github.com/lesomnus/wafl/kubernetes/gen/wafl"
	"github.com/lesomnus/wafl/kubernetes/gen/wafl/waflconnect"
	"google.golang.org/protobuf/proto"
	"k8s.io/apimachinery/pkg/util/validation"
)

type RunnerAPI struct{ Catalog *Catalog }
type InstanceAPI struct{ Catalog *Catalog }
type ReservationAPI struct{ Catalog *Catalog }

var instanceAlias = regexp.MustCompile(`^[a-z][a-z0-9-]{0,62}$`)

func validInstanceRef(ref *wf.InstanceRef) bool {
	if ref == nil {
		return false
	}
	switch v := ref.Key.(type) {
	case *wf.InstanceRef_Id:
		return validID(v.Id, 7)
	case *wf.InstanceRef_Alias:
		return instanceAlias.MatchString(v.Alias)
	}
	return false
}
func validRunnerRef(ref *wf.RunnerRef) bool {
	if ref == nil {
		return false
	}
	switch v := ref.Key.(type) {
	case *wf.RunnerRef_Id:
		return validID(v.Id, 8)
	case *wf.RunnerRef_Alias:
		return len(validation.IsDNS1123Subdomain(v.Alias)) == 0 && v.Alias != "self"
	}
	return false
}
func validReservationRef(ref *wf.ReservationRef) bool {
	if ref == nil {
		return false
	}
	switch v := ref.Key.(type) {
	case *wf.ReservationRef_Id:
		return validID(v.Id, 9)
	case *wf.ReservationRef_Address:
		p, e := netip.ParseAddrPort(v.Address)
		return e == nil && p.Addr().Is4() && p.Port() != 0
	}
	return false
}
func runnerMatches(ref *wf.RunnerRef, v *wf.Runner) bool {
	switch r := ref.Key.(type) {
	case *wf.RunnerRef_Id:
		return bytes.Equal(r.Id, v.Id)
	case *wf.RunnerRef_Alias:
		return r.Alias == v.Alias
	}
	return false
}
func instanceMatches(ref *wf.InstanceRef, v *wf.Instance) bool {
	switch r := ref.Key.(type) {
	case *wf.InstanceRef_Id:
		return bytes.Equal(r.Id, v.Id)
	case *wf.InstanceRef_Alias:
		return r.Alias == v.Alias
	}
	return false
}
func reservationMatches(ref *wf.ReservationRef, v *wf.Reservation) bool {
	switch r := ref.Key.(type) {
	case *wf.ReservationRef_Id:
		return bytes.Equal(r.Id, v.Id)
	case *wf.ReservationRef_Address:
		return r.Address == v.Address
	}
	return false
}
func invalidCatalog() error {
	return catalogError(connect.CodeInvalidArgument, "invalid reference or filter; controller runner aliases name virtual nodes, not self")
}

func (a *RunnerAPI) Get(ctx context.Context, r *connect.Request[wf.RunnerGetRequest]) (*connect.Response[wf.Runner], error) {
	if !validRunnerRef(r.Msg.Ref) {
		return nil, invalidCatalog()
	}
	result, e := collect(ctx, a.Catalog, r.Header(), "Runner.Get", func(_ context.Context, _ member, v *wf.Runner) ([]*wf.Runner, error) {
		if runnerMatches(r.Msg.Ref, v) {
			return []*wf.Runner{v}, nil
		}
		return nil, nil
	})
	if e != nil {
		return nil, e
	}
	item, e := unique(result)
	if e != nil {
		return nil, e
	}
	v := item.value
	if r.Msg.Select != nil {
		selectFields(v.ProtoReflect(), r.Msg.Select.ProtoReflect())
	}
	return observedResponse(v, result), nil
}
func (a *RunnerAPI) List(ctx context.Context, r *connect.Request[wf.RunnerListRequest]) (*connect.Response[wf.RunnerListResponse], error) {
	m := r.Msg
	if len(m.Filters) > 64 {
		return nil, invalidCatalog()
	}
	for _, f := range m.Filters {
		if f == nil || !validRunnerRef(f.Ref) {
			return nil, invalidCatalog()
		}
	}
	p, e := newCatalogPage(a.Catalog.Namespace, "runner", r.Header(), m.Filters, m.Size, m.After, m.Reverse, 8)
	if e != nil {
		return nil, e
	}
	result, e := collect(ctx, a.Catalog, r.Header(), "Runner.List", func(_ context.Context, _ member, v *wf.Runner) ([]*wf.Runner, error) {
		match := len(m.Filters) == 0
		for _, f := range m.Filters {
			match = match || runnerMatches(f.Ref, v)
		}
		if match {
			return []*wf.Runner{v}, nil
		}
		return nil, nil
	})
	if e != nil {
		return nil, e
	}
	values, next := pageItems(p, result.items, false)
	return observedResponse(&wf.RunnerListResponse{Items: values, Next: next}, result), nil
}

func (a *InstanceAPI) locate(ctx context.Context, h http.Header, ref *wf.InstanceRef) (*catalogResult[*wf.Instance], error) {
	if !validInstanceRef(ref) {
		return nil, invalidCatalog()
	}
	return collect(ctx, a.Catalog, h, "Instance.Get", func(ctx context.Context, node member, runner *wf.Runner) ([]*wf.Instance, error) {
		v, e := a.Catalog.client(node).Instances.Get(ctx, request(&wf.InstanceGetRequest{Ref: ref}, runner.Id, 0))
		if connect.CodeOf(e) == connect.CodeNotFound {
			return nil, nil
		}
		if e != nil {
			return nil, e
		}
		if !validID(v.Msg.Id, 7) || !instanceMatches(ref, v.Msg) || !bytes.Equal(v.Msg.Session, runner.Id) {
			return nil, catalogError(connect.CodeAborted, "runner session changed during query")
		}
		v.Msg.Secrets = nil
		return []*wf.Instance{v.Msg}, nil
	})
}
func (a *InstanceAPI) Get(ctx context.Context, r *connect.Request[wf.InstanceGetRequest]) (*connect.Response[wf.Instance], error) {
	if v := r.Msg.GetSelect().GetTenant(); v != nil && (v.GetAll() || v.GetAlias() || v.GetName() || v.GetDesc() || v.GetLabels() || v.GetDateUpdated() || v.GetDateCreated()) {
		return nil, catalogError(connect.CodeUnimplemented, "runner holds tenant references, not tenant metadata")
	}
	result, e := a.locate(ctx, r.Header(), r.Msg.Ref)
	if e != nil {
		return nil, e
	}
	item, e := unique(result)
	if e != nil {
		return nil, e
	}
	v := item.value
	if r.Msg.Select != nil {
		selectFields(v.ProtoReflect(), r.Msg.Select.ProtoReflect())
	}
	return observedResponse(v, result), nil
}
func (a *InstanceAPI) List(ctx context.Context, r *connect.Request[wf.InstanceListRequest]) (*connect.Response[wf.InstanceListResponse], error) {
	m := r.Msg
	if len(m.Filters) > 64 {
		return nil, invalidCatalog()
	}
	for _, f := range m.Filters {
		if f == nil || !validInstanceRef(f.Ref) {
			return nil, invalidCatalog()
		}
	}
	p, e := newCatalogPage(a.Catalog.Namespace, "instance", r.Header(), m.Filters, m.Size, m.After, m.Reverse, 7)
	if e != nil {
		return nil, e
	}
	result, e := collect(ctx, a.Catalog, r.Header(), "Instance.List", func(ctx context.Context, node member, runner *wf.Runner) ([]*wf.Instance, error) {
		return scan(ctx, p, runner.Id, 7, func(v *wf.Instance) bool {
			match := len(m.Filters) == 0
			for _, f := range m.Filters {
				match = match || instanceMatches(f.Ref, v)
			}
			return match
		}, func(after string) ([]*wf.Instance, string, error) {
			page, e := a.Catalog.client(node).Instances.List(ctx, request(&wf.InstanceListRequest{Size: int32(min(p.size+1, 1000)), After: after, Reverse: m.Reverse}, runner.Id, 0))
			if e != nil {
				return nil, "", e
			}
			for _, v := range page.Msg.Items {
				if !bytes.Equal(v.GetSession(), runner.Id) {
					return nil, "", catalogError(connect.CodeAborted, "runner session changed during query")
				}
				v.Secrets = nil
			}
			return page.Msg.Items, page.Msg.Next, nil
		})
	})
	if e != nil {
		return nil, e
	}
	values, next := pageItems(p, result.items, true)
	return observedResponse(&wf.InstanceListResponse{Items: values, Next: next}, result), nil
}
func (a *InstanceAPI) Tail(ctx context.Context, r *connect.Request[wf.InstanceTailRequest], out *connect.ServerStream[wf.Log]) error {
	result, e := a.locate(ctx, r.Header(), r.Msg.Ref)
	if e != nil {
		return e
	}
	item, e := unique(result)
	if e != nil {
		return e
	}
	// The short discovery timeout must not become the lifetime of a follow stream.
	msg := clone(r.Msg)
	msg.Ref = instanceRef(item.value.Id)
	stream, e := a.Catalog.client(item.member).Instances.Tail(ctx, request(msg, item.runner.Id, 0))
	if e != nil {
		return e
	}
	defer stream.Close()
	warningHeaders(out.ResponseHeader(), result.warnings)
	out.ResponseHeader().Set("wafl-session", UUID(item.runner.Id))
	for stream.Receive() {
		if e = out.Send(stream.Msg()); e != nil {
			return e
		}
	}
	return stream.Err()
}

func (a *ReservationAPI) locate(ctx context.Context, h http.Header, ref *wf.ReservationRef) (*catalogResult[*wf.Reservation], error) {
	if !validReservationRef(ref) {
		return nil, invalidCatalog()
	}
	return collect(ctx, a.Catalog, h, "Reservation.Get", func(ctx context.Context, node member, runner *wf.Runner) ([]*wf.Reservation, error) {
		if !slices.Contains(runner.Capabilities, "reservation-recovery-v1") {
			return nil, nil
		}
		// Get(address) on another pool is INVALID_ARGUMENT, so locate through its
		// generated List contract instead. An absent allocation is a normal miss.
		p := catalogPage{size: 0}
		values, e := scan(ctx, p, nil, 9, func(v *wf.Reservation) bool { return reservationMatches(ref, v) }, func(after string) ([]*wf.Reservation, string, error) {
			page, e := a.Catalog.client(node).Reservations.List(ctx, request(&wf.ReservationListRequest{After: after, Size: 1000}, runner.Id, 0))
			if e != nil {
				return nil, "", e
			}
			return page.Msg.Items, page.Msg.Next, nil
		})
		return values, e
	})
}
func (a *ReservationAPI) Get(ctx context.Context, r *connect.Request[wf.ReservationGetRequest]) (*connect.Response[wf.Reservation], error) {
	result, e := a.locate(ctx, r.Header(), r.Msg.Ref)
	if e != nil {
		return nil, e
	}
	if len(result.items) == 0 {
		return nil, missing(result)
	}
	v := result.items[0].value
	for _, item := range result.items[1:] {
		if !proto.Equal(v, item.value) {
			return nil, catalogError(connect.CodeAborted, "reservation observations differ; retry with wafl-session")
		}
	}
	if r.Msg.Select != nil {
		selectFields(v.ProtoReflect(), r.Msg.Select.ProtoReflect())
	}
	return observedResponse(v, result), nil
}
func (a *ReservationAPI) List(ctx context.Context, r *connect.Request[wf.ReservationListRequest]) (*connect.Response[wf.ReservationListResponse], error) {
	m := r.Msg
	if len(m.Filters) > 64 {
		return nil, invalidCatalog()
	}
	for _, f := range m.Filters {
		if f == nil || !validReservationRef(f.Ref) {
			return nil, invalidCatalog()
		}
	}
	p, e := newCatalogPage(a.Catalog.Namespace, "reservation", r.Header(), m.Filters, m.Size, m.After, m.Reverse, 9)
	if e != nil {
		return nil, e
	}
	result, e := collect(ctx, a.Catalog, r.Header(), "Reservation.List", func(ctx context.Context, node member, runner *wf.Runner) ([]*wf.Reservation, error) {
		if !slices.Contains(runner.Capabilities, "reservation-recovery-v1") {
			return nil, nil
		}
		return scan(ctx, p, nil, 9, func(v *wf.Reservation) bool {
			match := len(m.Filters) == 0
			for _, f := range m.Filters {
				match = match || reservationMatches(f.Ref, v)
			}
			return match
		}, func(after string) ([]*wf.Reservation, string, error) {
			page, e := a.Catalog.client(node).Reservations.List(ctx, request(&wf.ReservationListRequest{Size: int32(min(p.size+1, 1000)), After: after, Reverse: m.Reverse}, runner.Id, 0))
			if e != nil {
				return nil, "", e
			}
			return page.Msg.Items, page.Msg.Next, nil
		})
	})
	if e != nil {
		return nil, e
	}
	// Multiple processes may observe the same durable pool. Keep its first
	// observation in stable node order; warn if it changed during this read.
	seen := map[string]*wf.Reservation{}
	items := []observation[*wf.Reservation]{}
	for _, item := range result.items {
		key := string(item.value.Id)
		if old, ok := seen[key]; ok {
			if !proto.Equal(old, item.value) {
				result.warnings = append(result.warnings, "reservation observations changed; start a fresh query or select wafl-session")
			}
			continue
		}
		seen[key] = item.value
		items = append(items, item)
	}
	values, next := pageItems(p, items, false)
	return observedResponse(&wf.ReservationListResponse{Items: values, Next: next}, result), nil
}

var _ waflconnect.RunnerServiceHandler = (*RunnerAPI)(nil)
var _ waflconnect.InstanceServiceHandler = (*InstanceAPI)(nil)
var _ waflconnect.ReservationServiceHandler = (*ReservationAPI)(nil)
