#!/usr/bin/env python3
"""Build generic EDK II BaseTools in a private directory, leaving source untouched."""
import argparse
import os
from pathlib import Path
import runpy
import shutil
import subprocess

ROOT = Path(__file__).resolve().parents[1]


def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('--edk2', type=Path, required=True, help='generic EDK II checkout')
    p.add_argument('--output', type=Path, required=True, help='fresh directory outside the source trees')
    p.add_argument('--jobs', type=int, default=int(os.environ.get('PJB_JOBS', '4')))
    a = p.parse_args()
    edk2, output = a.edk2.resolve(), a.output.resolve()
    api = runpy.run_path(str(ROOT/'tools/upstream.py'))
    api['outside_sources'](ROOT, output)
    if output == edk2 or edk2 in output.parents:
        p.error('--output must be outside the EDK II source tree')
    if output.exists():
        p.error('output exists; choose a fresh BaseTools directory')
    if a.jobs < 1:
        p.error('--jobs must be positive')
    api['inspect_edk2'](edk2)
    brotli = api['BROTLI']
    if (api['git'](edk2, 'ls-tree', 'HEAD', '--', brotli).startswith('160000 ')
            and not (edk2/brotli/'c/include/brotli/encode.h').is_file()):
        p.error(f'initialize the EDK II submodule first: git -C {edk2} submodule update --init -- {brotli}')
    shutil.copytree(edk2/'BaseTools', output,
                    ignore=shutil.ignore_patterns('.git', '__pycache__', '*.pyc'))
    subprocess.run(['make', '-C', str(output), '-j', str(a.jobs)], check=True)
    print(f'EDK_TOOLS_PATH={output}')


if __name__ == '__main__':
    main()
