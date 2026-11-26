// Generate the structural App CRD schema from WAFL's protobuf descriptors.
// Kubernetes metadata/status and declarative validation are the only overlays.
package main

import (
	"bytes"
	"encoding/json"
	"fmt"
	"os"
	"path/filepath"

	"google.golang.org/protobuf/proto"
	"google.golang.org/protobuf/types/descriptorpb"
)

type object = map[string]any

var messages = map[string]*descriptorpb.DescriptorProto{}

func register(prefix string, m *descriptorpb.DescriptorProto) {
	name := prefix + "." + m.GetName()
	messages[name] = m
	for _, child := range m.GetNestedType() {
		register(name, child)
	}
}

func field(f *descriptorpb.FieldDescriptorProto) object {
	var s object
	switch f.GetType() {
	case descriptorpb.FieldDescriptorProto_TYPE_MESSAGE:
		m := messages[f.GetTypeName()]
		if m == nil {
			panic("missing descriptor: " + f.GetTypeName())
		}
		if m.GetOptions().GetMapEntry() {
			return object{"type": "object", "additionalProperties": field(m.GetField()[1])}
		}
		s = message(m)
	case descriptorpb.FieldDescriptorProto_TYPE_STRING:
		s = object{"type": "string"}
	case descriptorpb.FieldDescriptorProto_TYPE_BOOL:
		s = object{"type": "boolean"}
	case descriptorpb.FieldDescriptorProto_TYPE_UINT32, descriptorpb.FieldDescriptorProto_TYPE_UINT64:
		s = object{"type": "integer", "format": "int64", "minimum": 0}
	default:
		panic("unsupported CRD field: " + f.GetName())
	}
	if f.GetLabel() == descriptorpb.FieldDescriptorProto_LABEL_REPEATED {
		return object{"type": "array", "items": s}
	}
	return s
}

func message(m *descriptorpb.DescriptorProto) object {
	p := object{}
	for _, f := range m.GetField() {
		p[f.GetJsonName()] = field(f)
	}
	return object{"type": "object", "properties": p}
}

func props(s object) object { return s["properties"].(object) }
func at(s object, names ...string) object {
	for _, name := range names {
		s = props(s)[name].(object)
	}
	return s
}

func definition() object {
	fields := object{}
	for _, f := range messages[".wafl.App"].GetField() {
		switch f.GetName() {
		case "name", "domain", "manifest", "replicas", "scaling", "secrets", "readiness", "rollout":
			fields[f.GetJsonName()] = field(f)
		}
	}
	spec := object{"type": "object", "required": []string{"domain", "manifest"}, "properties": fields,
		"x-kubernetes-validations": []any{object{"rule": "!has(self.scaling) || !has(self.replicas) || self.replicas == 0", "message": "replicas must be omitted or zero with scaling"}}}
	domain := at(spec, "domain")
	domain["minLength"], domain["maxLength"] = 1, 253
	domain["pattern"] = `^[a-z0-9]([a-z0-9-]*[a-z0-9])?(\.[a-z0-9]([a-z0-9-]*[a-z0-9])?)*$`
	domain["x-kubernetes-validations"] = []any{object{"rule": "self == oldSelf", "message": "domain is immutable; create a new App"}}
	at(spec, "name")["maxLength"] = 256
	probe := at(spec, "readiness")
	probe["x-kubernetes-validations"] = []any{object{"rule": "has(self.http) != has(self.grpc)", "message": "choose http path or grpc service"}}
	at(probe, "http")["pattern"] = `^/`
	for _, key := range []string{"periodSeconds", "timeoutSeconds", "failures", "successes"} {
		at(probe, key)["minimum"], at(probe, key)["maximum"] = 1, 60
	}
	rollout := at(spec, "rollout")
	at(rollout, "surge")["default"], at(rollout, "surge")["maximum"] = 1, 100
	at(rollout, "unavailable")["maximum"] = 100
	at(rollout, "drainMs")["default"], at(rollout, "drainMs")["minimum"], at(rollout, "drainMs")["maximum"] = 30000, 1000, 600000
	rollout["x-kubernetes-validations"] = []any{object{"rule": "!has(self.surge) || self.surge > 0 || (has(self.unavailable) && self.unavailable > 0)", "message": "surge and unavailable cannot both be zero"}}
	secrets := at(spec, "secrets")
	secrets["maxItems"] = 64
	secrets["x-kubernetes-list-type"] = "map"
	secrets["x-kubernetes-list-map-keys"] = []string{"env"}
	secret := secrets["items"].(object)
	delete(props(secret), "file") // CLI imports files as Kubernetes Secrets.
	secret["required"] = []string{"env", "secretKeyRef"}
	at(secret, "env")["pattern"] = `^[A-Za-z_][A-Za-z0-9_]*$`
	at(secret, "env")["maxLength"] = 128
	at(secret, "env")["x-kubernetes-validations"] = []any{object{"rule": "self != 'PORT'", "message": "PORT is reserved"}}
	ref := at(secret, "secretKeyRef")
	ref["required"] = []string{"name", "key"}
	at(ref, "name")["minLength"], at(ref, "name")["maxLength"] = 1, 253
	at(ref, "name")["pattern"] = `^[a-z0-9]([a-z0-9-]*[a-z0-9])?(\.[a-z0-9]([a-z0-9-]*[a-z0-9])?)*$`
	at(ref, "key")["minLength"], at(ref, "key")["maxLength"] = 1, 253
	at(ref, "key")["pattern"] = `^[a-zA-Z0-9_.-]+$`
	m := at(spec, "manifest")
	m["required"] = []string{"bundle"}
	at(m, "runtime")["enum"] = []string{"", "wasmtime"}
	at(m, "abi")["enum"] = []string{"", "wasip2"}
	at(m, "protocol")["enum"] = []string{"", "http", "h2c"}
	at(m, "port")["minimum"], at(m, "port")["maximum"], at(m, "port")["default"] = 1, 65535, 8080
	at(m, "idleMs")["maximum"], at(m, "maxActive")["maximum"] = 0, 0
	at(m, "bundle")["required"] = []string{"url", "digest"}
	at(m, "bundle", "url")["minLength"], at(m, "bundle", "url")["maxLength"] = 1, 8192
	at(m, "bundle", "digest")["pattern"] = `^sha256:[0-9a-f]{64}$`
	binding := at(m, "bindings")["items"].(object)
	binding["required"] = []string{"port", "target"}
	at(binding, "port")["minimum"], at(binding, "port")["maximum"] = 1, 65535
	condition := object{"type": "object", "required": []string{"type", "status", "reason", "message", "lastTransitionTime", "observedGeneration"}, "properties": object{
		"type": object{"type": "string"}, "status": object{"type": "string", "enum": []string{"True", "False", "Unknown"}},
		"reason": object{"type": "string"}, "message": object{"type": "string"},
		"lastTransitionTime": object{"type": "string", "format": "date-time"},
		"observedGeneration": object{"type": "integer", "format": "int64", "minimum": 0},
	}}
	status := object{"type": "object", "properties": object{
		"deployment": object{"type": "string"}, "service": object{"type": "string"},
		"observedGeneration": object{"type": "integer", "format": "int64", "minimum": 0},
		"revision":           object{"type": "integer", "format": "int64", "minimum": 0},
		"desired":            object{"type": "integer", "format": "int64", "minimum": 0},
		"ready":              object{"type": "integer", "format": "int64", "minimum": 0},
		"conditions":         object{"type": "array", "items": condition, "x-kubernetes-list-type": "map", "x-kubernetes-list-map-keys": []string{"type"}},
	}}
	return object{"apiVersion": "apiextensions.k8s.io/v1", "kind": "CustomResourceDefinition", "metadata": object{"name": "apps.wafl.opt.td"}, "spec": object{
		"group": "wafl.opt.td", "scope": "Namespaced", "names": object{"kind": "App", "plural": "apps", "singular": "app", "shortNames": []string{"wapp"}},
		"versions": []any{object{"name": "v1alpha1", "served": true, "storage": true, "subresources": object{"status": object{}},
			"additionalPrinterColumns": []any{
				object{"name": "Domain", "type": "string", "jsonPath": ".spec.domain"},
				object{"name": "Desired", "type": "integer", "jsonPath": ".status.desired"},
				object{"name": "Running", "type": "integer", "jsonPath": ".status.ready"},
				object{"name": "Ready", "type": "string", "jsonPath": ".status.conditions[?(@.type=='Ready')].status"},
				object{"name": "Age", "type": "date", "jsonPath": ".metadata.creationTimestamp"},
			},
			"schema": object{"openAPIV3Schema": object{"type": "object", "required": []string{"spec"}, "properties": object{
				"apiVersion": object{"type": "string"}, "kind": object{"type": "string"}, "metadata": object{"type": "object"}, "spec": spec, "status": status,
			}}},
		}},
	}}
}

func main() {
	root := filepath.Join("..", "..")
	data, err := os.ReadFile(filepath.Join(root, "generated/storage/descriptors.binpb"))
	if err != nil {
		panic(err)
	}
	var descriptors descriptorpb.FileDescriptorSet
	if err := proto.Unmarshal(data, &descriptors); err != nil {
		panic(err)
	}
	for _, f := range descriptors.GetFile() {
		for _, m := range f.GetMessageType() {
			register("."+f.GetPackage(), m)
		}
	}
	if err := generateGo(root, &descriptors, len(os.Args) > 1 && os.Args[1] == "--check"); err != nil {
		panic(err)
	}
	data, err = json.MarshalIndent(definition(), "", "  ")
	if err != nil {
		panic(err)
	}
	data = append(data, '\n')
	copyPath := filepath.Join(root, "kubernetes/install/apps.json")
	if len(os.Args) > 1 && os.Args[1] == "--check" {
		old, err := os.ReadFile(copyPath)
		if err != nil || !bytes.Equal(old, data) {
			panic("embedded App CRD is stale")
		}
	} else if err := os.WriteFile(copyPath, data, 0644); err != nil {
		panic(err)
	}
	path := filepath.Join(root, "deploy/kubernetes/apps.json")
	if len(os.Args) > 1 && os.Args[1] == "--check" {
		old, err := os.ReadFile(path)
		if err != nil || !bytes.Equal(old, data) {
			fmt.Fprintln(os.Stderr, "App CRD is stale: run bash scripts/codegen.sh")
			os.Exit(1)
		}
		return
	}
	if err := os.MkdirAll(filepath.Dir(path), 0755); err != nil {
		panic(err)
	}
	if err := os.WriteFile(path, data, 0644); err != nil {
		panic(err)
	}
}
