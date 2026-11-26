package cli

import (
	"encoding/json"
	"errors"
	"fmt"

	"github.com/lesomnus/wafl/kubernetes/internal/platform"
	"github.com/spf13/cobra"
	meta "k8s.io/apimachinery/pkg/apis/meta/v1"
)

func trafficCommand(o *options) *cobra.Command {
	root := &cobra.Command{Use: "traffic", Short: "Inspect durable gateway observers"}
	root.AddCommand(&cobra.Command{Use: "list", RunE: func(cmd *cobra.Command, _ []string) error {
		c, e := o.clients()
		if e != nil {
			return e
		}
		v, e := c.Kube.CoreV1().ConfigMaps(o.system).List(cmd.Context(), meta.ListOptions{LabelSelector: platform.Kind + "=traffic"})
		if e != nil {
			return e
		}
		for _, cm := range v.Items {
			fmt.Println(cm.Name, cm.Data["value"])
		}
		return nil
	}})
	var withdrawn bool
	close := &cobra.Command{Use: "close CONFIGMAP", Short: "Recover a stopped observer after all its connections have ended", Args: cobra.ExactArgs(1), RunE: func(cmd *cobra.Command, args []string) error {
		if !withdrawn {
			return errors.New("--withdrawn confirms the observer and all its connections have stopped")
		}
		c, e := o.clients()
		if e != nil {
			return e
		}
		cm, e := c.Kube.CoreV1().ConfigMaps(o.system).Get(cmd.Context(), args[0], meta.GetOptions{})
		if e != nil {
			return e
		}
		if cm.Labels[platform.Kind] != "traffic" {
			return errors.New("not a WAFL observer")
		}
		var v map[string]any
		if e = json.Unmarshal([]byte(cm.Data["value"]), &v); e != nil {
			return e
		}
		v["closed"] = true
		v["draining"] = true
		v["active"] = 0
		data, _ := json.Marshal(v)
		cm.Data["value"] = string(data)
		_, e = c.Kube.CoreV1().ConfigMaps(o.system).Update(cmd.Context(), cm, meta.UpdateOptions{})
		return e
	}}
	close.Flags().BoolVar(&withdrawn, "withdrawn", false, "Confirm completed observer and connection withdrawal")
	root.AddCommand(close)
	return root
}
