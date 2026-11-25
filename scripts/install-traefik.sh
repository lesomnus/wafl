#!/usr/bin/env bash
# Pinned integration-test gateway. The runtime can use an existing Traefik v3.
set -euo pipefail
if [[ $(uname -s) != Linux || $(uname -m) != x86_64 ]]; then
  echo 'This helper installs Linux amd64 only; install Traefik v3 for your platform.' >&2
  exit 1
fi
destination=${1:?usage: bash scripts/install-traefik.sh DIRECTORY}
mkdir -p "$destination"
temporary=$(mktemp -d)
trap 'rm -rf "$temporary"' EXIT
curl --fail --location --retry 3 --output "$temporary/traefik.tar.gz" \
  https://github.com/traefik/traefik/releases/download/v3.8.0/traefik_v3.8.0_linux_amd64.tar.gz
printf '%s  %s\n' 266f4b02a3c9d16880c6d913053c6358df8163e296b6f0899ed4544d0a742bd6 \
  "$temporary/traefik.tar.gz" | sha256sum --check
tar -xzf "$temporary/traefik.tar.gz" -C "$temporary" traefik
install -m 755 "$temporary/traefik" "$destination/traefik"
