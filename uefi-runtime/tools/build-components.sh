#!/bin/bash
# Build the independent engine/page archive and its pinned UEFI ABI dependencies.
set -euo pipefail
PJB_HOST_ROOT=$(cd "$(dirname "$0")/.." && pwd)
: "${PJB_EDK2_ROOT:?set PJB_EDK2_ROOT to a pinned EDK II checkout}"
: "${PJB_EDK2_LIBC:?set PJB_EDK2_LIBC to a pinned edk2-libc checkout}"
: "${EDK_TOOLS_PATH:?set EDK_TOOLS_PATH to built EDK II BaseTools}"
: "${PJB_BUILD_DIR:?set PJB_BUILD_DIR to a private work directory}"
: "${PJB_ARCHIVE:?set the output archive path}"
PJB_FINAL_ARCHIVE=$PJB_ARCHIVE
if [[ -n ${PJB_RUST_ARCHIVE:-} ]]; then
  mkdir -p -- "$PJB_BUILD_DIR"
  cp -- "$PJB_RUST_ARCHIVE" "$PJB_BUILD_DIR/Rust.a"
else
  PJB_ARCHIVE="$PJB_BUILD_DIR/Rust.a" bash "$PJB_HOST_ROOT/tools/build-staticlib.sh"
fi
if [[ -z ${PJB_QUICKJS:-} ]]; then
  PJB_QUICKJS=$(find "${CARGO_HOME:?}/registry/src" -maxdepth 3 -type d -path '*/rquickjs-sys-0.12.2/quickjs' -print -quit)
fi
test -n "$PJB_QUICKJS"
mkdir -p "$PJB_BUILD_DIR/packages" "$PJB_BUILD_DIR/edk-work/Conf"
# This config is only for compiling the common ABI glue. Board config is compiled
# independently into the BSP SDK; this temporary EFI is never a delivered boot UI.
printf '{}\n' > "$PJB_BUILD_DIR/abi-config.json"
python3 "$PJB_HOST_ROOT/tools/prepare-edk2.py" --quickjs "$PJB_QUICKJS" \
  --archive "$PJB_BUILD_DIR/Rust.a" --edk2 "$PJB_EDK2_ROOT" --libc "$PJB_EDK2_LIBC" \
  --output "$PJB_BUILD_DIR/packages" --config "$PJB_BUILD_DIR/abi-config.json"
export WORKSPACE="$PJB_BUILD_DIR/edk-work" CONF_PATH="$PJB_BUILD_DIR/edk-work/Conf"
export PACKAGES_PATH="$PJB_EDK2_ROOT:$PJB_EDK2_LIBC:$PJB_BUILD_DIR/packages"
export GCC5_AARCH64_PREFIX=aarch64-linux-gnu- PYTHON_COMMAND=python3
cd "$WORKSPACE"
set +u
source "$PJB_EDK2_ROOT/edksetup.sh"
set -u
build -a AARCH64 -t GCC5 -b RELEASE -p PocketJsPkg/PocketJs.dsc -n "${PJB_JOBS:-4}"
python3 "$PJB_HOST_ROOT/tools/export-components.py" \
  --build "$WORKSPACE/Build/PocketJsBoot/RELEASE_GCC5/AARCH64" \
  --archive "$PJB_BUILD_DIR/Rust.a" --output "$PJB_FINAL_ARCHIVE" \
  --support "$PJB_BUILD_DIR/abi-support" --edk2 "$PJB_EDK2_ROOT" --libc "$PJB_EDK2_LIBC"
