#!/bin/bash
set -euo pipefail
repo=$(cd -- "$(dirname -- "$0")" && pwd -P)
: "${CARGO_TARGET_DIR:?set CARGO_TARGET_DIR outside the source tree}"
export RUSTUP_TOOLCHAIN=1.93.0
export CARGO_HOME="${CARGO_HOME:-$HOME/.cargo}"
cargo fetch --locked --manifest-path "$repo/uefi-runtime/Cargo.toml"
PYTHONDONTWRITEBYTECODE=1 python3 -m unittest discover -s "$repo/tools/tests" -v
export POCKETJS_QUICKJS_INCLUDE=$(find "$CARGO_HOME/registry/src" -maxdepth 3 -type d -path '*/rquickjs-sys-0.12.2/quickjs' -print -quit)
test -n "$POCKETJS_QUICKJS_INCLUDE"
cargo test --locked --manifest-path "$repo/uefi-runtime/Cargo.toml"
cargo test --locked --workspace --manifest-path "$repo/pocketjsforedk2/Cargo.toml"
cargo test --locked --manifest-path "$repo/default-ui/Cargo.toml"
if [[ ${1:-} != --rust-only ]]; then
  : "${PJB_EDK2_ROOT:?set PJB_EDK2_ROOT for native UEFI C tests, or pass --rust-only}"
  PYTHONDONTWRITEBYTECODE=1 python3 -m unittest discover \
    -s "$repo/uefi-runtime/tests" -p 'test_*.py' -v
fi
