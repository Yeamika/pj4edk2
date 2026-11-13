#!/usr/bin/env python3
"""Build the small kernel, initramfs/module and optional pj4edk2 EFI application."""
import argparse,subprocess,sys
from pathlib import Path
ROOT=Path(__file__).resolve().parent
p=argparse.ArgumentParser(description=__doc__)
p.add_argument('--output',type=Path,required=True)
p.add_argument('--kernel-only',action='store_true')
p.add_argument('--jobs',type=int,default=4)
p.add_argument('--edk2',type=Path)
p.add_argument('--source-cache',type=Path)
a=p.parse_args();out=a.output.resolve()
if out==ROOT or ROOT in out.parents:p.error('output must be outside the source tree')
subprocess.run([sys.executable,str(ROOT/'kernel/build.py'),'--output',str(out/'linux'),'--jobs',str(a.jobs)],check=True)
if not a.kernel_only:
    command=[sys.executable,str(ROOT/'uefi/build.py'),'--output',str(out/'uefi'),'--jobs',str(a.jobs)]
    for flag,path in [('--edk2',a.edk2),('--source-cache',a.source_cache)]:
        if path:command.extend([flag,str(path.resolve())])
    subprocess.run(command,check=True)

    subprocess.run([sys.executable,str(ROOT/'efi/build.py'),'--output',str(out/'efi'),
                    '--common-build',str(out/'uefi/common'),'--jobs',str(a.jobs)],check=True)
