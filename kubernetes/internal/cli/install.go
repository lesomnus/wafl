package cli

import (
	"bytes"
	"context"
	"encoding/json"
	"fmt"
	"io"
	"net"
	"net/url"
	"os"
	"os/exec"
	"path/filepath"
	"strings"
	"time"

	"github.com/lesomnus/wafl/kubernetes/install"
	"github.com/lesomnus/wafl/kubernetes/internal/platform"
	"github.com/spf13/cobra"
	apierrors "k8s.io/apimachinery/pkg/api/errors"
	meta "k8s.io/apimachinery/pkg/apis/meta/v1"
	"k8s.io/apimachinery/pkg/apis/meta/v1/unstructured"
	"k8s.io/apimachinery/pkg/runtime/schema"
	"k8s.io/apimachinery/pkg/types"
	yaml "k8s.io/apimachinery/pkg/util/yaml"
	"k8s.io/client-go/tools/clientcmd"
)

func run(ctx context.Context, name string, args ...string) error {
	cmd := exec.CommandContext(ctx, name, args...)
	cmd.Stdout = os.Stdout
	cmd.Stderr = os.Stderr
	return cmd.Run()
}

var resources = map[string]schema.GroupVersionResource{
	"Namespace": {Version: "v1", Resource: "namespaces"}, "ServiceAccount": {Version: "v1", Resource: "serviceaccounts"}, "ConfigMap": {Version: "v1", Resource: "configmaps"}, "Service": {Version: "v1", Resource: "services"},
	"Deployment": {Group: "apps", Version: "v1", Resource: "deployments"}, "ClusterRole": {Group: "rbac.authorization.k8s.io", Version: "v1", Resource: "clusterroles"}, "ClusterRoleBinding": {Group: "rbac.authorization.k8s.io", Version: "v1", Resource: "clusterrolebindings"}, "CustomResourceDefinition": {Group: "apiextensions.k8s.io", Version: "v1", Resource: "customresourcedefinitions"},
}

func apply(ctx context.Context, c *platform.Clients, data []byte) error {
	decoder := yaml.NewYAMLOrJSONDecoder(bytes.NewReader(data), 4096)
	for {
		var value map[string]any
		if e := decoder.Decode(&value); e == io.EOF {
			return nil
		} else if e != nil {
			return e
		}
		if len(value) == 0 {
			continue
		}
		u := &unstructured.Unstructured{Object: value}
		gvr, ok := resources[u.GetKind()]
		if !ok {
			return fmt.Errorf("unknown installer resource %s", u.GetKind())
		}
		body, e := json.Marshal(value)
		if e != nil {
			return e
		}
		_, e = c.Dynamic.Resource(gvr).Namespace(u.GetNamespace()).Patch(ctx, u.GetName(), types.ApplyPatchType, body, meta.PatchOptions{FieldManager: "wafl-install", Force: platform.Ptr(true)})
		if e != nil {
			return e
		}
	}
}
func installPlatform(ctx context.Context, o *options, image, gateway string, dev bool) error {
	c, e := o.clients()
	if e != nil {
		return e
	}
	installed, e := c.Kube.RbacV1().ClusterRoleBindings().Get(ctx, "wafl-controller", meta.GetOptions{})
	if e != nil && !apierrors.IsNotFound(e) {
		return e
	}
	if e == nil {
		for _, subject := range installed.Subjects {
			if subject.Kind == "ServiceAccount" && subject.Namespace != o.system {
				return fmt.Errorf("WAFL is already installed in namespace %s; use --system-namespace %s", subject.Namespace, subject.Namespace)
			}
		}
	}
	if e = apply(ctx, c, install.Apps); e != nil {
		return e
	}
	manifest := strings.ReplaceAll(string(install.Platform), "ghcr.io/lesomnus/wafl:latest", image)
	manifest = strings.ReplaceAll(manifest, "ghcr.io/lesomnus/wafl-traefik:latest", gateway)
	manifest = strings.ReplaceAll(manifest, "wafl-system", o.system)
	if dev {
		manifest = strings.ReplaceAll(manifest, "          image:", "          imagePullPolicy: IfNotPresent\n          image:")
	}
	if e = apply(ctx, c, []byte(manifest)); e != nil {
		return e
	}
	if dev {
		manifest = strings.ReplaceAll(string(install.Assets), "ghcr.io/lesomnus/wafl:latest", image)
		manifest = strings.ReplaceAll(manifest, "wafl-system", o.system)
		manifest = strings.ReplaceAll(manifest, "          image:", "          imagePullPolicy: IfNotPresent\n          image:")
		if e = apply(ctx, c, []byte(manifest)); e != nil {
			return e
		}
	}
	return nil
}
func installCommand(o *options) *cobra.Command {
	var image, gateway string
	cmd := &cobra.Command{Use: "install", Short: "Install WAFL into the selected Kubernetes cluster", RunE: func(cmd *cobra.Command, _ []string) error {
		return installPlatform(cmd.Context(), o, image, gateway, false)
	}}
	cmd.Flags().StringVar(&image, "image", "ghcr.io/lesomnus/wafl:latest", "WAFL runner/controller image")
	cmd.Flags().StringVar(&gateway, "gateway-image", "ghcr.io/lesomnus/wafl-traefik:latest", "Traefik image containing the WAFL plugin")
	return cmd
}
func devCommand(o *options) *cobra.Command {
	var cluster, image, gateway string
	var port int
	var load bool
	cmd := &cobra.Command{Use: "dev [app.yaml]", Short: "Create/reuse a local kind cluster and install the complete WAFL platform", Args: cobra.MaximumNArgs(1), RunE: func(cmd *cobra.Command, args []string) error {
		ctx := cmd.Context()
		gatewayHost, bindHost := "127.0.0.1", "127.0.0.1"
		if dockerURL, e := url.Parse(os.Getenv("DOCKER_HOST")); e == nil && dockerURL.Scheme == "tcp" && dockerURL.Hostname() != "localhost" && dockerURL.Hostname() != "127.0.0.1" && dockerURL.Hostname() != "::1" {
			gatewayHost = dockerURL.Hostname()
			bindHost = "0.0.0.0"
		}
		if _, e := exec.LookPath("kind"); e != nil {
			return fmt.Errorf("wafl dev requires kind and Docker: %w", e)
		}
		if _, e := exec.LookPath("docker"); e != nil {
			return e
		}
		root, e := os.UserCacheDir()
		if e != nil {
			return e
		}
		root = filepath.Join(root, "wafl", "dev", cluster)
		if e = os.MkdirAll(root, 0700); e != nil {
			return e
		}
		config := filepath.Join(root, "kind.yaml")
		kubeconfig := filepath.Join(root, "kubeconfig")
		clusters, e := exec.CommandContext(ctx, "kind", "get", "clusters").Output()
		if e != nil {
			return e
		}
		found := false
		for _, v := range strings.Fields(string(clusters)) {
			if v == cluster {
				found = true
			}
		}
		if !found {
			data := fmt.Sprintf("kind: Cluster\napiVersion: kind.x-k8s.io/v1alpha4\nnetworking:\n  apiServerAddress: %s\nnodes:\n- role: control-plane\n  extraPortMappings:\n  - containerPort: 30080\n    hostPort: %d\n    listenAddress: %s\n    protocol: TCP\n", bindHost, port, bindHost)
			if e = os.WriteFile(config, []byte(data), 0600); e != nil {
				return e
			}
			if e = run(ctx, "kind", "create", "cluster", "--name", cluster, "--config", config, "--kubeconfig", kubeconfig, "--wait", "120s"); e != nil {
				return e
			}
		}
		if e = run(ctx, "kind", "export", "kubeconfig", "--name", cluster, "--kubeconfig", kubeconfig); e != nil {
			return e
		}
		if gatewayHost != "127.0.0.1" {
			config, e := clientcmd.LoadFromFile(kubeconfig)
			if e != nil {
				return e
			}
			for _, c := range config.Clusters {
				u, e := url.Parse(c.Server)
				if e != nil {
					return e
				}
				c.TLSServerName = u.Hostname()
				u.Host = net.JoinHostPort(gatewayHost, u.Port())
				c.Server = u.String()
			}
			if e = clientcmd.WriteToFile(*config, kubeconfig); e != nil {
				return e
			}
		}
		o.kubeconfig = kubeconfig
		o.context = "kind-" + cluster
		if load {
			if e = run(ctx, "kind", "load", "docker-image", "--name", cluster, image, gateway); e != nil {
				return e
			}
		}
		if e = installPlatform(ctx, o, image, gateway, true); e != nil {
			return e
		}
		c, e := o.clients()
		if e != nil {
			return e
		}
		if load {
			images := map[string]string{"wafl-controller": image, "wafl-runner": image, "wafl-assets": image, "wafl-traefik": gateway}
			for name, tag := range images {
				identity, e := exec.CommandContext(ctx, "docker", "image", "inspect", "--format", "{{.Id}}", tag).Output()
				if e != nil {
					return e
				}
				d, e := c.Kube.AppsV1().Deployments(o.system).Get(ctx, name, meta.GetOptions{})
				if e != nil {
					return e
				}
				key := platform.Group + "/devImage"
				value := strings.TrimSpace(string(identity))
				if d.Spec.Template.Annotations[key] == value {
					continue
				}
				if d.Spec.Template.Annotations == nil {
					d.Spec.Template.Annotations = map[string]string{}
				}
				d.Spec.Template.Annotations[key] = value
				if _, e = c.Kube.AppsV1().Deployments(o.system).Update(ctx, d, meta.UpdateOptions{}); e != nil {
					return e
				}
			}
		}
		ns, e := c.Kube.CoreV1().Namespaces().Get(ctx, o.system, meta.GetOptions{})
		if e != nil {
			return e
		}
		if ns.Annotations == nil {
			ns.Annotations = map[string]string{}
		}
		ns.Annotations[platform.Group+"/dev"] = cluster
		if _, e = c.Kube.CoreV1().Namespaces().Update(ctx, ns, meta.UpdateOptions{}); e != nil {
			return e
		}
		deadline := time.Now().Add(3 * time.Minute)
		for {
			d, e := c.Kube.AppsV1().Deployments(o.system).List(ctx, meta.ListOptions{})
			complete := e == nil && len(d.Items) >= 4
			if e == nil {
				for _, v := range d.Items {
					if v.Status.ObservedGeneration < v.Generation || v.Status.UpdatedReplicas != *v.Spec.Replicas || v.Status.AvailableReplicas < *v.Spec.Replicas {
						complete = false
					}
				}
			}
			if complete {
				nodes, e := c.Kube.CoreV1().Nodes().List(ctx, meta.ListOptions{LabelSelector: platform.Runtime + "=wasmtime"})
				ready := int32(0)
				if e == nil {
					for _, node := range nodes.Items {
						if node.Spec.Unschedulable {
							continue
						}
						for _, condition := range node.Status.Conditions {
							if condition.Type == "Ready" && condition.Status == "True" {
								ready++
							}
						}
					}
				}
				for _, deployment := range d.Items {
					if deployment.Name == "wafl-runner" && ready < *deployment.Spec.Replicas {
						complete = false
					}
				}
			}
			if complete {
				break
			}
			if time.Now().After(deadline) {
				return fmt.Errorf("platform startup timed out; inspect kubectl --kubeconfig %s get pods -n %s", kubeconfig, o.system)
			}
			select {
			case <-ctx.Done():
				return ctx.Err()
			case <-time.After(time.Second):
			}
		}
		if len(args) > 0 {
			add := appCommand(o)
			add.SetArgs(append([]string{"add"}, args...))
			if e = add.ExecuteContext(ctx); e != nil {
				return e
			}
		}
		fmt.Printf("WAFL ready at http://%s\nKubeconfig: %s\nUse wafl --kubeconfig %s app add app.yaml\n", net.JoinHostPort(gatewayHost, fmt.Sprint(port)), kubeconfig, kubeconfig)
		return nil
	}}
	f := cmd.Flags()
	f.StringVar(&cluster, "name", "wafl", "Dedicated kind cluster name")
	f.IntVar(&port, "port", 8080, "Local gateway port (chosen when creating the cluster)")
	f.StringVar(&image, "image", "ghcr.io/lesomnus/wafl:latest", "WAFL image")
	f.StringVar(&gateway, "gateway-image", "ghcr.io/lesomnus/wafl-traefik:latest", "Gateway image")
	f.BoolVar(&load, "load", false, "Load both images from the local Docker daemon")
	return cmd
}
