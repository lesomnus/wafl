package platform

import (
	"context"
	"encoding/json"
	"math"
	"time"

	apps "k8s.io/api/apps/v1"
	meta "k8s.io/apimachinery/pkg/apis/meta/v1"
	"k8s.io/apimachinery/pkg/apis/meta/v1/unstructured"
)

type scaleState struct {
	Date, Activity, Change int64
	Busy, Started          uint64
}

func (c *Controller) scale(ctx context.Context, a *unstructured.Unstructured, d *apps.Deployment) error {
	spec, e := decodeApp(a)
	if e != nil || spec.Scaling == nil {
		return e
	}
	s := spec.Scaling
	observations, e := c.Traffic.all(ctx, string(a.GetUID()))
	if e != nil {
		return e
	}
	now := time.Now().UnixMilli()
	state := scaleState{Date: now, Activity: now, Change: now}
	_ = json.Unmarshal([]byte(d.Annotations[Group+"/scale"]), &state)
	active := uint64(0)
	busy := uint64(0)
	started := uint64(0)
	fresh := true
	for _, o := range observations {
		active += uint64(o.Active)
		busy += o.BusyMs
		started += o.Started
		if !o.Closed && (now-int64(o.DateSeen) > 5000 || int64(o.DateSeen) > now) {
			fresh = false
		}
	}
	load := float64(active)
	if now > state.Date && busy >= state.Busy {
		load = math.Max(load, float64(busy-state.Busy)/float64(now-state.Date))
	}
	if active > 0 || started > state.Started {
		state.Activity = now
	}
	desired := int32(math.Ceil(load / float64(s.Target)))
	desired = max(int32(s.Min), min(int32(s.Max), desired))
	current := *d.Spec.Replicas
	if desired == 0 && now-state.Activity < int64(s.IdleMs) {
		desired = min(1, current)
	}
	if desired > current {
		if now-state.Change < int64(s.UpMs) {
			desired = current
		} else {
			desired = min(desired, current+1)
		}
	} else if desired < current {
		if !fresh || now-state.Change < int64(s.DownMs) {
			desired = current
		} else {
			desired = max(desired, current-1)
		}
	}
	if !fresh && desired < current {
		desired = current
	}
	if desired != current {
		state.Change = now
	}
	state.Date = now
	state.Busy = busy
	state.Started = started
	data, _ := json.Marshal(state)
	copy := d.DeepCopy()
	if copy.Annotations == nil {
		copy.Annotations = map[string]string{}
	}
	copy.Annotations[Group+"/scale"] = string(data)
	copy.Spec.Replicas = &desired
	_, e = c.Clients.Kube.AppsV1().Deployments(a.GetNamespace()).Update(ctx, copy, meta.UpdateOptions{})
	return e
}
