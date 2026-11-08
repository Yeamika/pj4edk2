#!/bin/bash
set -euo pipefail
pjb_root=$(cd -- "$(dirname -- "$0")/.." && pwd -P)
pj_target=${CARGO_TARGET_DIR:?set CARGO_TARGET_DIR outside sources}
pj_output=${PJB_ARCHIVE:?set PJB_ARCHIVE}
export CARGO_TARGET_DIR="$pj_target"
unset POCKETJS_QUICKJS_INCLUDE
export RUSTFLAGS="-Z fixed-x18 -C panic=abort"
export RUSTC_BOOTSTRAP=1
pjb_features=()
if test -n "${PJB_CARGO_FEATURES:-}"; then pjb_features=(--features "$PJB_CARGO_FEATURES"); fi
cargo build --manifest-path "$pjb_root/Cargo.toml" --locked -Z build-std=core,alloc --release --target aarch64-unknown-none "${pjb_features[@]}"
mkdir -p -- "$(dirname -- "$pj_output")"
cp -- "$CARGO_TARGET_DIR/aarch64-unknown-none/release/libpocketjs_uefi_runtime.a" "$pj_output"
