// Package install embeds the same manifests published for kubectl users.
package install

import _ "embed"

//go:embed platform.yaml
var Platform []byte

//go:embed apps.json
var Apps []byte

//go:embed assets.yaml
var Assets []byte
