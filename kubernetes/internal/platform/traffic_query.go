package platform

import (
	"bytes"
	"encoding/base64"
	"encoding/json"
	"errors"
	"slices"

	"connectrpc.com/connect"
	wf "github.com/lesomnus/wafl/kubernetes/gen/wafl"
	"google.golang.org/protobuf/reflect/protoreflect"
)

// Preserve the generated service's selection contract, including nested edges.
func selectFields(value, selection protoreflect.Message) {
	if all := selection.Descriptor().Fields().ByName("all"); all != nil && selection.Get(all).Bool() {
		return
	}
	fields := value.Descriptor().Fields()
	for n := 0; n < fields.Len(); n++ {
		field := fields.Get(n)
		if field.Name() == "id" {
			continue
		}
		pick := selection.Descriptor().Fields().ByNumber(field.Number())
		if pick == nil || !selection.Has(pick) {
			value.Clear(field)
			continue
		}
		if pick.Kind() == protoreflect.BoolKind {
			if !selection.Get(pick).Bool() {
				value.Clear(field)
			}
		} else if value.Has(field) && pick.Kind() == protoreflect.MessageKind {
			selectFields(value.Get(field).Message(), selection.Get(pick).Message())
		}
	}
}

type trafficCursor struct {
	ID      []byte
	Reverse bool
	Scope   string
}

func trafficPage(request *wf.TrafficListRequest, items []*wf.Traffic) (*connect.Response[wf.TrafficListResponse], error) {
	invalid := func() (*connect.Response[wf.TrafficListResponse], error) {
		return nil, connect.NewError(connect.CodeInvalidArgument, errors.New("invalid traffic filter or cursor"))
	}
	if len(request.Filters) > 64 {
		return invalid()
	}
	for _, f := range request.Filters {
		if f == nil || f.Ref == nil && f.App == nil || f.Ref != nil && !validID(f.Ref.GetId(), 13) || f.App != nil && !validID(f.App.GetId(), 10) {
			return invalid()
		}
	}
	scope := digest(request.Filters)
	var cursor trafficCursor
	if request.After != "" {
		if len(request.After) > 512 {
			return invalid()
		}
		b, e := base64.RawURLEncoding.DecodeString(request.After)
		if e != nil || json.Unmarshal(b, &cursor) != nil || !validID(cursor.ID, 13) || cursor.Reverse != request.Reverse || cursor.Scope != scope {
			return invalid()
		}
	}
	filtered := []*wf.Traffic{}
	for _, v := range items {
		matches := len(request.Filters) == 0
		for _, f := range request.Filters {
			if (f.Ref == nil || bytes.Equal(f.Ref.GetId(), v.Id)) && (f.App == nil || bytes.Equal(f.App.GetId(), v.GetApp().GetId())) {
				matches = true
				break
			}
		}
		if !matches {
			continue
		}
		if request.After != "" {
			cmp := bytes.Compare(v.Id, cursor.ID)
			if !request.Reverse && cmp <= 0 || request.Reverse && cmp >= 0 {
				continue
			}
		}
		filtered = append(filtered, v)
	}
	slices.SortFunc(filtered, func(a, b *wf.Traffic) int {
		cmp := bytes.Compare(a.Id, b.Id)
		if request.Reverse {
			return -cmp
		}
		return cmp
	})
	size := int(request.Size)
	if size <= 0 {
		size = 100
	}
	size = min(size, 1000)
	result := &wf.TrafficListResponse{}
	if len(filtered) > size {
		b, _ := json.Marshal(trafficCursor{ID: filtered[size-1].Id, Reverse: request.Reverse, Scope: scope})
		result.Next = base64.RawURLEncoding.EncodeToString(b)
		filtered = filtered[:size]
	}
	result.Items = filtered
	return connect.NewResponse(result), nil
}
