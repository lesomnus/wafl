package cli

import (
	"errors"

	wf "github.com/lesomnus/wafl/kubernetes/gen/wafl"
	"github.com/spf13/cobra"
)

func trafficCommand(o *options) *cobra.Command {
	root := &cobra.Command{Use: "traffic", Short: "Inspect gateway observers through the controller"}
	root.AddCommand(&cobra.Command{Use: "list", Args: cobra.NoArgs, RunE: func(cmd *cobra.Command, _ []string) error {
		q := &wf.TrafficListRequest{}
		for {
			r, e := o.traffic().List(cmd.Context(), request(o, q))
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
	}})
	var withdrawn bool
	close := &cobra.Command{Use: "close ID", Args: cobra.ExactArgs(1), Short: "Recover an observer after all its connections have stopped", RunE: func(cmd *cobra.Command, args []string) error {
		if !withdrawn {
			return errors.New("--withdrawn confirms the observer and all its connections have stopped")
		}
		id, e := parseID(args[0], 13)
		if e != nil {
			return e
		}
		r, e := o.traffic().Close(cmd.Context(), request(o, &wf.TrafficCloseRequest{Ref: &wf.TrafficRef{Key: &wf.TrafficRef_Id{Id: id}}, Withdrawn: true}))
		if e != nil {
			return e
		}
		return output(r.Msg)
	}}
	close.Flags().BoolVar(&withdrawn, "withdrawn", false, "Confirm completed observer and connection withdrawal")
	root.AddCommand(close)
	return root
}
