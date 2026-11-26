package cli

import (
	"context"
	"crypto/sha256"
	"encoding/hex"
	"encoding/json"
	"errors"
	"fmt"
	"io"
	"net/http"
	"net/url"
	"os"
	"path/filepath"
	"slices"
	"strconv"
	"strings"
	"unicode/utf8"

	"github.com/lesomnus/protobuf-patch/patch"
	wf "github.com/lesomnus/wafl/kubernetes/gen/wafl"
	"github.com/spf13/cobra"
	"google.golang.org/protobuf/encoding/protojson"
	"google.golang.org/protobuf/reflect/protoreflect"
	"sigs.k8s.io/yaml"
)

func readDocument(path string) (map[string]any, error) {
	file, e := os.Open(path)
	if e != nil {
		return nil, e
	}
	defer file.Close()
	b, e := io.ReadAll(io.LimitReader(file, (1<<20)+1))
	if e != nil {
		return nil, e
	}
	if len(b) > 1<<20 {
		return nil, errors.New("document exceeds 1 MiB")
	}
	b, e = yaml.YAMLToJSONStrict(b)
	if e != nil {
		return nil, errors.New("invalid App YAML")
	}
	var v map[string]any
	e = json.Unmarshal(b, &v)
	if e != nil || v == nil {
		return nil, errors.New("document must be a mapping")
	}
	return v, nil
}
func reference(value string) *wf.AppRef {
	if id, e := parseID(value, 10); e == nil {
		return &wf.AppRef{Key: &wf.AppRef_Id{Id: id}}
	}
	return &wf.AppRef{Key: &wf.AppRef_Alias{Alias: strings.TrimPrefix(value, "@")}}
}
func document(path string, o *options) (map[string]any, error) {
	v, e := readDocument(path)
	if e != nil {
		return nil, e
	}
	if v["kind"] == "App" {
		// Reading an exported declaration does not need Kubernetes clients. The
		// operator can still submit the original CRD with kubectl independently.
		metadata, _ := v["metadata"].(map[string]any)
		spec, ok := v["spec"].(map[string]any)
		if !ok {
			return nil, errors.New("App spec must be a mapping")
		}
		if ns, _ := metadata["namespace"].(string); ns != "" && ns != strings.TrimPrefix(o.tenant, "@") {
			return nil, errors.New("declaration namespace differs from --tenant")
		}
		spec["alias"] = metadata["name"]
		return spec, nil
	}
	return v, nil
}

// Normalize protobuf field spelling without touching opaque map keys or
// discarding explicit zero/null values from a merge document.
func canonical(input map[string]any, descriptor protoreflect.MessageDescriptor) (map[string]any, error) {
	out := map[string]any{}
	for key, value := range input {
		field := descriptor.Fields().ByJSONName(key)
		if field == nil {
			field = descriptor.Fields().ByName(protoreflect.Name(key))
		}
		if field == nil {
			return nil, errors.New("unknown App field")
		}
		key = field.JSONName()
		if _, exists := out[key]; exists {
			return nil, errors.New("repeated App field")
		}
		if field.Kind() == protoreflect.MessageKind && !field.IsMap() && value != nil {
			if field.IsList() {
				if list, ok := value.([]any); ok {
					for i, entry := range list {
						if m, ok := entry.(map[string]any); ok {
							v, e := canonical(m, field.Message())
							if e != nil {
								return nil, e
							}
							list[i] = v
						}
					}
				}
			} else if m, ok := value.(map[string]any); ok {
				v, e := canonical(m, field.Message())
				if e != nil {
					return nil, e
				}
				value = v
			}
		}
		out[key] = value
	}
	return out, nil
}
func localFile(source, base string) (string, error) {
	u, e := url.Parse(source)
	if e != nil {
		return "", e
	}
	if u.Scheme != "file" {
		return "", nil
	}
	if u.Host != "" && u.Host != "localhost" {
		return "", errors.New("file URL must name a local file")
	}
	path := u.Path
	if path == "" {
		path = u.Opaque
	}
	if path == "" {
		return "", errors.New("empty file URL")
	}
	if !filepath.IsAbs(path) {
		path = filepath.Join(base, path)
	}
	return path, nil
}
func uploadBundle(ctx context.Context, o *options, bundle map[string]any, base string) error {
	source, _ := bundle["url"].(string)
	path, e := localFile(source, base)
	if e != nil || path == "" {
		return e
	}
	file, e := os.Open(path)
	if e != nil {
		return e
	}
	defer file.Close()
	info, e := file.Stat()
	if e != nil || !info.Mode().IsRegular() {
		return errors.New("bundle must be a regular file")
	}
	h := sha256.New()
	n, e := io.Copy(h, io.LimitReader(file, (1<<30)+1))
	if e != nil {
		return e
	}
	if n > 1<<30 {
		return errors.New("bundle exceeds 1 GiB")
	}
	digest := hex.EncodeToString(h.Sum(nil))
	if expected, _ := bundle["digest"].(string); expected != "" && expected != "sha256:"+digest {
		return errors.New("bundle digest mismatch")
	}
	if _, e = file.Seek(0, io.SeekStart); e != nil {
		return e
	}
	r, e := http.NewRequestWithContext(ctx, http.MethodPut, strings.TrimRight(o.endpoint, "/")+"/bundles/"+digest, file)
	if e != nil {
		return e
	}
	r.Header.Set("Content-Type", "application/gzip")
	r.Header.Set("wafl-tenant", o.tenant)
	response, e := http.DefaultClient.Do(r)
	if e != nil {
		return e
	}
	defer response.Body.Close()
	b, e := io.ReadAll(io.LimitReader(response.Body, 8193))
	if e != nil {
		return e
	}
	if response.StatusCode != http.StatusCreated {
		return fmt.Errorf("controller rejected bundle upload (HTTP %d)", response.StatusCode)
	}
	var asset wf.Asset
	if len(b) > 8192 || protojson.Unmarshal(b, &asset) != nil || asset.Digest != "sha256:"+digest || asset.GetUrl() == "" {
		return errors.New("invalid controller bundle response")
	}
	bundle["url"] = asset.GetUrl()
	bundle["digest"] = asset.Digest
	return nil
}
func prepareFiles(ctx context.Context, o *options, v map[string]any, base string) error {
	if m, ok := v["manifest"].(map[string]any); ok {
		if bundle, ok := m["bundle"].(map[string]any); ok {
			if e := uploadBundle(ctx, o, bundle, base); e != nil {
				return e
			}
		}
	}
	raw, exists := v["secrets"]
	if !exists || raw == nil {
		return nil
	}
	bindings, ok := raw.([]any)
	if !ok {
		return errors.New("secrets must be a list")
	}
	values, _ := v["secretValues"].(map[string]any)
	if values == nil {
		values = map[string]any{}
	}
	kept := []any{}
	seen := map[string]bool{}
	for _, entry := range bindings {
		binding, ok := entry.(map[string]any)
		if !ok {
			return errors.New("secret must be a mapping")
		}
		env, ok := binding["env"].(string)
		if !ok || env == "" || seen[env] {
			return errors.New("invalid or repeated Secret environment name")
		}
		seen[env] = true
		path, ok := binding["file"].(string)
		if !ok {
			kept = append(kept, binding)
			continue
		}
		if _, ok = binding["secretKeyRef"]; ok {
			return errors.New("Secret must have exactly one source")
		}
		if !filepath.IsAbs(path) {
			path = filepath.Join(base, path)
		}
		file, e := os.Open(path)
		if e != nil {
			return errors.New("cannot open secret file")
		}
		info, e := file.Stat()
		if e != nil || !info.Mode().IsRegular() {
			file.Close()
			return errors.New("Secret must be a regular file")
		}
		b, e := io.ReadAll(io.LimitReader(file, 16385))
		file.Close()
		if e != nil || len(b) > 16384 || !utf8.Valid(b) || strings.ContainsRune(string(b), 0) {
			return errors.New("invalid secret file contents")
		}
		values[env] = string(b)
	}
	v["secrets"] = kept
	if len(values) > 0 {
		v["secretValues"] = values
	}
	return nil
}
func merge(base, changes map[string]any) {
	for key, value := range changes {
		if value == nil {
			delete(base, key)
			continue
		}
		if m, ok := value.(map[string]any); ok {
			old, _ := base[key].(map[string]any)
			if old == nil {
				old = map[string]any{}
			}
			merge(old, m)
			base[key] = old
		} else {
			base[key] = value
		}
	}
}
func fieldValue(field protoreflect.FieldDescriptor, value protoreflect.Value) patch.Value {
	if field.IsList() {
		values := []patch.Value{}
		list := value.List()
		for i := 0; i < list.Len(); i++ {
			values = append(values, singleValue(field.Kind(), list.Get(i)))
		}
		return patch.List(values...)
	}
	if field.IsMap() {
		values := []patch.MapEntry{}
		value.Map().Range(func(key protoreflect.MapKey, v protoreflect.Value) bool {
			values = append(values, patch.E(patch.MapStr(key.String()), singleValue(field.MapValue().Kind(), v)))
			return true
		})
		return patch.Map(values...)
	}
	return singleValue(field.Kind(), value)
}
func singleValue(kind protoreflect.Kind, value protoreflect.Value) patch.Value {
	switch kind {
	case protoreflect.BoolKind:
		return patch.Bool(value.Bool())
	case protoreflect.StringKind:
		return patch.Str(value.String())
	case protoreflect.BytesKind:
		return patch.Bytes(value.Bytes())
	case protoreflect.Uint32Kind, protoreflect.Fixed32Kind:
		return patch.Uint32(uint32(value.Uint()))
	case protoreflect.Uint64Kind, protoreflect.Fixed64Kind:
		return patch.Uint64(value.Uint())
	case protoreflect.Int32Kind, protoreflect.Sint32Kind, protoreflect.Sfixed32Kind:
		return patch.Int32(int32(value.Int()))
	case protoreflect.Int64Kind, protoreflect.Sint64Kind, protoreflect.Sfixed64Kind:
		return patch.Int64(value.Int())
	case protoreflect.FloatKind:
		return patch.Float32(float32(value.Float()))
	case protoreflect.DoubleKind:
		return patch.Float64(value.Float())
	case protoreflect.EnumKind:
		return patch.Enum(int32(value.Enum()))
	case protoreflect.MessageKind:
		fields := []patch.FieldValue{}
		m := value.Message()
		descriptors := m.Descriptor().Fields()
		for i := 0; i < descriptors.Len(); i++ {
			f := descriptors.Get(i)
			if m.Has(f) {
				fields = append(fields, patch.F(patch.Num(uint32(f.Number())), fieldValue(f, m.Get(f))))
			}
		}
		return patch.Msg(fields...)
	default:
		panic("unsupported App field kind")
	}
}
func appCommand(o *options) *cobra.Command {
	root := &cobra.Command{Use: "app", Short: "Manage apps through the controller API"}
	add := &cobra.Command{Use: "add app.yaml", Args: cobra.ExactArgs(1), RunE: func(cmd *cobra.Command, args []string) error {
		v, e := document(args[0], o)
		if e != nil {
			return e
		}
		v, e = canonical(v, (&wf.AppAddRequest{}).ProtoReflect().Descriptor())
		if e != nil {
			return e
		}
		if _, exists := v["replicas"]; !exists && v["scaling"] == nil {
			v["replicas"] = 1
		}
		if rollout, ok := v["rollout"].(map[string]any); ok {
			if _, exists := rollout["surge"]; !exists {
				rollout["surge"] = 1
			}
		}
		if e = prepareFiles(cmd.Context(), o, v, filepath.Dir(args[0])); e != nil {
			return e
		}
		b, e := json.Marshal(v)
		if e != nil {
			return errors.New("invalid App declaration")
		}
		q := &wf.AppAddRequest{}
		if e = protojson.Unmarshal(b, q); e != nil {
			return errors.New("invalid App declaration or unknown field")
		}
		r, e := o.apps().Add(cmd.Context(), request(o, q))
		if e != nil {
			return e
		}
		if o.json {
			return output(r.Msg)
		}
		_, e = fmt.Fprintf(os.Stdout, "%s\t@%s\t%s\n", uuid(r.Msg.Id), r.Msg.Alias, r.Msg.GetTenant().GetAlias())
		return e
	}}
	get := &cobra.Command{Use: "get ID|@ALIAS", Args: cobra.ExactArgs(1), RunE: func(cmd *cobra.Command, args []string) error {
		r, e := o.apps().Get(cmd.Context(), request(o, &wf.AppGetRequest{Ref: reference(args[0])}))
		if e != nil {
			return e
		}
		return output(r.Msg)
	}}
	list := &cobra.Command{Use: "list", Args: cobra.NoArgs, RunE: func(cmd *cobra.Command, _ []string) error {
		q := &wf.AppListRequest{}
		for {
			r, e := o.apps().List(cmd.Context(), request(o, q))
			if e != nil {
				return e
			}
			for _, v := range r.Msg.Items {
				if e = output(v); e != nil {
					return e
				}
			}
			if r.Msg.Next == "" {
				return nil
			}
			q.After = r.Msg.Next
		}
	}}
	erase := &cobra.Command{Use: "erase ID|@ALIAS", Args: cobra.ExactArgs(1), RunE: func(cmd *cobra.Command, args []string) error {
		r, e := o.apps().Erase(cmd.Context(), request(o, reference(args[0])))
		if e != nil {
			return e
		}
		if o.json {
			return output(r.Msg)
		}
		return nil
	}}
	var patchFile string
	update := &cobra.Command{Use: "patch ID|@ALIAS --file patch.yaml", Args: cobra.ExactArgs(1), RunE: func(cmd *cobra.Command, args []string) error {
		changes, e := readDocument(patchFile)
		if e != nil {
			return e
		}
		if wrapped, ok := changes["spec"].(map[string]any); ok {
			changes = wrapped
		}
		changes, e = canonical(changes, (&wf.App{}).ProtoReflect().Descriptor())
		if e != nil {
			return e
		}
		for key := range changes {
			switch key {
			case "name", "replicas", "manifest", "scaling", "secrets", "readiness", "rollout", "secretValues", "secret_values":
			default:
				return errors.New("patch contains an unknown or read-only field")
			}
		}
		ref := reference(args[0])
		current, e := o.apps().Get(cmd.Context(), request(o, &wf.AppGetRequest{Ref: ref}))
		if e != nil {
			return e
		}
		// An alias may be erased and recreated with the same generation. Bind the
		// update to the identity that Get actually returned as well as its revision.
		ref = &wf.AppRef{Key: &wf.AppRef_Id{Id: current.Msg.Id}}
		// Convert merge-style YAML into the shared protobuf patch format. Fence the
		// Get/merge/Apply sequence, so a simultaneous edit cannot be overwritten.
		raw, _ := protojson.Marshal(current.Msg)
		var base map[string]any
		_ = json.Unmarshal(raw, &base)
		merge(base, changes)
		if e = prepareFiles(cmd.Context(), o, base, filepath.Dir(patchFile)); e != nil {
			return e
		}
		data, _ := json.Marshal(base)
		next := &wf.App{}
		if e = protojson.Unmarshal(data, next); e != nil {
			return errors.New("invalid App patch")
		}
		if len(next.SecretValues) > 0 {
			changes["secretValues"] = base["secretValues"]
		}
		keys := []string{}
		for key := range changes {
			keys = append(keys, key)
		}
		slices.Sort(keys)
		ops := []patch.Op{}
		m := next.ProtoReflect()
		for _, key := range keys {
			field := m.Descriptor().Fields().ByJSONName(key)
			if field == nil {
				field = m.Descriptor().Fields().ByName(protoreflect.Name(key))
			}
			if field == nil {
				return errors.New("invalid App patch field")
			}
			target := patch.Target(patch.Num(uint32(field.Number())))
			if changes[key] == nil {
				ops = append(ops, target.Skip().Remove())
			} else {
				ops = append(ops, target.Assign(fieldValue(field, m.Get(field))))
			}
		}
		if len(ops) == 0 {
			return errors.New("empty App patch")
		}
		p, e := patch.New("wafl.App", ops[0], ops[1:]...)
		if e != nil {
			return errors.New("invalid App patch")
		}
		q := request(o, &wf.AppApplyRequest{Ref: ref, Patch: p})
		q.Header().Set("wafl-revision", strconv.FormatUint(current.Msg.Revision, 10))
		r, e := o.apps().Apply(cmd.Context(), q)
		if e != nil {
			return e
		}
		return output(r.Msg)
	}}
	update.Flags().StringVar(&patchFile, "file", "", "App fields to merge; null removes a field")
	_ = update.MarkFlagRequired("file")
	var follow bool
	var instance string
	var cursor uint64
	var limit uint32
	logs := &cobra.Command{Use: "logs ID|@ALIAS", Args: cobra.ExactArgs(1), RunE: func(cmd *cobra.Command, args []string) error {
		q := &wf.AppTailRequest{Ref: reference(args[0]), Follow: follow, Cursor: cursor, Limit: limit}
		if instance != "" {
			id, e := parseID(instance, 7)
			if e != nil {
				return e
			}
			q.Instance = &wf.InstanceRef{Key: &wf.InstanceRef_Id{Id: id}}
		}
		stream, e := o.apps().Tail(cmd.Context(), request(o, q))
		if e != nil {
			return e
		}
		defer stream.Close()
		for stream.Receive() {
			if o.json {
				b, e := protojson.Marshal(stream.Msg())
				if e != nil {
					return e
				}
				if _, e = fmt.Fprintln(os.Stdout, string(b)); e != nil {
					return e
				}
			} else {
				if _, e = os.Stdout.Write(stream.Msg().Data); e != nil {
					return e
				}
			}
		}
		return stream.Err()
	}}
	logs.Flags().BoolVarP(&follow, "follow", "f", false, "Follow the selected execution")
	logs.Flags().StringVar(&instance, "instance", "", "Select an execution by ID")
	logs.Flags().Uint64Var(&cursor, "cursor", 0, "Resume beyond a sequence; requires --instance")
	logs.Flags().Uint32Var(&limit, "limit", 0, "Initial history records; zero returns retained history")
	root.AddCommand(add, get, list, erase, update, logs)
	return root
}
