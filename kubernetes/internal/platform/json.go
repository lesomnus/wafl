package platform

import (
	"encoding/json"
	"fmt"
	"strconv"

	"google.golang.org/protobuf/encoding/protojson"
	"google.golang.org/protobuf/reflect/protoreflect"
)

// Retain explicit zero values that override Kubernetes defaults and convert
// only protobuf integer fields. Environment variable names are opaque strings.
func kubernetesJSON(message protoreflect.Message, source map[string]any) (map[string]any, error) {
	b, e := protojson.Marshal(message.Interface())
	if e != nil {
		return nil, e
	}
	var out map[string]any
	if e = json.Unmarshal(b, &out); e != nil {
		return nil, e
	}
	fields := message.Descriptor().Fields()
	for i := 0; i < fields.Len(); i++ {
		field := fields.Get(i)
		name := field.JSONName()
		raw, present := source[name]
		if !present {
			raw, present = source[field.TextName()]
		}
		value, exists := out[name]
		if !exists && present {
			out[name] = raw
			value = raw
			exists = true
		}
		if !exists || value == nil {
			continue
		}
		if field.IsMap() {
			continue
		}
		if field.IsList() {
			if field.Kind() == protoreflect.MessageKind {
				values := value.([]any)
				original, _ := raw.([]any)
				for j := range values {
					input := map[string]any{}
					if j < len(original) {
						input, _ = original[j].(map[string]any)
					}
					values[j], e = kubernetesJSON(message.Get(field).List().Get(j).Message(), input)
					if e != nil {
						return nil, e
					}
				}
			}
			continue
		}
		switch field.Kind() {
		case protoreflect.MessageKind:
			input, _ := raw.(map[string]any)
			out[name], e = kubernetesJSON(message.Get(field).Message(), input)
			if e != nil {
				return nil, e
			}
		case protoreflect.Uint64Kind, protoreflect.Int64Kind, protoreflect.Sint64Kind, protoreflect.Fixed64Kind, protoreflect.Sfixed64Kind:
			if text, ok := value.(string); ok {
				number, e := strconv.ParseInt(text, 10, 64)
				if e != nil {
					return nil, fmt.Errorf("%s exceeds Kubernetes integer range", name)
				}
				out[name] = number
			}
		}
	}
	return out, nil
}
