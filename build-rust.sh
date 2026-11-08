#!/bin/bash
# Rust-only stage, suitable for an existing compiler machine.
set -euo pipefail
repo=$(cd -- "$(dirname -- "$0")" && pwd -P)
output= default_ui=${PJB_DEFAULT_UI:-$repo/default-ui}
source_args=()
while (($#)); do
  case "$1" in
    --output) output=${2:?}; shift 2;;
    --default-ui) default_ui=${2:?}; shift 2;;
    --source-lock|--source-cache|--source-repo|--source-ref) source_args+=("$1" "${2:?}"); shift 2;;
    --help|-h) echo 'Usage: build-rust.sh --output DIR [--default-ui DIR] [--source-cache DIR] [--source-lock FILE] [--source-repo NAME=URL] [--source-ref NAME=COMMIT|BRANCH|TAG|latest] (legacy: build-rust.sh DIR)'; exit 0;;
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
python3 "$repo/tools/upstream.py" prepare --output "$output" "${source_args[@]}"
build_repo=$(python3 -c 'import json,sys; print(json.load(open(sys.argv[1]))["source_tree"])' "$output/source-paths.json")
quickjs=$(python3 -c 'import json,sys; print(json.load(open(sys.argv[1]))["quickjs"])' "$output/source-paths.json")
# Cargo may update the private lock when an explicitly selected upstream changes
# its dependency graph. The source checkout and committed lock remain untouched.
cargo fetch --manifest-path "$build_repo/uefi-runtime/Cargo.toml"
bash "$build_repo/uefi-runtime/tools/build-staticlib.sh"
cp -- "$build_repo/uefi-runtime/Cargo.lock" "$output/Cargo.lock"
python3 - "$quickjs" "$output/quickjs" <<'PY'
import shutil, sys
shutil.copytree(sys.argv[1], sys.argv[2], ignore=shutil.ignore_patterns('.git'))
PY
python3 "$repo/tools/rust-bundle.py" create "$repo" "$output" --default-ui "$PJB_DEFAULT_UI" --source-lock "$output/upstream.lock.json"
