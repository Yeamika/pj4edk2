#!/bin/bash
# Produce the independent runtime archive and its matching UEFI ABI libraries.
set -euo pipefail
repo=$(cd -- "$(dirname -- "$0")" && pwd -P)
edk2="$repo/../edk2-ums9620/edk2"
output= rust_build= default_ui=${PJB_DEFAULT_UI:-$repo/default-ui}
source_args=()
while (($#)); do
  case "$1" in
    --output) output=${2:?}; shift 2;;
    --edk2) edk2=${2:?}; shift 2;;
    --rust-build) rust_build=${2:?}; shift 2;;
    --default-ui) default_ui=${2:?}; shift 2;;
    --source-lock|--source-cache|--source-repo|--source-ref) source_args+=("$1" "${2:?}"); shift 2;;
    --help|-h) echo 'Usage: build.sh --output DIR [--default-ui DIR] [--edk2 EDK2_CHECKOUT] [--rust-build DIR] [--source-cache DIR] [--source-lock FILE] [--source-repo NAME=URL] [--source-ref NAME=COMMIT|BRANCH|TAG|latest] (set EDK_TOOLS_PATH)'; exit 0;;
    *) echo "Unknown option: $1" >&2; exit 2;;
  esac
done
: "${output:?pass --output}"
PJB_DEFAULT_UI=$(realpath -e "$default_ui")
export PJB_DEFAULT_UI
python3 "$repo/tools/ui-input.py" "$PJB_DEFAULT_UI"
: "${EDK_TOOLS_PATH:?set EDK_TOOLS_PATH to compiled BaseTools}"
PJB_EDK2_ROOT=$(realpath -e "$edk2")
export PJB_EDK2_ROOT
PJB_BUILD_DIR=$(realpath -m "$output")
export PJB_BUILD_DIR
case "$PJB_BUILD_DIR/" in "$repo/"*) echo 'Output must be outside the source tree' >&2; exit 2;; esac
case "$PJB_BUILD_DIR/" in "$PJB_DEFAULT_UI/"*) echo 'Output must be outside the UI directory' >&2; exit 2;; esac
test ! -e "$PJB_BUILD_DIR/abi-support" || { echo 'Choose a fresh output directory' >&2; exit 2; }
export PJB_ARCHIVE="$PJB_BUILD_DIR/PocketJS.a"
CARGO_TARGET_DIR=$(realpath -m "${CARGO_TARGET_DIR:-$PJB_BUILD_DIR/cargo-target}")
export CARGO_TARGET_DIR
for input in "$repo" "$PJB_DEFAULT_UI"; do
  case "$CARGO_TARGET_DIR/" in "$input/"*) echo 'CARGO_TARGET_DIR must be outside the source and UI directories' >&2; exit 2;; esac
done
export CARGO_HOME="${CARGO_HOME:-$HOME/.cargo}"
export RUSTUP_TOOLCHAIN=1.93.0
python3 - "$repo" "$PJB_EDK2_ROOT" <<'PY'
import json, pathlib, subprocess, sys
repo = pathlib.Path(sys.argv[1])
lock = json.loads((repo/'sources.lock.json').read_text())
for name, path in [('edk2', pathlib.Path(sys.argv[2]))]:
    actual = subprocess.check_output(['git', '-C', str(path), 'rev-parse', 'HEAD'], text=True).strip()
    if actual != lock['dependencies'][name]['commit']:
        raise SystemExit(f'{name}: incompatible checkout {actual}')
PY
mkdir -p -- "$PJB_BUILD_DIR"
python3 "$repo/tools/upstream.py" prepare --output "$PJB_BUILD_DIR" "${source_args[@]}"
build_repo=$(python3 -c 'import json,sys; print(json.load(open(sys.argv[1]))["source_tree"])' "$PJB_BUILD_DIR/source-paths.json")
PJB_EDK2_LIBC=$(python3 -c 'import json,sys; print(json.load(open(sys.argv[1]))["edk2-libc"])' "$PJB_BUILD_DIR/source-paths.json")
PJB_QUICKJS=$(python3 -c 'import json,sys; print(json.load(open(sys.argv[1]))["quickjs"])' "$PJB_BUILD_DIR/source-paths.json")
export PJB_EDK2_LIBC PJB_QUICKJS
export PJB_UPSTREAM_LOCK="$PJB_BUILD_DIR/upstream.lock.json"
if [[ -n $rust_build ]]; then
  rust_build=$(realpath "$rust_build")
  python3 "$repo/tools/rust-bundle.py" verify "$repo" "$rust_build" --default-ui "$PJB_DEFAULT_UI" --source-lock "$PJB_UPSTREAM_LOCK" --quickjs "$PJB_QUICKJS"
  export PJB_RUST_ARCHIVE="$rust_build/Rust.a" PJB_QUICKJS="$rust_build/quickjs"
  cp -- "$rust_build/Cargo.lock" "$PJB_BUILD_DIR/Cargo.lock"
else
  cargo fetch --manifest-path "$build_repo/uefi-runtime/Cargo.toml"
  cp -- "$build_repo/uefi-runtime/Cargo.lock" "$PJB_BUILD_DIR/Cargo.lock"
fi
bash "$build_repo/uefi-runtime/tools/build-components.sh"
python3 - "$repo" "$PJB_BUILD_DIR" "$PJB_DEFAULT_UI" <<'PY'
import hashlib, json, pathlib, runpy, sys
repo, out, ui = map(pathlib.Path, sys.argv[1:])
api = runpy.run_path(str(repo/'tools/rust-bundle.py'))
record = dict(api['identity'](repo, ui, out/'upstream.lock.json'),
              archive_sha256=api['sha'](out/'PocketJS.a'),
              abi_manifest_sha256=api['sha'](out/'abi-support/manifest.json'),
              cargo_lock_sha256=api['sha'](out/'Cargo.lock'))
(out/'runtime-build.json').write_text(json.dumps(record, indent=2)+'\n')
PY
