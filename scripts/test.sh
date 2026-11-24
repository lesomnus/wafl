#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
cargo build --locked --target wasm32-wasip2 -p wafl-example-hello -p wafl-test-guests
target=$(cargo metadata --format-version 1 --no-deps | python3 -c 'import json,sys; print(json.load(sys.stdin)["target_directory"])')
export WAFL_TEST_GUEST_DIR="$target/wasm32-wasip2/debug"
cargo test --locked -p wafl-runner "$@"
