package platform

import (
	"bytes"
	"context"
	"encoding/json"
	"errors"
	"fmt"
	"reflect"
	"regexp"
	"sort"
	"strings"
	"unicode/utf8"

	wf "github.com/lesomnus/wafl/kubernetes/gen/wafl"
	apps "k8s.io/api/apps/v1"
	core "k8s.io/api/core/v1"
	apierrors "k8s.io/apimachinery/pkg/api/errors"
	"k8s.io/apimachinery/pkg/api/resource"
	meta "k8s.io/apimachinery/pkg/apis/meta/v1"
	"k8s.io/apimachinery/pkg/apis/meta/v1/unstructured"
	"k8s.io/apimachinery/pkg/util/intstr"
)

var envName = regexp.MustCompile(`^[A-Za-z_][A-Za-z0-9_]{0,127}$`)

// Snapshots are immutable and named by the source UID/resourceVersion, never by
// a secret-value hash. Old ReplicaSets retain their original secret references.
func (c *Clients) snapshot(ctx context.Context, a *unstructured.Unstructured, spec *wf.App) ([]core.EnvVar, error) {
	env := []core.EnvVar{}
	seen := map[string]bool{}
	sources := map[string]*core.Secret{}
	total := 0
	for _, binding := range spec.Secrets {
		name := binding.Env
		ref := binding.GetSecretKeyRef()
		if !envName.MatchString(name) || name == "PORT" || seen[name] || len(spec.Secrets) > 64 {
			return nil, errors.New("invalid or duplicate secret environment name")
		}
		if _, exists := spec.Manifest.Env[name]; exists {
			return nil, errors.New("secret environment conflicts with manifest.env")
		}
		seen[name] = true
		if ref == nil {
			return nil, errors.New("Kubernetes App secrets require secretKeyRef")
		}
		s, known := sources[ref.Name]
		if !known {
			var err error
			s, err = c.Kube.CoreV1().Secrets(a.GetNamespace()).Get(ctx, ref.Name, meta.GetOptions{})
			if apierrors.IsNotFound(err) {
				s = nil
			} else if err != nil {
				return nil, errors.New("cannot read source Secret")
			}
			sources[ref.Name] = s
		}
		if s == nil {
			if ref.Optional {
				continue
			}
			return nil, errors.New("source Secret is missing")
		}
		value, found := s.Data[ref.Key]
		if !found {
			if ref.Optional {
				continue
			}
			return nil, errors.New("source Secret key is missing")
		}
		total += len(value) + len(name)
		if !utf8.Valid(value) || bytes.ContainsRune(value, 0) || len(value) > 16384 || total > 65536 {
			return nil, errors.New("secret environment exceeds limits or is not UTF-8")
		}
		key := "wafl-secret-" + digest([]string{string(a.GetUID()), string(s.UID), s.ResourceVersion, ref.Key})[:32]
		secret := &core.Secret{ObjectMeta: meta.ObjectMeta{Name: key, Namespace: a.GetNamespace(), Labels: map[string]string{Managed: "true", AppUID: string(a.GetUID()), Kind: "snapshot"}, OwnerReferences: []meta.OwnerReference{owner(a)}}, Immutable: Ptr(true), Data: map[string][]byte{"value": value}}
		_, err := c.Kube.CoreV1().Secrets(a.GetNamespace()).Create(ctx, secret, meta.CreateOptions{})
		if apierrors.IsAlreadyExists(err) {
			existing, e := c.Kube.CoreV1().Secrets(a.GetNamespace()).Get(ctx, key, meta.GetOptions{})
			if e != nil || existing.Labels[AppUID] != string(a.GetUID()) || !bytes.Equal(existing.Data["value"], value) || existing.Immutable == nil || !*existing.Immutable {
				return nil, errors.New("secret snapshot identity conflict")
			}
		} else if err != nil {
			return nil, err
		}
		env = append(env, core.EnvVar{Name: name, ValueFrom: &core.EnvVarSource{SecretKeyRef: &core.SecretKeySelector{LocalObjectReference: core.LocalObjectReference{Name: key}, Key: "value"}}})
	}
	sort.Slice(env, func(i, j int) bool { return env[i].Name < env[j].Name })
	return env, nil
}

func readiness(spec *wf.App) *core.Probe {
	p := &core.Probe{ProbeHandler: core.ProbeHandler{TCPSocket: &core.TCPSocketAction{Port: intstr.FromInt32(int32(spec.Manifest.Port))}}, PeriodSeconds: 1, TimeoutSeconds: 1, FailureThreshold: 3, SuccessThreshold: 1}
	if r := spec.Readiness; r != nil {
		switch kind := r.Kind.(type) {
		case *wf.Probe_Http:
			p.ProbeHandler = core.ProbeHandler{HTTPGet: &core.HTTPGetAction{Path: kind.Http, Port: intstr.FromInt32(int32(spec.Manifest.Port))}}
		case *wf.Probe_Grpc:
			p.ProbeHandler = core.ProbeHandler{GRPC: &core.GRPCAction{Port: int32(spec.Manifest.Port), Service: Ptr(kind.Grpc)}}
		}
		if r.PeriodSeconds > 0 {
			p.PeriodSeconds = int32(r.PeriodSeconds)
		}
		if r.TimeoutSeconds > 0 {
			p.TimeoutSeconds = int32(r.TimeoutSeconds)
		}
		if r.Failures > 0 {
			p.FailureThreshold = int32(r.Failures)
		}
		if r.Successes > 0 {
			p.SuccessThreshold = int32(r.Successes)
		}
	}
	return p
}

func deployment(a *unstructured.Unstructured, spec *wf.App, env []core.EnvVar) *apps.Deployment {
	labels := map[string]string{Managed: "true", AppUID: string(a.GetUID())}
	annotations := map[string]string{Manifest: manifestJSON(spec.Manifest)}
	template := core.PodTemplateSpec{ObjectMeta: meta.ObjectMeta{Labels: labels, Annotations: annotations, Finalizers: []string{Finalizer}}, Spec: core.PodSpec{
		AutomountServiceAccountToken: Ptr(false), EnableServiceLinks: Ptr(false), RestartPolicy: core.RestartPolicyAlways,
		NodeSelector: map[string]string{Runtime: "wasmtime"}, Tolerations: []core.Toleration{{Key: Runtime, Operator: core.TolerationOpEqual, Value: "wasmtime", Effect: core.TaintEffectNoSchedule}},
		TerminationGracePeriodSeconds: Ptr(int64((spec.Rollout.DrainMs + 999) / 1000)),
		Containers: []core.Container{{Name: "app", Image: "wafl.invalid/wasm:pending", ImagePullPolicy: core.PullNever, Env: env,
			Ports: []core.ContainerPort{{Name: "app", ContainerPort: int32(spec.Manifest.Port)}}, ReadinessProbe: readiness(spec),
			Resources: core.ResourceRequirements{Requests: core.ResourceList{Slots: resource.MustParse("1"), Memory: *resource.NewQuantity(int64(spec.Manifest.MemoryBytes), resource.DecimalSI)}, Limits: core.ResourceList{Slots: resource.MustParse("1"), Memory: *resource.NewQuantity(int64(spec.Manifest.MemoryBytes), resource.DecimalSI)}}}},
	}}
	revision := digest(template)
	template.Annotations[Revision] = revision
	template.Spec.Containers[0].Image = "wafl.invalid/wasm:" + revision
	replicas := int32(spec.Replicas)
	if spec.Scaling != nil {
		replicas = int32(spec.Scaling.Min)
	}
	return &apps.Deployment{ObjectMeta: meta.ObjectMeta{Name: appName(a), Namespace: a.GetNamespace(), Labels: labels, OwnerReferences: []meta.OwnerReference{owner(a)}}, Spec: apps.DeploymentSpec{
		Replicas: &replicas, RevisionHistoryLimit: Ptr(int32(3)), ProgressDeadlineSeconds: Ptr(int32(600)), Selector: &meta.LabelSelector{MatchLabels: labels}, Template: template,
		Strategy: apps.DeploymentStrategy{Type: apps.RollingUpdateDeploymentStrategyType, RollingUpdate: &apps.RollingUpdateDeployment{MaxSurge: Ptr(intstr.FromInt32(int32(spec.Rollout.Surge))), MaxUnavailable: Ptr(intstr.FromInt32(int32(spec.Rollout.Unavailable)))}}}}
}

// Validate the supported Pod profile before starting any execution. This also
// rejects manually authored Pods accidentally scheduled onto a WAFL node.
func validatePod(p *core.Pod) (*wf.Manifest, error) {
	fail := func() (*wf.Manifest, error) {
		return nil, errors.New("unsupported WAFL Pod profile; create an App instead")
	}
	if !managed(p) || p.Annotations[Revision] == "" || len(p.Spec.Containers) != 1 || len(p.Spec.InitContainers) > 0 || len(p.Spec.EphemeralContainers) > 0 || len(p.Spec.Volumes) > 0 || p.Spec.HostNetwork || p.Spec.HostPID || p.Spec.HostIPC || p.Spec.SecurityContext != nil && !reflect.DeepEqual(*p.Spec.SecurityContext, core.PodSecurityContext{}) || p.Spec.RuntimeClassName != nil {
		return fail()
	}
	c := p.Spec.Containers[0]
	if c.Name != "app" || c.Image != "wafl.invalid/wasm:"+p.Annotations[Revision] || len(c.Command) > 0 || len(c.Args) > 0 || len(c.VolumeMounts) > 0 || len(c.EnvFrom) > 0 || c.SecurityContext != nil || c.Lifecycle != nil || c.LivenessProbe != nil || c.StartupProbe != nil || c.WorkingDir != "" || c.Stdin || c.TTY || len(c.VolumeDevices) > 0 {
		return fail()
	}
	for _, resources := range []core.ResourceList{c.Resources.Requests, c.Resources.Limits} {
		for name := range resources {
			if name != Slots && name != Memory {
				return fail()
			}
		}
	}
	for _, e := range c.Env {
		if e.Value != "" || e.ValueFrom == nil || e.ValueFrom.SecretKeyRef == nil || e.ValueFrom.ConfigMapKeyRef != nil || e.ValueFrom.FieldRef != nil || e.ValueFrom.ResourceFieldRef != nil {
			return fail()
		}
	}
	m, err := podManifest(p)
	if err != nil {
		return nil, errors.New("invalid immutable manifest")
	}
	if m.Runtime != "wasmtime" || m.Abi != "wasip2" || m.Port == 0 || m.Port > 65535 || m.IdleMs != 0 || m.MaxActive != 0 {
		return fail()
	}
	for _, resources := range []core.ResourceList{c.Resources.Requests, c.Resources.Limits} {
		slots, memory := resources[Slots], resources[Memory]
		if slots.Cmp(resource.MustParse("1")) != 0 || m.MemoryBytes == 0 || m.MemoryBytes > 1<<63-1 || memory.Cmp(*resource.NewQuantity(int64(m.MemoryBytes), resource.DecimalSI)) != 0 {
			return fail()
		}
	}
	if len(c.Ports) != 1 || c.Ports[0].ContainerPort != int32(m.Port) || c.Ports[0].HostPort != 0 {
		return fail()
	}
	if c.ReadinessProbe == nil || c.ReadinessProbe.Exec != nil {
		return fail()
	}
	probe := c.ReadinessProbe
	if probe.InitialDelaySeconds != 0 || probe.TerminationGracePeriodSeconds != nil || probe.PeriodSeconds < 1 || probe.TimeoutSeconds < 1 || probe.FailureThreshold < 1 || probe.SuccessThreshold < 1 {
		return fail()
	}
	count := 0
	if h := probe.HTTPGet; h != nil {
		count++
		if h.Port.IntVal != int32(m.Port) || h.Port.Type != intstr.Int || h.Host != "" || len(h.HTTPHeaders) > 0 || h.Scheme != "" && h.Scheme != core.URISchemeHTTP || !strings.HasPrefix(h.Path, "/") {
			return fail()
		}
	}
	if t := probe.TCPSocket; t != nil {
		count++
		if t.Port.IntVal != int32(m.Port) || t.Port.Type != intstr.Int || t.Host != "" {
			return fail()
		}
	}
	if g := probe.GRPC; g != nil {
		count++
		if g.Port != int32(m.Port) {
			return fail()
		}
	}
	if count != 1 {
		return fail()
	}
	if !strings.HasPrefix(m.Bundle.GetDigest(), "sha256:") {
		return fail()
	}
	return m, nil
}

func (c *Clients) reconcileDeployment(ctx context.Context, a *unstructured.Unstructured) (*apps.Deployment, *wf.App, error) {
	spec, err := decodeApp(a)
	if err != nil {
		return nil, nil, err
	}
	env, err := c.snapshot(ctx, a, spec)
	if err != nil {
		return nil, spec, err
	}
	wanted := deployment(a, spec, env)
	api := c.Kube.AppsV1().Deployments(a.GetNamespace())
	existing, err := api.Get(ctx, wanted.Name, meta.GetOptions{})
	if apierrors.IsNotFound(err) {
		d, e := api.Create(ctx, wanted, meta.CreateOptions{})
		return d, spec, e
	}
	if err != nil {
		return nil, spec, err
	}
	if existing.Labels[AppUID] != string(a.GetUID()) {
		return nil, spec, fmt.Errorf("Deployment ownership conflict")
	}
	if spec.Scaling != nil {
		wanted.Spec.Replicas = existing.Spec.Replicas
	}
	// Kubernetes defaults many fields. Compare our desired inputs rather than
	// overwriting the template every time a default is populated by the API.
	if existing.Spec.Template.Annotations[Revision] == wanted.Spec.Template.Annotations[Revision] && *existing.Spec.Replicas == *wanted.Spec.Replicas {
		old, _ := json.Marshal(existing.Spec.Strategy)
		next, _ := json.Marshal(wanted.Spec.Strategy)
		if bytes.Equal(old, next) {
			return existing, spec, nil
		}
	}
	wanted.ResourceVersion = existing.ResourceVersion
	wanted.Annotations = existing.Annotations
	d, err := api.Update(ctx, wanted, meta.UpdateOptions{})
	return d, spec, err
}
