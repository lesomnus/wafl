#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
# This creates a dedicated disposable kind cluster. It never uses the caller's
# current Kubernetes context. Build hosts may use a remote Docker daemon.
cluster=${WAFL_TEST_CLUSTER:-wafl-test}
image=${WAFL_TEST_IMAGE:-wafl:kubernetes-test}
gateway=${WAFL_TEST_GATEWAY_IMAGE:-wafl-traefik:kubernetes-test}
baseline=${WAFL_TEST_BASELINE_IMAGE:-wafl:kubernetes-baseline-2e463666}
export WAFL_TEST_BASELINE_IMAGE="$baseline"
export WAFL_TEST_CLUSTER="$cluster"
if [ -z "${WAFL_TEST_KUBECONFIG:-}" ]; then
  bash scripts/build.sh
  cargo build --locked --target wasm32-wasip2 -p wafl-test-guests
  target=$(cargo metadata --format-version 1 --no-deps | python3 -c 'import json,sys; print(json.load(sys.stdin)["target_directory"])')
  export WAFL_TEST_GUEST_DIR="$target/wasm32-wasip2/debug"
  export WAFL_TEST_CLI="$target/debug/wafl"
  # Different executable bytes, not just different manifests or runtime env.
  for version in a b; do
    WAFL_TEST_BUILD_VERSION="$version" cargo build --locked --target wasm32-wasip2 -p wafl-test-guests --bin grpc
    cp "$WAFL_TEST_GUEST_DIR/grpc.wasm" "$WAFL_TEST_GUEST_DIR/grpc-$version.wasm"
  done
  stage=$(mktemp -d)
  trap 'rm -rf "$stage"' EXIT
  cp "$target/debug/wafl" "$target/debug/wafl-kube" "$stage/"
  # Test the locally compiled binaries. Production uses the multi-stage
  # Dockerfile, whose Debian build/runtime versions match each other.
  cp kubernetes/tests/Dockerfile "$stage/Dockerfile"
  docker build -t "$image" "$stage"
  docker build -f Dockerfile.traefik -t "$gateway" .
  "$target/debug/wafl-kube" bootstrap dev --name "$cluster" --image "$image" --gateway-image "$gateway" --load --port "${WAFL_TEST_PORT:-18080}" --control-port "${WAFL_TEST_CONTROL_PORT:-18082}"
  # A custom WAFL_TEST_BASELINE_IMAGE must already exist in the Docker daemon.
  if [ "$baseline" = wafl:kubernetes-baseline-2e463666 ]; then
    # Published predecessor 2e4636662980411a9d229bff32f5b8ed9d3cffdb.
    reference=ghcr.io/lesomnus/wafl@sha256:0b357c3ddeb927b5e3d587d6d5bd1a4d5f97e373ff71166f17f3fcdf2a4bedd2
    docker pull "$reference"
    docker tag "$reference" "$baseline"
  fi
  kind load docker-image --name "$cluster" "$baseline"
  export WAFL_TEST_KUBECONFIG="${XDG_CACHE_HOME:-$HOME/.cache}/wafl/dev/$cluster/kubeconfig"
fi
# The operator test harness forwards ports; the tested WAFL client has no
# kubeconfig or Kubernetes helper executables.
(cd kubernetes && go test -race ./tests -count=1 -v -timeout=20m)
