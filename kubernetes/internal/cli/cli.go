// Package cli talks only to WAFL APIs. It neither loads kubeconfig nor imports
// Kubernetes clients, shells out to kubectl, or discovers runner Pod addresses.
package cli

import (
	"encoding/hex"
	"errors"
	"fmt"
	"net/http"
	"os"
	"strings"

	"connectrpc.com/connect"
	"github.com/lesomnus/wafl/kubernetes/gen/wafl/waflconnect"
	"github.com/spf13/cobra"
	"google.golang.org/protobuf/encoding/protojson"
	"google.golang.org/protobuf/proto"
)

type options struct {
	endpoint, tenant string
	json             bool
}

func env(name, fallback string) string {
	if v := os.Getenv(name); v != "" {
		return v
	}
	return fallback
}
func Command() *cobra.Command {
	o := &options{}
	root := &cobra.Command{Use: "wafl", Short: "Manage applications through the WAFL controller API", SilenceUsage: true, SilenceErrors: true}
	f := root.PersistentFlags()
	f.StringVar(&o.endpoint, "endpoint", env("WAFL_ENDPOINT", "http://127.0.0.1:8082"), "Controller API endpoint")
	f.StringVar(&o.tenant, "tenant", env("WAFL_TENANT", "default"), "Tenant alias or ID")
	f.BoolVar(&o.json, "json", false, "Print protobuf JSON")
	root.AddCommand(appCommand(o), trafficCommand(o))
	return root
}
func (o *options) apps() waflconnect.AppServiceClient {
	return waflconnect.NewAppServiceClient(http.DefaultClient, o.endpoint, connect.WithProtoJSON())
}
func (o *options) traffic() waflconnect.TrafficServiceClient {
	return waflconnect.NewTrafficServiceClient(http.DefaultClient, o.endpoint, connect.WithProtoJSON())
}
func request[T any](o *options, v *T) *connect.Request[T] {
	r := connect.NewRequest(v)
	r.Header().Set("wafl-tenant", strings.TrimPrefix(o.tenant, "@"))
	return r
}
func output(v proto.Message) error {
	b, e := protojson.MarshalOptions{Multiline: true, Indent: "  "}.Marshal(v)
	if e == nil {
		_, e = fmt.Fprintln(os.Stdout, string(b))
	}
	return e
}
func uuid(id []byte) string {
	if len(id) != 16 {
		return ""
	}
	h := hex.EncodeToString(id)
	return h[:8] + "-" + h[8:12] + "-" + h[12:16] + "-" + h[16:20] + "-" + h[20:]
}
func parseID(value string, domain byte) ([]byte, error) {
	if len(value) != 36 || value[8] != '-' || value[13] != '-' || value[18] != '-' || value[23] != '-' {
		return nil, errors.New("invalid UUID")
	}
	b, e := hex.DecodeString(strings.ReplaceAll(value, "-", ""))
	if e != nil || len(b) != 16 || b[6]>>4 != 8 || b[8]&0xc0 != 0x80 || b[9] != domain {
		return nil, errors.New("invalid resource ID")
	}
	return b, nil
}
