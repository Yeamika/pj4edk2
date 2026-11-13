#!/usr/bin/env python3
"""Link the inherited pj4edk2 uefi runtime as a generic ARM64 EFI application."""
import argparse,json,os,shutil,subprocess
from pathlib import Path
LAB=Path(__file__).resolve().parents[1]
ROOT=LAB/'uefi/pj4edk2'
def main():
    p=argparse.ArgumentParser(description=__doc__)
    p.add_argument('--output',type=Path,required=True)
    p.add_argument('--edk2',type=Path,help='optional generic EDK II checkout; otherwise pinned upstream is downloaded')
    p.add_argument('--source-cache',type=Path)
    p.add_argument('--common-build',type=Path,help='reuse completed build.sh output from this exact runtime/UI source')
    p.add_argument('--jobs',type=int,default=4)
    a=p.parse_args();out=a.output.resolve()
    if out==LAB or LAB in out.parents:p.error('output must be outside the repository')
    out.mkdir(parents=True,exist_ok=True)
    common=a.common_build.resolve() if a.common_build else out/'common'
    if not a.common_build:
        command=['bash',str(ROOT/'build.sh'),'--output',str(common)]
        for flag,path in [('--edk2',a.edk2),('--source-cache',a.source_cache)]:
            if path:command.extend([flag,str(path.resolve())])
        subprocess.run(command,check=True)
    paths=json.loads((common/'source-paths.json').read_text())
    edk=Path(paths['edk2']);libc=Path(paths['edk2-libc']);quickjs=Path(paths['quickjs'])
    env=os.environ.copy();env['PJB_UPSTREAM_LOCK']=str(common/'upstream.lock.json')
    subprocess.run(['python3',str(ROOT/'uefi-runtime/tools/prepare-edk2.py'),
        '--quickjs',str(quickjs),'--archive',str(common/'Rust.a'),'--edk2',str(edk),'--libc',str(libc),
        '--output',str(out/'packages'),'--config',str(LAB/'uefi/config.json')],env=env,check=True)
    pkg=out/'packages/PocketJsPkg'
    # QEMU-only GOP setup, compiled from source with the menu. The upstream
    # fw_cfg implementation is copied unchanged from the selected EDK II tree.
    shutil.copy2(LAB/'uefi/QemuDisplay.c',pkg/'QemuDisplay.c')
    shutil.copy2(edk/'OvmfPkg/Library/QemuFwCfgLib/QemuFwCfgLibMmio.c',pkg/'QemuFwCfgLibMmio.c')
    inf=pkg/'PocketJs.inf';text=inf.read_text()
    text=text.replace('  Entry.c','  Entry.c\n  QemuDisplay.c\n  QemuFwCfgLibMmio.c')
    text=text.replace('  MdePkg/MdePkg.dec','  MdePkg/MdePkg.dec\n  MdeModulePkg/MdeModulePkg.dec\n  OvmfPkg/OvmfPkg.dec\n  EmbeddedPkg/EmbeddedPkg.dec')
    text=text.replace('  QuickJsCore\n','  QuickJsCore\n  FrameBufferBltLib\n  IoLib\n')
    text=text.replace('[Guids]','  gEfiGraphicsOutputProtocolGuid\n  gFdtClientProtocolGuid\n[Guids]')
    inf.write_text(text)
    entry=pkg/'Entry.c';text=entry.read_text()
    text=text.replace('extern int pjb_main(', 'EFI_STATUS Pj4edk2QemuDisplayInit(VOID);\nVOID Pj4edk2QemuDisplayRestore(VOID);\nextern int pjb_main(')
    text=text.replace('  Run.Result = EFI_ABORTED;', '  if (EFI_ERROR(Pj4edk2QemuDisplayInit())) {\n    gBS->FreePages(Stack, Pages); return EFI_UNSUPPORTED;\n  }\n  Run.Result = EFI_ABORTED;')
    text=text.replace('  gBS->FreePages(Stack, Pages);\n  return Run.Result;', '  Pj4edk2QemuDisplayRestore();\n  gBS->FreePages(Stack, Pages);\n  return Run.Result;')
    entry.write_text(text)
    dsc=pkg/'PocketJs.dsc';text=dsc.read_text()
    text=text.replace('[Components]','[LibraryClasses]\n  FrameBufferBltLib|MdeModulePkg/Library/FrameBufferBltLib/FrameBufferBltLib.inf\n[Components]')
    dsc.write_text(text)
    work=out/'workspace';(work/'Conf').mkdir(parents=True,exist_ok=True)
    env.update(WORKSPACE=str(work),CONF_PATH=str(work/'Conf'),PACKAGES_PATH=os.pathsep.join(map(str,[edk,libc,out/'packages'])),
               EDK_TOOLS_PATH=env.get('EDK_TOOLS_PATH',str(common/'basetools')),GCC5_AARCH64_PREFIX='aarch64-linux-gnu-',PYTHON_COMMAND='python3',PJ4EDK2_EDK2=str(edk),PJ4EDK2_JOBS=str(a.jobs))
    # The only shell text is constant; caller values travel through environment/argv.
    subprocess.run(['bash','-c','set -e; set --; source "$PJ4EDK2_EDK2/edksetup.sh"; build -a AARCH64 -t GCC5 -b RELEASE -p PocketJsPkg/PocketJs.dsc -n "$PJ4EDK2_JOBS"','build',str(a.jobs)],cwd=work,env=env,check=True)
    shutil.copy2(work/'Build/PocketJsBoot/RELEASE_GCC5/AARCH64/PocketJsBoot.efi',out/'PocketJsBoot.efi')
    print(out/'PocketJsBoot.efi')
if __name__=='__main__':main()
