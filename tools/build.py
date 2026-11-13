#!/usr/bin/env python3
"""Build the shared UI package and pinned upstream PocketJS WASM embedding."""
import argparse,hashlib,json,os,shutil,subprocess,sys,tomllib
from pathlib import Path
p=argparse.ArgumentParser(description=__doc__)
p.add_argument('--output',type=Path)
p.add_argument('--target-dir',type=Path)
p.add_argument('--ui',type=Path,help='external UI directory containing ui.json')
p.add_argument('--runtime-output',type=Path)
p.add_argument('--build-std',action='store_true',help='build the WASM standard library from installed rust-src')
a=p.parse_args();root=Path(__file__).resolve().parents[1]
source=root/'components/pj4edk2'
upstream=source/'vendor/pocketjs'
if not (upstream/'programs/web/wasm/src/lib.rs').is_file():
    raise SystemExit('Initialize components/pj4edk2 and its vendor/pocketjs submodule first')
env=os.environ.copy()
env.setdefault('RUSTUP_TOOLCHAIN',tomllib.loads((source/'rust-toolchain.toml').read_text())['toolchain']['channel'])
target=(a.target_dir or Path(env.get('CARGO_TARGET_DIR',root/'target'))).resolve()
command=[sys.executable,str(source/'tools/pack-ui.py'),'--output',str((a.output or root/'ui.pocket').resolve()),'--target-dir',str(target)]
if a.ui:command+=['--ui',str(a.ui.resolve())]
subprocess.run(command,env=env,check=True)
command=['cargo','build','--locked','--release','--target','wasm32-unknown-unknown',
         '--manifest-path',str(root/'wasm/Cargo.toml'),'--target-dir',str(target)]
if a.build_std:
    env['RUSTC_BOOTSTRAP']='1'
    command+=['-Z','build-std=std,panic_abort']
subprocess.run(command,env=env,check=True)
output=(a.runtime_output or root/'runtime').resolve()
output.mkdir(parents=True,exist_ok=True)
shutil.copy2(target/'wasm32-unknown-unknown/release/pj4edk2_web_runtime.wasm',output/'pocketjs.wasm')
for relative in ('platforms/web/wasm-ops.js','platforms/web/generated-ui.js',
                 'engine/guest-web/src/generated-ui.js','engine/guest-web/src/ui-descriptors.js','LICENSE'):
    destination=output/relative
    destination.parent.mkdir(parents=True,exist_ok=True)
    shutil.copy2(upstream/relative,destination)
commit=subprocess.check_output(['git','-C',str(upstream),'rev-parse','HEAD'],text=True).strip()
record={'backend':'pocketjs-wasm-software','repository':'https://github.com/pocket-nexus/pocketjs',
        'commit':commit,'presentation':'pj4edk2-rendering:nearest-textures+pixel-snap',
        'wasm_sha256':hashlib.sha256((output/'pocketjs.wasm').read_bytes()).hexdigest()}
(output/'manifest.json').write_text(json.dumps(record,indent=2)+'\n')
print(output)
