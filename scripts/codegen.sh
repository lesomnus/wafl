#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
revision=684502f74a2176ef79a4feeb5d6a9d13c78b2021
tools_dir=${WAFL_TOOLS_DIR:-"$PWD/target/codegen"}
mkdir -p "$tools_dir"
tools_dir=$(cd "$tools_dir" && pwd)
if [[ ! -d "$tools_dir/paydirt/.git" ]]; then
  git clone --filter=blob:none https://github.com/lesomnus/paydirt "$tools_dir/paydirt"
fi
if [[ ! -f "$tools_dir/revision" ]] || [[ $(cat "$tools_dir/revision") != "$revision" ]]; then
  git -C "$tools_dir/paydirt" fetch origin "$revision"
  git -C "$tools_dir/paydirt" checkout --detach "$revision"
  cargo install --locked --root "$tools_dir" --path "$tools_dir/paydirt/cli"
  bash "$tools_dir/paydirt/scripts/install-codegen-linux.sh" "$tools_dir/bin"
  printf '%s\n' "$revision" > "$tools_dir/revision"
fi
export PATH="$tools_dir/bin:$PATH"
paydirt gen "$@"
(cd scripts/crd && go run . "$@")
