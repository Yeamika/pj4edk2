#!/usr/bin/env python3
"""Split compiled runtime/pages from UEFI ABI libraries; no board UI snapshots."""
import argparse
import hashlib
import json
from pathlib import Path
import shutil
import subprocess
import tempfile

p = argparse.ArgumentParser()
for name in ('build', 'archive', 'output', 'support', 'edk2', 'libc'):
    p.add_argument('--' + name, type=Path, required=True)
a = p.parse_args()
lists = list(a.build.rglob('PocketJs/OUTPUT/static_library_files.lst'))
if len(lists) != 1:
    raise SystemExit('Expected exactly one compiled PocketJs module')
module = lists[0].parent
libs = [Path(line) for line in lists[0].read_text().splitlines() if line.strip()]
quickjs = next(f for f in libs if f.name == 'QuickJsCore.lib')
runtime_objects = [module / (n + '.obj') for n in ('Entry', 'TimeCompat', 'Chainload', 'Hardware', 'Settings', 'Discovery', 'Fastboot', 'Display')]
a.output.parent.mkdir(parents=True, exist_ok=True)
if a.support.exists():
    raise SystemExit('Retain existing ABI support output; choose a fresh build')
a.support.mkdir(parents=True)
with tempfile.TemporaryDirectory(dir=a.output.parent) as tmp:
    tmp = Path(tmp)
    # MRI archive mode preserves all members, including duplicate archive names.
    # Use local simple names to avoid its whitespace/path parsing ambiguities.
    shutil.copy2(a.archive, tmp / 'rust.a')
    shutil.copy2(quickjs, tmp / 'guest.a')
    for index, file in enumerate(runtime_objects):
        shutil.copy2(file, tmp / f'host{index}.o')
    commands = ['CREATE complete.a', 'ADDLIB rust.a', 'ADDLIB guest.a']
    commands += [f'ADDMOD host{i}.o' for i in range(len(runtime_objects))]
    subprocess.run(['aarch64-linux-gnu-ar', '-M'], input='\n'.join(commands + ['SAVE', 'END', '']),
                   text=True, cwd=tmp, check=True)
    subprocess.run(['aarch64-linux-gnu-objcopy', '--remove-section=.eh_frame',
                    '--remove-section=.eh_frame_hdr', str(tmp / 'complete.a')], check=True)
    shutil.copy2(tmp / 'complete.a', a.output)
dependencies = []
for file in libs:
    if file.name in ('PocketJsCore.lib', 'QuickJsCore.lib', 'PocketJsBoot.lib'):
        continue
    name = f'{len(dependencies):03d}-{file.name}'
    shutil.copy2(file, a.support / name)
    dependencies.append(name)
shutil.copy2(module / 'AutoGen.obj', a.support / 'AutoGen.o')
dependencies.append('AutoGen.o')
shutil.copy2(a.edk2 / 'BaseTools/Scripts/GccBase.lds', a.support / 'GccBase.lds')
def commit(path):
    return subprocess.check_output(['git', '-C', str(path), 'rev-parse', 'HEAD'], text=True).strip()
manifest = {'schema': 1, 'dependencies': dependencies, 'entry': '_ModuleEntryPoint',
            'edk2': commit(a.edk2), 'libc': commit(a.libc),
            'runtime_sha256': hashlib.sha256(a.output.read_bytes()).hexdigest(),
            'files': {f.name: hashlib.sha256(f.read_bytes()).hexdigest() for f in a.support.iterdir()}}
(a.support / 'manifest.json').write_text(json.dumps(manifest, indent=2) + '\n')
print(a.output)
