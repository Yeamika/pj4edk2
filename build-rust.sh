#!/bin/bash
# Rust-only stage, suitable for an existing compiler machine.
set -euo pipefail
repo=$(cd -- "$(dirname -- "$0")" && pwd -P)
output= default_ui=${PJB_DEFAULT_UI:-$repo/default-ui}
while (($#)); do
  case "$1" in
    --output) output=${2:?}; shift 2;;
    --default-ui) default_ui=${2:?}; shift 2;;
    --help|-h) echo 'Usage: build-rust.sh --output DIR [--default-ui DIR] (legacy: build-rust.sh DIR)'; exit 0;;
    -*) echo "Unknown option: $1" >&2; exit 2;;
    *) test -z "$output" || { echo 'Output specified more than once' >&2; exit 2; }; output=$1; shift;;
  esac
done
output=$(realpath -m "${output:?pass --output}")
PJB_DEFAULT_UI=$(realpath -e "$default_ui")
export PJB_DEFAULT_UI
python3 "$repo/tools/ui-input.py" "$PJB_DEFAULT_UI"
case "$output/" in "$repo/"*) echo 'Output must be outside the source tree' >&2; exit 2;; esac
case "$output/" in "$PJB_DEFAULT_UI/"*) echo 'Output must be outside the UI directory' >&2; exit 2;; esac
test ! -e "$output" || { echo 'Choose a fresh Rust output directory' >&2; exit 2; }
export RUSTUP_TOOLCHAIN=1.93.0
export CARGO_HOME="${CARGO_HOME:-$HOME/.cargo}"
CARGO_TARGET_DIR=$(realpath -m "${CARGO_TARGET_DIR:-$output/cargo-target}")
export CARGO_TARGET_DIR
for input in "$repo" "$PJB_DEFAULT_UI"; do
  case "$CARGO_TARGET_DIR/" in "$input/"*) echo 'CARGO_TARGET_DIR must be outside the source and UI directories' >&2; exit 2;; esac
done
export PJB_ARCHIVE="$output/Rust.a"
cargo fetch --locked --manifest-path "$repo/reference/pocketjs-uefi-platform/Cargo.toml"
bash "$repo/reference/pocketjs-uefi-platform/tools/build-staticlib.sh"
quickjs=$(find "$CARGO_HOME/registry/src" -maxdepth 3 -type d -path '*/rquickjs-sys-0.12.2/quickjs' -print -quit)
test -n "$quickjs"
cp -R -- "$quickjs" "$output/quickjs"
python3 "$repo/tools/rust-bundle.py" create "$repo" "$output" --default-ui "$PJB_DEFAULT_UI"
