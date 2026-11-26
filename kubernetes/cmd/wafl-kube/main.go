package main

import (
	"context"
	"fmt"
	"os"
	"os/signal"
	"syscall"

	"github.com/lesomnus/wafl/kubernetes/internal/bootstrap"
	"github.com/lesomnus/wafl/kubernetes/internal/cli"
	"github.com/lesomnus/wafl/kubernetes/internal/server"
)

func main() {
	ctx, stop := signal.NotifyContext(context.Background(), os.Interrupt, syscall.SIGTERM)
	defer stop()
	command := cli.Command()
	command.AddCommand(server.Command(), server.Assets(), server.Drain(), bootstrap.Command())
	if len(os.Args) > 1 && os.Args[1] == "bootstrap" {
		// Bootstrap is invoked directly by operators, never dispatched by wafl.
		command.Use = "wafl-kube"
	}
	if e := command.ExecuteContext(ctx); e != nil {
		fmt.Fprintln(os.Stderr, e)
		os.Exit(1)
	}
}
