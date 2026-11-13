#!/usr/bin/env python3
"""Build the Linux display handoff EFI app, independently of Rust and QuickJS."""
import argparse,json,os,shutil,subprocess
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
def main():
    p=argparse.ArgumentParser(description=__doc__)
    p.add_argument('--output',type=Path,required=True)
    p.add_argument('--edk2',type=Path,help='generic initialized EDK II source; required without --common-build')
    p.add_argument('--basetools',type=Path,help='optional prebuilt BaseTools')
    p.add_argument('--common-build',type=Path,help='reuse EDK II paths/BaseTools from the UEFI menu build')
    p.add_argument('--pj4edk2',type=Path,default=ROOT/'uefi/pj4edk2',help='source of the public BootDisplay ABI header')
    p.add_argument('--jobs',type=int,default=4)
    a=p.parse_args();out=a.output.resolve()
    if out==ROOT or ROOT in out.parents:p.error('output must be outside the source repository')
    if a.jobs<1:p.error('jobs must be positive')
    if a.common_build:
        common=a.common_build.resolve();edk=a.edk2 or Path(json.loads((common/'source-paths.json').read_text())['edk2'])
        tools=a.basetools or common/'basetools'
    else:
        if not a.edk2:p.error('supply --edk2 or --common-build')
        edk=a.edk2;tools=a.basetools or out/'basetools'
    edk=edk.resolve();tools=tools.resolve();out.mkdir(parents=True,exist_ok=True)
    if not tools.exists():
        subprocess.run(['python3',str(a.pj4edk2/'tools/build-basetools.py'),'--edk2',str(edk),'--output',str(tools),'--jobs',str(a.jobs)],check=True)
    pkg=out/'packages/DisplayBridgePkg';pkg.mkdir(parents=True,exist_ok=True)
    shutil.copy2(ROOT/'efi/DisplayBridge.inf',pkg/'DisplayBridge.inf')
    shutil.copy2(ROOT/'efi/common/edk2/DisplayBridge.c',pkg/'DisplayBridge.c')
    for header in (ROOT/'efi/common/include').glob('*.h'):shutil.copy2(header,pkg/header.name)
    shutil.copy2(a.pj4edk2/'pocketjsforedk2/include/pocketjs_boot_context.h',pkg/'pocketjs_boot_context.h')
    shutil.copytree(ROOT/'efi/baremetal',pkg/'baremetal',dirs_exist_ok=True)
    intrinsics='MdePkg/Library/CompilerIntrinsicsLib/CompilerIntrinsicsLib.inf'
    if not (edk/intrinsics).exists():intrinsics='ArmPkg/Library/CompilerIntrinsicsLib/CompilerIntrinsicsLib.inf'
    stack='MdePkg/Library/StackCheckLib/StackCheckLib.inf'
    if not (edk/stack).exists():stack='MdePkg/Library/BaseStackCheckLib/BaseStackCheckLib.inf'
    (pkg/'DisplayBridge.dsc').write_text('''[Defines]
  PLATFORM_NAME = DisplayBridge
  PLATFORM_GUID = 39b2574c-55e2-4956-a252-9135835bacb6
  PLATFORM_VERSION = 0.1
  DSC_SPECIFICATION = 0x00010006
  OUTPUT_DIRECTORY = Build/DisplayBridge
  SUPPORTED_ARCHITECTURES = AARCH64
  BUILD_TARGETS = RELEASE
  SKUID_IDENTIFIER = DEFAULT
[LibraryClasses]
!include MdePkg/MdeLibs.dsc.inc
  BaseLib|MdePkg/Library/BaseLib/BaseLib.inf
  BaseMemoryLib|MdePkg/Library/BaseMemoryLib/BaseMemoryLib.inf
  DebugLib|MdePkg/Library/BaseDebugLibNull/BaseDebugLibNull.inf
  MemoryAllocationLib|MdePkg/Library/UefiMemoryAllocationLib/UefiMemoryAllocationLib.inf
  UefiBootServicesTableLib|MdePkg/Library/UefiBootServicesTableLib/UefiBootServicesTableLib.inf
  UefiRuntimeServicesTableLib|MdePkg/Library/UefiRuntimeServicesTableLib/UefiRuntimeServicesTableLib.inf
  UefiApplicationEntryPoint|MdePkg/Library/UefiApplicationEntryPoint/UefiApplicationEntryPoint.inf
  UefiLib|MdePkg/Library/UefiLib/UefiLib.inf
  DevicePathLib|MdePkg/Library/UefiDevicePathLib/UefiDevicePathLib.inf
  PrintLib|MdePkg/Library/BasePrintLib/BasePrintLib.inf
  PcdLib|MdePkg/Library/BasePcdLibNull/BasePcdLibNull.inf
  FdtLib|EmbeddedPkg/Library/FdtLib/FdtLib.inf
  CompilerIntrinsicsLib|'''+intrinsics+'\n  NULL|'+stack+'''
[BuildOptions]
  GCC:*_*_AARCH64_CC_FLAGS = -fno-lto -ffixed-x18
  GCC:*_*_AARCH64_DLINK_FLAGS == -Wl,--emit-relocs -nostdlib -Wl,--gc-sections -u _ModuleEntryPoint -Wl,-e,_ModuleEntryPoint -Wl,-Map,$(DEST_DIR_DEBUG)/$(BASE_NAME).map -z common-page-size=0x1000 -fno-lto -Wl,--no-undefined
  GCC:*_*_AARCH64_CC_XIPFLAGS == -mstrict-align
[Components]
  DisplayBridgePkg/DisplayBridge.inf
  DisplayBridgePkg/baremetal/Baremetal.inf
''')
    work=out/'workspace';(work/'Conf').mkdir(parents=True,exist_ok=True)
    env=os.environ.copy();env.update(WORKSPACE=str(work),CONF_PATH=str(work/'Conf'),
        PACKAGES_PATH=os.pathsep.join(map(str,[edk,out/'packages'])),EDK_TOOLS_PATH=str(tools),
        GCC5_AARCH64_PREFIX='aarch64-linux-gnu-',PYTHON_COMMAND='python3',PJ4EDK2_EDK2=str(edk),PJ4EDK2_JOBS=str(a.jobs))
    subprocess.run(['bash','-c','set -e; set --; source "$PJ4EDK2_EDK2/edksetup.sh"; build -a AARCH64 -t GCC5 -b RELEASE -p DisplayBridgePkg/DisplayBridge.dsc -n "$PJ4EDK2_JOBS"'],cwd=work,env=env,check=True)
    shutil.copy2(work/'Build/DisplayBridge/RELEASE_GCC5/AARCH64/DisplayBridge.efi',out/'DisplayBridge.efi')
    shutil.copy2(work/'Build/DisplayBridge/RELEASE_GCC5/AARCH64/Baremetal.efi',out/'Baremetal.efi')
    print(out/'DisplayBridge.efi')
if __name__=='__main__':main()
