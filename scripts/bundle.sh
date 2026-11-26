#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
bundle_example=${2:-hello}
case "$bundle_example" in
  hello|redis|tcp) ;;
  *) printf 'unknown example: %s\n' "$bundle_example" >&2; exit 1 ;;
esac
cargo build --locked -p "wafl-example-$bundle_example" --target wasm32-wasip2
bundle_target=$(cargo metadata --format-version 1 --no-deps | python3 -c 'import json,sys; print(json.load(sys.stdin)["target_directory"])')
bundle_output=${1:-"$bundle_target/$bundle_example.tar.gz"}
bundle_stage=$(mktemp -d)
trap 'rm -rf "$bundle_stage"' EXIT
cp "$bundle_target/wasm32-wasip2/debug/wafl-example-$bundle_example.wasm" "$bundle_stage/main.wasm"
mkdir "$bundle_stage/data"
if [ -d "examples/$bundle_example/data" ]; then
  cp -R "examples/$bundle_example/data/." "$bundle_stage/data/"
fi
mkdir -p "$(dirname "$bundle_output")"
tar -czf "$bundle_output" -C "$bundle_stage" main.wasm data
printf '%s\n' "$bundle_output"
