#!/bin/bash
# Produce the independent runtime archive and its matching UEFI ABI libraries.
set -euo pipefail
repo=$(cd -- "$(dirname -- "$0")" && pwd -P)
edk2="$repo/../edk2-ums9620/edk2"
output= rust_build=
while (($#)); do
  case "$1" in
    --output) output=${2:?}; shift 2;;
    --edk2) edk2=${2:?}; shift 2;;
    --rust-build) rust_build=${2:?}; shift 2;;
    --help|-h) echo 'Usage: build.sh --output DIR [--edk2 EDK2_CHECKOUT] [--rust-build DIR] (set EDK_TOOLS_PATH)'; exit 0;;
    *) echo "Unknown option: $1" >&2; exit 2;;
  esac
done
: "${output:?pass --output}" "${EDK_TOOLS_PATH:?set EDK_TOOLS_PATH to compiled BaseTools}"
export PJB_EDK2_ROOT=$(realpath "$edk2")
export PJB_EDK2_LIBC="$repo/edk2-libc"
export PJB_BUILD_DIR=$(realpath -m "$output")
case "$PJB_BUILD_DIR/" in "$repo/"*) echo 'Output must be outside the source tree' >&2; exit 2;; esac
test ! -e "$PJB_BUILD_DIR/abi-support" || { echo 'Choose a fresh output directory' >&2; exit 2; }
export PJB_ARCHIVE="$PJB_BUILD_DIR/PocketJS.a"
export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-$PJB_BUILD_DIR/cargo-target}"
export CARGO_HOME="${CARGO_HOME:-$HOME/.cargo}"
export RUSTUP_TOOLCHAIN=1.93.0
python3 - "$repo" "$PJB_EDK2_ROOT" <<'PY'
import json, pathlib, subprocess, sys
repo = pathlib.Path(sys.argv[1])
lock = json.loads((repo/'sources.lock.json').read_text())
for name, path in [('edk2', pathlib.Path(sys.argv[2])), ('edk2-libc', repo/'edk2-libc')]:
    actual = subprocess.check_output(['git', '-C', str(path), 'rev-parse', 'HEAD'], text=True).strip()
    if actual != lock['dependencies'][name]['commit']:
        raise SystemExit(f'{name}: incompatible checkout {actual}')
PY
mkdir -p -- "$PJB_BUILD_DIR"
if [[ -n $rust_build ]]; then
  rust_build=$(realpath "$rust_build")
  python3 "$repo/tools/rust-bundle.py" verify "$repo" "$rust_build"
  export PJB_RUST_ARCHIVE="$rust_build/Rust.a" PJB_QUICKJS="$rust_build/quickjs"
else
  cargo fetch --locked --manifest-path "$repo/reference/pocketjs-uefi-platform/Cargo.toml"
fi
bash "$repo/reference/pocketjs-uefi-platform/tools/build-components.sh"
python3 - "$repo" "$PJB_BUILD_DIR" <<'PY'
import hashlib, json, pathlib, runpy, sys
repo, out = map(pathlib.Path, sys.argv[1:])
api = runpy.run_path(str(repo/'tools/rust-bundle.py'))
record = {'schema': 1, 'sources': api['sources'](repo),
          'archive_sha256': api['sha'](out/'PocketJS.a'),
          'abi_manifest_sha256': api['sha'](out/'abi-support/manifest.json')}
(out/'runtime-build.json').write_text(json.dumps(record, indent=2)+'\n')
PY
