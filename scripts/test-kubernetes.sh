#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
# This creates a dedicated disposable kind cluster. It never uses the caller's
# current Kubernetes context. Build hosts may use a remote Docker daemon.
cluster=${WAFL_TEST_CLUSTER:-wafl-test}
image=${WAFL_TEST_IMAGE:-wafl:kubernetes-test}
gateway=${WAFL_TEST_GATEWAY_IMAGE:-wafl-traefik:kubernetes-test}
export WAFL_TEST_CLUSTER="$cluster"
if [ -z "${WAFL_TEST_KUBECONFIG:-}" ]; then
  bash scripts/build.sh
  cargo build --locked --target wasm32-wasip2 -p wafl-test-guests
  target=$(cargo metadata --format-version 1 --no-deps | python3 -c 'import json,sys; print(json.load(sys.stdin)["target_directory"])')
  export WAFL_TEST_GUEST_DIR="$target/wasm32-wasip2/debug"
  export WAFL_TEST_CLI="$target/debug/wafl"
  stage=$(mktemp -d)
  trap 'rm -rf "$stage"' EXIT
  cp "$target/debug/wafl" "$target/debug/wafl-kube" "$stage/"
  # Test the locally compiled binaries. Production uses the multi-stage
  # Dockerfile, whose Debian build/runtime versions match each other.
  cp kubernetes/tests/Dockerfile "$stage/Dockerfile"
  docker build -t "$image" "$stage"
  docker build -f Dockerfile.traefik -t "$gateway" .
  "$WAFL_TEST_CLI" dev --name "$cluster" --image "$image" --gateway-image "$gateway" --load --port "${WAFL_TEST_PORT:-18080}"
  export WAFL_TEST_KUBECONFIG="${XDG_CACHE_HOME:-$HOME/.cache}/wafl/dev/$cluster/kubeconfig"
fi
# Tests use kubectl port-forward, so gateway requests work with remote Docker.
(cd kubernetes && go test -race ./tests -count=1 -v -timeout=20m)
