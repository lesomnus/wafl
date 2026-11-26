// Package bootstrap is an operator tool, separate from the WAFL API client.
package bootstrap

import (
	"github.com/lesomnus/wafl/kubernetes/internal/platform"
	"github.com/spf13/cobra"
)

type options struct{ kubeconfig, context, system, tenant string }

func (o *options) clients() (*platform.Clients, error) { return platform.Open(o.kubeconfig, o.context) }
func Command() *cobra.Command {
	o := &options{}
	root := &cobra.Command{Use: "bootstrap", Short: "Operator tools for installing WAFL on Kubernetes"}
	root.PersistentFlags().StringVar(&o.kubeconfig, "kubeconfig", "", "Operator kubeconfig")
	root.PersistentFlags().StringVar(&o.context, "context", "", "Operator Kubernetes context")
	root.PersistentFlags().StringVar(&o.system, "system-namespace", "wafl-system", "Infrastructure namespace")
	root.PersistentFlags().StringVar(&o.tenant, "tenant", "default", "Tenant for the optional local App")
	root.AddCommand(installCommand(o), devCommand(o))
	return root
}
