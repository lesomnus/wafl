package main

import (
	"bytes"
	"fmt"
	"os"
	"os/exec"
	"path/filepath"
	"sort"
	"strings"

	"google.golang.org/protobuf/proto"
	"google.golang.org/protobuf/types/descriptorpb"
	"google.golang.org/protobuf/types/pluginpb"
)

// Both implementations consume the same paydirt-generated service descriptors.
// No second handwritten copy of the Instance/Traffic RPC contract is maintained.
func generateGo(root string, set *descriptorpb.FileDescriptorSet, check bool) error {
	const module = "github.com/lesomnus/wafl/kubernetes"
	req := &pluginpb.CodeGeneratorRequest{ProtoFile: set.File}
	parameters := []string{"module=" + module}
	for _, f := range set.File {
		if strings.HasPrefix(f.GetName(), "google/protobuf/") {
			continue
		}
		if strings.HasPrefix(f.GetName(), "patch/") {
			// Use the reference patch engine's bindings, avoiding duplicate
			// registration of patch descriptors in the controller/client binary.
			parameters = append(parameters, "M"+f.GetName()+"=github.com/lesomnus/protobuf-patch/patchpb")
			continue
		}
		req.FileToGenerate = append(req.FileToGenerate, f.GetName())
		parameters = append(parameters, "M"+f.GetName()+"="+module+"/gen/"+strings.ReplaceAll(f.GetPackage(), ".", "/"))
	}
	sort.Strings(req.FileToGenerate)
	outputs := map[string][]byte{}
	for _, plugin := range []string{"protoc-gen-go", "protoc-gen-connect-go"} {
		req.Parameter = proto.String(strings.Join(parameters, ","))
		input, err := proto.Marshal(req)
		if err != nil {
			return err
		}
		cmd := exec.Command(plugin)
		cmd.Stdin = bytes.NewReader(input)
		output, err := cmd.Output()
		if err != nil {
			return fmt.Errorf("%s: %w", plugin, err)
		}
		var response pluginpb.CodeGeneratorResponse
		if err := proto.Unmarshal(output, &response); err != nil {
			return err
		}
		if response.GetError() != "" {
			return fmt.Errorf("%s: %s", plugin, response.GetError())
		}
		for _, file := range response.File {
			outputs[file.GetName()] = []byte(file.GetContent())
		}
	}
	for name, data := range outputs {
		path := filepath.Join(root, "kubernetes", name)
		if check {
			old, err := os.ReadFile(path)
			if err != nil || !bytes.Equal(old, data) {
				return fmt.Errorf("generated Go file is stale: %s", name)
			}
		} else {
			if err := os.MkdirAll(filepath.Dir(path), 0755); err != nil {
				return err
			}
			if err := os.WriteFile(path, data, 0644); err != nil {
				return err
			}
		}
	}
	return nil
}
