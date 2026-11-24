#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
cargo build --locked -p wafl-example-hello --target wasm32-wasip2
bundle_target=$(cargo metadata --format-version 1 --no-deps | python3 -c 'import json,sys; print(json.load(sys.stdin)["target_directory"])')
bundle_output=${1:-"$bundle_target/hello.tar.gz"}
bundle_stage=$(mktemp -d)
trap 'rm -rf "$bundle_stage"' EXIT
cp "$bundle_target/wasm32-wasip2/debug/wafl-example-hello.wasm" "$bundle_stage/main.wasm"
cp -R examples/hello/data "$bundle_stage/data"
mkdir -p "$(dirname "$bundle_output")"
tar -czf "$bundle_output" -C "$bundle_stage" main.wasm data
printf '%s\n' "$bundle_output"
