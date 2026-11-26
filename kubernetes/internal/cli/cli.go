package cli

import (
	"context"
	"fmt"
	"net/http"
	"os"
	"time"

	"github.com/lesomnus/wafl/kubernetes/internal/platform"
	"github.com/spf13/cobra"
)

type options struct {
	kubeconfig, context, namespace, system string
	json                                   bool
}

func (o *options) clients() (*platform.Clients, error) { return platform.Open(o.kubeconfig, o.context) }
func Command() *cobra.Command {
	o := &options{}
	root := &cobra.Command{Use: "wafl", Short: "WAFL Kubernetes application platform", SilenceUsage: true, SilenceErrors: true}
	f := root.PersistentFlags()
	f.StringVar(&o.kubeconfig, "kubeconfig", "", "Kubeconfig file (otherwise standard Kubernetes discovery)")
	f.StringVar(&o.context, "context", "", "Kubernetes context")
	f.StringVarP(&o.namespace, "namespace", "n", "default", "App namespace")
	f.StringVar(&o.system, "system-namespace", "wafl-system", "WAFL infrastructure namespace")
	f.BoolVar(&o.json, "json", false, "Print Kubernetes JSON")
	serve := &cobra.Command{Use: "serve", Short: "Run a WAFL service in the foreground"}
	controller := &platform.Controller{}
	control := &cobra.Command{Use: "controller", Short: "Run the Kubernetes App controller", RunE: func(cmd *cobra.Command, _ []string) error {
		c, e := o.clients()
		if e != nil {
			return e
		}
		controller.Clients = c
		controller.Namespace = o.system
		if !cmd.Flags().Changed("control") {
			controller.Control = fmt.Sprintf("http://wafl-controller.%s.svc:8082", o.system)
		}
		if !cmd.Flags().Changed("activate") {
			controller.Activate = fmt.Sprintf("http://wafl-controller.%s.svc:8083", o.system)
		}
		hostname, _ := os.Hostname()
		controller.Identity = fmt.Sprintf("%s-%d", hostname, os.Getpid())
		return controller.Serve(cmd.Context())
	}}
	cf := control.Flags()
	cf.StringVar(&controller.Listen, "listen", "0.0.0.0:8082", "Controller API and Traefik configuration address")
	cf.StringVar(&controller.ActivationListen, "activation-listen", "0.0.0.0:8083", "HTTP/gRPC activation address")
	cf.StringVar(&controller.Control, "control", "http://wafl-controller.wafl-system.svc:8082", "Controller origin reachable by Traefik")
	cf.StringVar(&controller.Activate, "activate", "http://wafl-controller.wafl-system.svc:8083", "Activator origin reachable by Traefik")
	cf.StringVar(&controller.Entrypoint, "entrypoint", "web", "Traefik entrypoint")
	var node, runner string
	agent := &cobra.Command{Use: "node", Short: "Connect one runner to a Virtual Kubelet node", RunE: func(cmd *cobra.Command, _ []string) error {
		if node == "" {
			return fmt.Errorf("--node is required")
		}
		c, e := o.clients()
		if e != nil {
			return e
		}
		return platform.NewProvider(c, node, runner, o.system).Serve(cmd.Context())
	}}
	agent.Flags().StringVar(&node, "node", "", "Stable virtual node name for this runner Pod")
	agent.Flags().StringVar(&runner, "runner", "http://127.0.0.1:8081", "Runner control origin reachable from controllers and CLI")
	serve.AddCommand(control, agent)
	root.AddCommand(serve, appCommand(o), installCommand(o), devCommand(o), trafficCommand(o))
	var directory, listen string
	assets := &cobra.Command{Use: "assets", Hidden: true, RunE: func(cmd *cobra.Command, _ []string) error {
		server := &http.Server{Addr: listen, Handler: http.FileServer(http.Dir(directory)), ReadHeaderTimeout: 10 * time.Second}
		go func() { <-cmd.Context().Done(); _ = server.Shutdown(context.Background()) }()
		e := server.ListenAndServe()
		if e == http.ErrServerClosed {
			return nil
		}
		return e
	}}
	assets.Flags().StringVar(&directory, "directory", "/assets", "Bundle directory")
	assets.Flags().StringVar(&listen, "listen", ":8090", "Listen address")
	root.AddCommand(assets)
	var drainNode string
	var drainTimeout time.Duration
	drain := &cobra.Command{Use: "node-drain", Hidden: true, RunE: func(cmd *cobra.Command, _ []string) error {
		c, e := o.clients()
		if e != nil {
			return e
		}
		ctx, cancel := context.WithTimeout(cmd.Context(), drainTimeout)
		defer cancel()
		return platform.DrainNode(ctx, c, drainNode)
	}}
	drain.Flags().StringVar(&drainNode, "node", "wafl-"+os.Getenv("POD_NAME"), "Virtual node name")
	drain.Flags().DurationVar(&drainTimeout, "timeout", 45*time.Second, "Maximum graceful runner drain")
	root.AddCommand(drain)
	return root
}
