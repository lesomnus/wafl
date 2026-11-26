package server

import (
	"context"
	"fmt"
	"net/http"
	"os"
	"time"

	"github.com/lesomnus/wafl/kubernetes/internal/platform"
	"github.com/spf13/cobra"
)

type options struct{ kubeconfig, context, system string }

func (o *options) clients() (*platform.Clients, error) { return platform.Open(o.kubeconfig, o.context) }
func flags(command *cobra.Command, o *options) {
	f := command.PersistentFlags()
	f.StringVar(&o.kubeconfig, "kubeconfig", "", "Server kubeconfig; defaults to in-cluster credentials")
	f.StringVar(&o.context, "context", "", "Server Kubernetes context")
	f.StringVar(&o.system, "system-namespace", "wafl-system", "Infrastructure namespace")
}
func Command() *cobra.Command {
	o := &options{}
	root := &cobra.Command{Use: "serve", Short: "Run a WAFL platform service"}
	flags(root, o)
	controller := &platform.Controller{}
	control := &cobra.Command{Use: "controller", RunE: func(cmd *cobra.Command, _ []string) error {
		c, e := o.clients()
		if e != nil {
			return e
		}
		controller.Clients = c
		controller.Namespace = o.system
		hostname, _ := os.Hostname()
		controller.Identity = fmt.Sprintf("%s-%d", hostname, os.Getpid())
		return controller.Serve(cmd.Context())
	}}
	f := control.Flags()
	f.StringVar(&controller.Listen, "listen", "0.0.0.0:8082", "Controller management API and proxy routing address")
	f.StringVar(&controller.Assets, "assets", "", "Optional shared asset store origin for file uploads")
	f.DurationVar(&controller.RunnerTimeout, "runner-timeout", 3*time.Second, "Deadline for each runner during catalog queries")
	f.IntVar(&controller.RunnerParallel, "runner-parallel", 16, "Maximum concurrent runner catalog queries")
	var name, runner string
	node := &cobra.Command{Use: "node", RunE: func(cmd *cobra.Command, _ []string) error {
		if name == "" {
			return fmt.Errorf("--node is required")
		}
		c, e := o.clients()
		if e != nil {
			return e
		}
		return platform.NewProvider(c, name, runner, o.system).Serve(cmd.Context())
	}}
	node.Flags().StringVar(&name, "node", "", "Virtual node name for this runner")
	node.Flags().StringVar(&runner, "runner", "http://127.0.0.1:8081", "Runner API origin reachable by controllers")
	root.AddCommand(control, node)
	return root
}
func Assets() *cobra.Command {
	var directory, listen string
	cmd := &cobra.Command{Use: "assets", Hidden: true, RunE: func(cmd *cobra.Command, _ []string) error {
		handler, e := platform.AssetHandler(directory)
		if e != nil {
			return e
		}
		server := &http.Server{Addr: listen, Handler: handler, ReadHeaderTimeout: 10 * time.Second}
		go func() { <-cmd.Context().Done(); _ = server.Close() }()
		e = server.ListenAndServe()
		if e == http.ErrServerClosed {
			return nil
		}
		return e
	}}
	cmd.Flags().StringVar(&directory, "directory", "/assets", "Shared bundle directory")
	cmd.Flags().StringVar(&listen, "listen", ":8090", "Artifact server address")
	return cmd
}
func Drain() *cobra.Command {
	o := &options{}
	var name string
	var timeout time.Duration
	cmd := &cobra.Command{Use: "node-drain", Hidden: true, RunE: func(cmd *cobra.Command, _ []string) error {
		c, e := o.clients()
		if e != nil {
			return e
		}
		ctx, cancel := context.WithTimeout(cmd.Context(), timeout)
		defer cancel()
		return platform.DrainNode(ctx, c, name)
	}}
	flags(cmd, o)
	cmd.Flags().StringVar(&name, "node", "wafl-"+os.Getenv("POD_NAME"), "Virtual node name")
	cmd.Flags().DurationVar(&timeout, "timeout", 45*time.Second, "Maximum graceful drain")
	return cmd
}
