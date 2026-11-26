package cli

import (
	"bytes"
	"context"
	"crypto/rand"
	"crypto/sha256"
	"encoding/hex"
	"encoding/json"
	"errors"
	"fmt"
	"io"
	"net/http"
	"net/url"
	"os"
	"os/exec"
	"path/filepath"
	"regexp"
	"strconv"
	"strings"
	"time"
	"unicode/utf8"

	"connectrpc.com/connect"
	wf "github.com/lesomnus/wafl/kubernetes/gen/wafl"
	"github.com/lesomnus/wafl/kubernetes/internal/platform"
	"github.com/spf13/cobra"
	"google.golang.org/protobuf/encoding/protojson"
	"google.golang.org/protobuf/reflect/protoreflect"
	core "k8s.io/api/core/v1"
	meta "k8s.io/apimachinery/pkg/apis/meta/v1"
	"k8s.io/apimachinery/pkg/apis/meta/v1/unstructured"
	"k8s.io/apimachinery/pkg/types"
	"sigs.k8s.io/yaml"
)

func readDocument(path string) (map[string]any, error) {
	b, e := os.ReadFile(path)
	if e != nil {
		return nil, e
	}
	if len(b) > 1<<20 {
		return nil, errors.New("document exceeds 1 MiB")
	}
	b, e = yaml.YAMLToJSONStrict(b)
	if e != nil {
		return nil, e
	}
	var v map[string]any
	e = json.Unmarshal(b, &v)
	return v, e
}
func name(value string) string { return strings.TrimPrefix(value, "@") }
func output(o *options, v any) error {
	e := json.NewEncoder(os.Stdout)
	if o.json {
		e.SetIndent("", "  ")
	}
	return e.Encode(v)
}
func appObject(path, namespace string) (*unstructured.Unstructured, error) {
	v, e := readDocument(path)
	if e != nil {
		return nil, e
	}
	if v["kind"] == "App" {
		a := &unstructured.Unstructured{Object: v}
		if a.GetAPIVersion() != platform.Group+"/v1alpha1" {
			return nil, errors.New("unsupported App apiVersion")
		}
		if a.GetNamespace() == "" {
			a.SetNamespace(namespace)
		}
		return a, nil
	}
	alias, _ := v["alias"].(string)
	delete(v, "alias")
	if _, ok := v["tenant"]; ok {
		return nil, errors.New("Kubernetes namespaces replace controller tenant mappings")
	}
	if alias == "" {
		domain, _ := v["domain"].(string)
		alias = strings.ReplaceAll(domain, ".", "-")
	}
	if !regexp.MustCompile(`^[a-z0-9]([-a-z0-9]*[a-z0-9])?$`).MatchString(alias) || len(alias) > 63 {
		return nil, errors.New("set a DNS-compatible alias or metadata.name")
	}
	return &unstructured.Unstructured{Object: map[string]any{"apiVersion": platform.Group + "/v1alpha1", "kind": "App", "metadata": map[string]any{"name": alias, "namespace": namespace}, "spec": v}}, nil
}
func normalize(a *unstructured.Unstructured) error {
	source, ok := a.Object["spec"].(map[string]any)
	if !ok {
		return errors.New("spec must be a mapping")
	}
	allowed := map[string]bool{"name": true, "domain": true, "manifest": true, "replicas": true, "scaling": true, "secrets": true, "readiness": true, "rollout": true}
	for name := range source {
		if !allowed[name] {
			return fmt.Errorf("unsupported App spec field %s", name)
		}
	}
	b, e := json.Marshal(source)
	if e != nil {
		return e
	}
	var spec wf.App
	if e = protojson.Unmarshal(b, &spec); e != nil {
		return e
	}
	out, e := kubernetesJSON(spec.ProtoReflect(), source)
	if e != nil {
		return e
	}
	a.Object["spec"] = out
	return nil
}

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
func pinBundle(ctx context.Context, c *platform.Clients, a *unstructured.Unstructured, base, system string) error {
	spec := a.Object["spec"].(map[string]any)
	manifest, ok := spec["manifest"].(map[string]any)
	if !ok {
		return errors.New("manifest is required")
	}
	bundle, ok := manifest["bundle"].(map[string]any)
	if !ok {
		return errors.New("bundle is required")
	}
	source, _ := bundle["url"].(string)
	u, e := url.Parse(source)
	if e != nil {
		return e
	}
	if bundle["digest"] != "" && bundle["digest"] != nil && u.Scheme != "file" {
		return nil
	}
	var reader io.ReadCloser
	var local string
	switch u.Scheme {
	case "file":
		local = u.Path
		if local == "" {
			local = u.Opaque
		}
		if !filepath.IsAbs(local) {
			local = filepath.Join(base, local)
		}
		file, e := os.Open(local)
		if e != nil {
			return e
		}
		info, e := file.Stat()
		if e != nil || !info.Mode().IsRegular() {
			file.Close()
			return errors.New("bundle must be a regular file")
		}
		reader = file
	case "http", "https":
		r, e := http.NewRequestWithContext(ctx, http.MethodGet, source, nil)
		if e != nil {
			return e
		}
		response, e := (&http.Client{Timeout: 2 * time.Minute}).Do(r)
		if e != nil {
			return e
		}
		if response.StatusCode != 200 {
			response.Body.Close()
			return errors.New("bundle download failed")
		}
		reader = response.Body
	default:
		return errors.New("supply bundle.digest for this URL scheme")
	}
	defer reader.Close()
	hash := sha256.New()
	size, e := io.Copy(hash, io.LimitReader(reader, (1<<30)+1))
	if e != nil {
		return e
	}
	if size > 1<<30 {
		return errors.New("bundle exceeds 1 GiB")
	}
	digest := "sha256:" + hex.EncodeToString(hash.Sum(nil))
	if expected, ok := bundle["digest"].(string); ok && expected != "" && expected != digest {
		return errors.New("bundle digest mismatch")
	}
	bundle["digest"] = digest
	if local != "" {
		ns, e := c.Kube.CoreV1().Namespaces().Get(ctx, system, meta.GetOptions{})
		if e != nil {
			return e
		}
		cluster := ns.Annotations[platform.Group+"/dev"]
		if cluster == "" {
			return errors.New("file bundles require wafl dev; publish an HTTP(S) bundle for a remote cluster")
		}
		nodes, e := exec.CommandContext(ctx, "kind", "get", "nodes", "--name", cluster).Output()
		if e != nil {
			return e
		}
		filename := strings.TrimPrefix(digest, "sha256:") + ".tar.gz"
		for _, node := range strings.Fields(string(nodes)) {
			if e = run(ctx, "docker", "exec", node, "mkdir", "-p", "/wafl-assets"); e != nil {
				return e
			}
			if e = run(ctx, "docker", "cp", local, node+":/wafl-assets/"+filename); e != nil {
				return e
			}
			if e = run(ctx, "docker", "exec", node, "chmod", "0644", "/wafl-assets/"+filename); e != nil {
				return e
			}
		}
		bundle["url"] = "http://wafl-assets." + system + ".svc:8090/" + filename
	}
	return nil
}
func importSecrets(ctx context.Context, c *platform.Clients, a *unstructured.Unstructured, base string) ([]string, error) {
	spec := a.Object["spec"].(map[string]any)
	bindings, _ := spec["secrets"].([]any)
	created := []string{}
	for _, value := range bindings {
		binding, ok := value.(map[string]any)
		if !ok {
			return created, errors.New("secret must be a mapping")
		}
		path, ok := binding["file"].(string)
		if !ok {
			continue
		}
		if !filepath.IsAbs(path) {
			path = filepath.Join(base, path)
		}
		file, e := os.Open(path)
		if e != nil {
			return created, errors.New("cannot open secret file")
		}
		info, e := file.Stat()
		if e != nil || !info.Mode().IsRegular() {
			file.Close()
			return created, errors.New("secret must be a regular file")
		}
		data, e := io.ReadAll(io.LimitReader(file, 16385))
		file.Close()
		if e != nil || len(data) > 16384 || !utf8.Valid(data) || bytes.ContainsRune(data, 0) {
			return created, errors.New("invalid secret file contents")
		}
		id := make([]byte, 12)
		if _, e = rand.Read(id); e != nil {
			return created, e
		}
		key := "wafl-file-" + hex.EncodeToString(id)
		secret := &core.Secret{ObjectMeta: meta.ObjectMeta{Name: key, Namespace: a.GetNamespace(), Labels: map[string]string{platform.Managed: "true"}}, Immutable: platform.Ptr(true), Data: map[string][]byte{"value": data}}
		if _, e = c.Kube.CoreV1().Secrets(a.GetNamespace()).Create(ctx, secret, meta.CreateOptions{}); e != nil {
			return created, e
		}
		created = append(created, key)
		delete(binding, "file")
		binding["secretKeyRef"] = map[string]any{"name": key, "key": "value"}
	}
	return created, nil
}
func appCommand(o *options) *cobra.Command {
	app := &cobra.Command{Use: "app", Short: "Manage Kubernetes App resources"}
	add := &cobra.Command{Use: "add app.yaml", Args: cobra.ExactArgs(1), RunE: func(cmd *cobra.Command, args []string) error {
		c, e := o.clients()
		if e != nil {
			return e
		}
		a, e := appObject(args[0], o.namespace)
		if e != nil {
			return e
		}
		base := filepath.Dir(args[0])
		if e = pinBundle(cmd.Context(), c, a, base, o.system); e != nil {
			return e
		}
		secrets, e := importSecrets(cmd.Context(), c, a, base)
		committed := false
		defer func() {
			if !committed {
				for _, s := range secrets {
					_ = c.Kube.CoreV1().Secrets(a.GetNamespace()).Delete(context.Background(), s, meta.DeleteOptions{})
				}
			}
		}()
		if e != nil {
			return e
		}
		if e = normalize(a); e != nil {
			return e
		}
		created, e := c.Dynamic.Resource(platform.Apps).Namespace(a.GetNamespace()).Create(cmd.Context(), a, meta.CreateOptions{})
		if e != nil {
			return e
		}
		committed = true
		for _, key := range secrets {
			s, e := c.Kube.CoreV1().Secrets(a.GetNamespace()).Get(cmd.Context(), key, meta.GetOptions{})
			if e != nil {
				return e
			}
			s.OwnerReferences = []meta.OwnerReference{{APIVersion: platform.Group + "/v1alpha1", Kind: "App", Name: created.GetName(), UID: created.GetUID()}}
			if _, e = c.Kube.CoreV1().Secrets(a.GetNamespace()).Update(cmd.Context(), s, meta.UpdateOptions{}); e != nil {
				return e
			}
		}
		if o.json {
			return output(o, created.Object)
		}
		fmt.Printf("%s\t@%s\t%s\n", created.GetUID(), created.GetName(), created.GetNamespace())
		return nil
	}}
	get := &cobra.Command{Use: "get NAME", Args: cobra.ExactArgs(1), RunE: func(cmd *cobra.Command, args []string) error {
		c, e := o.clients()
		if e != nil {
			return e
		}
		v, e := c.Dynamic.Resource(platform.Apps).Namespace(o.namespace).Get(cmd.Context(), name(args[0]), meta.GetOptions{})
		if e != nil {
			return e
		}
		return output(o, v.UnstructuredContent())
	}}
	list := &cobra.Command{Use: "list", RunE: func(cmd *cobra.Command, _ []string) error {
		c, e := o.clients()
		if e != nil {
			return e
		}
		v, e := c.Dynamic.Resource(platform.Apps).Namespace(o.namespace).List(cmd.Context(), meta.ListOptions{})
		if e != nil {
			return e
		}
		return output(o, v.UnstructuredContent())
	}}
	erase := &cobra.Command{Use: "erase NAME", Args: cobra.ExactArgs(1), RunE: func(cmd *cobra.Command, args []string) error {
		c, e := o.clients()
		if e != nil {
			return e
		}
		return c.Dynamic.Resource(platform.Apps).Namespace(o.namespace).Delete(cmd.Context(), name(args[0]), meta.DeleteOptions{PropagationPolicy: platform.Ptr(meta.DeletePropagationForeground)})
	}}
	var patchFile string
	patch := &cobra.Command{Use: "patch NAME --file patch.yaml", Args: cobra.ExactArgs(1), RunE: func(cmd *cobra.Command, args []string) error {
		v, e := readDocument(patchFile)
		if e != nil {
			return e
		}
		if _, ok := v["spec"]; !ok {
			v = map[string]any{"spec": v}
		}
		b, e := json.Marshal(v)
		if e != nil {
			return e
		}
		c, e := o.clients()
		if e != nil {
			return e
		}
		out, e := c.Dynamic.Resource(platform.Apps).Namespace(o.namespace).Patch(cmd.Context(), name(args[0]), types.MergePatchType, b, meta.PatchOptions{})
		if e != nil {
			return e
		}
		return output(o, out.Object)
	}}
	patch.Flags().StringVar(&patchFile, "file", "", "App spec merge patch (null removes a field)")
	_ = patch.MarkFlagRequired("file")
	var follow bool
	logs := &cobra.Command{Use: "logs NAME", Args: cobra.ExactArgs(1), RunE: func(cmd *cobra.Command, args []string) error {
		c, e := o.clients()
		if e != nil {
			return e
		}
		a, e := c.Dynamic.Resource(platform.Apps).Namespace(o.namespace).Get(cmd.Context(), name(args[0]), meta.GetOptions{})
		if e != nil {
			return e
		}
		pods, e := c.Kube.CoreV1().Pods(o.namespace).List(cmd.Context(), meta.ListOptions{LabelSelector: platform.AppUID + "=" + string(a.GetUID())})
		if e != nil {
			return e
		}
		for _, p := range pods.Items {
			endpoint := p.Annotations[platform.Runner]
			id := platform.ID(7, string(p.UID))
			if endpoint == "" {
				continue
			}
			if physical := p.Annotations[platform.RunnerPod]; physical != "" {
				local, stop, e := forward(cmd.Context(), c, physical, 8081)
				if e != nil {
					return e
				}
				defer stop()
				endpoint = local
			}
			client := platform.NewRunner(endpoint)
			stream, e := client.Instances.Tail(cmd.Context(), connect.NewRequest(&wf.InstanceTailRequest{Ref: &wf.InstanceRef{Key: &wf.InstanceRef_Id{Id: id}}, Follow: follow}))
			if e != nil {
				return e
			}
			for stream.Receive() {
				entry := stream.Msg()
				if o.json {
					data, e := protojson.Marshal(entry)
					if e != nil {
						return e
					}
					fmt.Println(string(data))
				} else {
					if _, e := os.Stdout.Write(entry.Data); e != nil {
						return e
					}
				}
			}
			return stream.Err()
		}
		return errors.New("App has no running execution")
	}}
	logs.Flags().BoolVarP(&follow, "follow", "f", false, "Follow the selected execution")
	app.AddCommand(add, get, list, erase, patch, logs)
	return app
}
