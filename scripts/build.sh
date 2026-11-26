#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
cargo build --locked --bin wafl "$@"
target=$(cargo metadata --format-version 1 --no-deps | python3 -c 'import json,sys; print(json.load(sys.stdin)["target_directory"])')
profile=debug
if [[ " $* " == *" --release "* ]]; then profile=release; fi
(cd kubernetes && CGO_ENABLED=0 go build -trimpath -ldflags='-s -w' -o "$target/$profile/wafl-kube" ./cmd/wafl-kube)
