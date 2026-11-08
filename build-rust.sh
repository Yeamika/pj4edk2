#!/bin/bash
# Rust-only stage, suitable for an existing compiler machine.
set -euo pipefail
repo=$(cd -- "$(dirname -- "$0")" && pwd -P)
output=$(realpath -m "${1:?usage: build-rust.sh OUTPUT_DIRECTORY}")
case "$output/" in "$repo/"*) echo 'Output must be outside the source tree' >&2; exit 2;; esac
test ! -e "$output" || { echo 'Choose a fresh Rust output directory' >&2; exit 2; }
export RUSTUP_TOOLCHAIN=1.93.0
export CARGO_HOME="${CARGO_HOME:-$HOME/.cargo}"
export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-$output/cargo-target}"
export PJB_ARCHIVE="$output/Rust.a"
cargo fetch --locked --manifest-path "$repo/reference/pocketjs-uefi-platform/Cargo.toml"
bash "$repo/reference/pocketjs-uefi-platform/tools/build-staticlib.sh"
quickjs=$(find "$CARGO_HOME/registry/src" -maxdepth 3 -type d -path '*/rquickjs-sys-0.12.2/quickjs' -print -quit)
test -n "$quickjs"
cp -R -- "$quickjs" "$output/quickjs"
python3 "$repo/tools/rust-bundle.py" create "$repo" "$output"
