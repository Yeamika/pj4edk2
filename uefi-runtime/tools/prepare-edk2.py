#!/usr/bin/env python3
"""Prepare an EDK II app with the official guest and edk2-libc, no Linux libc."""
import argparse, hashlib, json, os, shutil, subprocess
from pathlib import Path

p=argparse.ArgumentParser()
for name in ('quickjs','archive','edk2','libc','output','config'):
 p.add_argument('--'+name,type=Path,required=True)
a=p.parse_args();root=Path(__file__).resolve().parents[1];edk=a.edk2.resolve();libc=a.libc.resolve()
pkg=a.output.resolve()/'PocketJsPkg';pkg.mkdir(parents=True,exist_ok=True)
upstream=root.parent/'vendor/pocketjs'
for source,target in [(a.quickjs,pkg/'QuickJsCore/quickjs'),(upstream/'engine/guest-quickjs',pkg/'QuickJsCore/guest'),(upstream/'contracts/generated',pkg/'QuickJsCore/contracts')]:
 shutil.copytree(source,target,dirs_exist_ok=True,ignore=shutil.ignore_patterns('.git'))
# The official guest remains unchanged. QuickJS's own single-threaded branch
# is also appropriate to this UEFI owner; select it with an explicit port macro.
cutils=pkg/'QuickJsCore/quickjs/cutils.h'
text=cutils.read_text()
text=text.replace('!defined(__wasi__) && !defined(__DJGPP)', '!defined(__wasi__) && !defined(__DJGPP) && !defined(PJB_UEFI)')
text=text.replace('defined(EMSCRIPTEN) || defined(__wasi__) || defined(__DJGPP)', 'defined(EMSCRIPTEN) || defined(__wasi__) || defined(__DJGPP) || defined(PJB_UEFI)')
cutils.write_text(text)
(pkg/'RustCore').mkdir(exist_ok=True);shutil.copy2(a.archive,pkg/'RustCore/PocketJsCore.lib')
# Rust's precompiled core includes unwind metadata even with panic=abort.
# No unwinding crosses this firmware boundary; GenFw cannot preserve its
# PC-relative references when converting ELF sections to PE/COFF.
subprocess.run(['aarch64-linux-gnu-objcopy','--remove-section=.eh_frame',
                '--remove-section=.eh_frame_hdr',str(pkg/'RustCore/PocketJsCore.lib')],check=True)
(pkg/'PocketJsPkg.dec').write_text('''[Defines]
  DEC_SPECIFICATION = 0x00010005
  PACKAGE_NAME = PocketJsPkg
  PACKAGE_GUID = 488df163-3e6d-4860-94a6-25d179724b17
  PACKAGE_VERSION = 0.1
[Includes]
  QuickJsCore/quickjs
  QuickJsCore/guest/include
  QuickJsCore/guest/src
  QuickJsCore/contracts
[LibraryClasses]
  PocketJsCore|Include/PocketJsCore.h
  QuickJsCore|Include/QuickJsCore.h
''')
(pkg/'Include').mkdir(exist_ok=True)
for name in ['PocketJsCore','QuickJsCore']:(pkg/f'Include/{name}.h').write_text('/* Link-only EDK II library class. */\n')
(pkg/'RustCore/RustCore.inf').write_text('''[Defines]
  INF_VERSION = 0x00010006
  BASE_NAME = PocketJsCore
  FILE_GUID = eec3c175-03ca-48ac-b4ec-5c17795d9aa6
  MODULE_TYPE = BASE
  VERSION_STRING = 0.1
  LIBRARY_CLASS = PocketJsCore
[Binaries.AARCH64]
  LIB|PocketJsCore.lib|*
''')
(pkg/'QuickJsCore/QuickJsCore.inf').write_text('''[Defines]
  INF_VERSION = 0x00010006
  BASE_NAME = QuickJsCore
  FILE_GUID = 51e2259d-a117-49cb-9c46-b5f7fa47224a
  MODULE_TYPE = BASE
  VERSION_STRING = 0.1
  LIBRARY_CLASS = QuickJsCore
[Sources]
  quickjs/quickjs.c
  quickjs/libregexp.c
  quickjs/libunicode.c
  quickjs/dtoa.c
  guest/src/guest.c
[Packages]
  MdePkg/MdePkg.dec
  StdLib/StdLib.dec
  PocketJsPkg/PocketJsPkg.dec
[LibraryClasses]
  LibC
  LibMath
[BuildOptions]
  GCC:*_*_AARCH64_CC_FLAGS = -std=gnu11 -Wno-error -Werror=implicit-function-declaration -Werror=implicit-int -DNDEBUG -DPJB_UEFI -D__STDC_NO_ATOMICS__=1 -U__linux__ -U__unix__ -D__NEWLIB__ -include PjbCCompat.h
''')
(pkg/'QuickJsCore/PjbCCompat.h').write_text('''#include <math.h>
#include <sys/time.h>
/* edk2-libc hides these C99 prototypes behind its feature-test macros. */
double round(double);
double trunc(double);
double hypot(double, double);
double acosh(double);
double asinh(double);
double atanh(double);
double log1p(double);
double cbrt(double);
long lrint(double);
int clock_gettime(clockid_t, struct timespec *);
#define alloca __builtin_alloca
#ifndef isfinite
#define isfinite(x) __builtin_isfinite(x)
#endif
#ifndef signbit
#define signbit(x) __builtin_signbit(x)
#endif
''')
configuration=json.loads(a.config.read_text())
config_bytes=json.dumps(configuration,ensure_ascii=True,separators=(',',':')).encode()
if len(config_bytes)>65536: raise ValueError('platform configuration too large')
(pkg/'PlatformConfig.c').write_text('#include <stddef.h>\nstatic const unsigned char Config[] = {'+','.join(map(str,config_bytes))+'};\nconst unsigned char *pjb_platform_config(size_t *length) { *length = sizeof(Config); return Config; }\n')
shutil.copy2(root.parent/'pocketjsforedk2/include/pocketjs_boot_context.h',pkg/'pocketjs_boot_context.h')
shutil.copy2(root.parent/'pocketjsforedk2/include/pj4edk2_fastboot.h',pkg/'pj4edk2_fastboot.h')
shutil.copy2(root.parent/'pocketjsforedk2/include/pj4edk2_display.h',pkg/'pj4edk2_display.h')
for name in ['Entry.c','TimeCompat.c','Chainload.c','Hardware.c','Settings.c','Discovery.c','Esp.h','Fastboot.c','Display.c']:
 shutil.copy2(root/'edk2'/name,pkg/name)
(pkg/'PocketJs.inf').write_text('''[Defines]
  INF_VERSION = 0x00010006
  BASE_NAME = PocketJsBoot
  FILE_GUID = edb979d9-310e-4145-950e-ac1691a1010a
  MODULE_TYPE = UEFI_APPLICATION
  VERSION_STRING = 0.1
  ENTRY_POINT = PjbUefiMain
[Sources]
  Entry.c
  PlatformConfig.c
  TimeCompat.c
  Chainload.c
  Hardware.c
  Settings.c
  Discovery.c
  Fastboot.c
  Display.c
[Packages]
  MdePkg/MdePkg.dec
  ShellPkg/ShellPkg.dec
  StdLib/StdLib.dec
  PocketJsPkg/PocketJsPkg.dec
[LibraryClasses]
  LibC
  LibMath
  ShellCEntryLib
  UefiApplicationEntryPoint
  UefiBootServicesTableLib
  BaseLib
  BaseMemoryLib
  DevicePathLib
  MemoryAllocationLib
  UefiLib
  PocketJsCore
  QuickJsCore
[Protocols]
  gEfiGraphicsOutputProtocolGuid
  gEfiLoadedImageProtocolGuid
  gEfiSimpleFileSystemProtocolGuid
  gEfiFirmwareVolume2ProtocolGuid
  gEfiPartitionInfoProtocolGuid
[Guids]
  gEfiFileInfoGuid
  gEfiPartTypeSystemPartGuid
''')
dsc=(libc/'StdLib/StdLib.dsc').read_text().split('[Components]')[0]
stdlib_inc=(libc/'StdLib/StdLib.inc').read_text()
dsc=dsc.replace('PLATFORM_NAME                  = StdLib','PLATFORM_NAME                  = PocketJsBoot')
dsc=dsc.replace('OUTPUT_DIRECTORY               = Build/StdLib','OUTPUT_DIRECTORY               = Build/PocketJsBoot')
# Older platform-pinned EDK II trees keep these intrinsics in ArmPkg.
intrinsics='MdePkg/Library/CompilerIntrinsicsLib/CompilerIntrinsicsLib.inf'
if not (edk/intrinsics).is_file():
 replacement='ArmPkg/Library/CompilerIntrinsicsLib/CompilerIntrinsicsLib.inf'
 if not (edk/replacement).is_file(): raise ValueError('EDK II lacks ARM compiler intrinsics')
 dsc=dsc.replace(intrinsics,replacement)
 stdlib_inc=stdlib_inc.replace(intrinsics,replacement)
stack_check='MdePkg/Library/StackCheckLib/StackCheckLib.inf'
if not (edk/stack_check).is_file():
 replacement='MdePkg/Library/BaseStackCheckLib/BaseStackCheckLib.inf'
 if not (edk/replacement).is_file(): raise ValueError('EDK II lacks stack check library')
 stdlib_inc=stdlib_inc.replace(stack_check,replacement)
(pkg/'StdLibCompat.inc').write_text(stdlib_inc)
dsc+='''
[BuildOptions]
  GCC:*_*_AARCH64_CC_FLAGS = -fno-lto -ffixed-x18
  GCC:*_*_AARCH64_DLINK_FLAGS == -Wl,--emit-relocs -nostdlib -Wl,--gc-sections -u _ModuleEntryPoint -Wl,-e,_ModuleEntryPoint -Wl,-Map,$(DEST_DIR_DEBUG)/$(BASE_NAME).map -z common-page-size=0x1000 -fno-lto -Wl,--no-undefined
  GCC:*_*_AARCH64_CC_XIPFLAGS == -mstrict-align
[LibraryClasses]
  PocketJsCore|PocketJsPkg/RustCore/RustCore.inf
  QuickJsCore|PocketJsPkg/QuickJsCore/QuickJsCore.inf
  TimerLib|MdePkg/Library/BaseTimerLibNullTemplate/BaseTimerLibNullTemplate.inf
!include PocketJsPkg/StdLibCompat.inc
[Components]
  PocketJsPkg/PocketJs.inf
'''
(pkg/'PocketJs.dsc').write_text(dsc)
upstream_lock=json.loads(Path(os.environ['PJB_UPSTREAM_LOCK']).read_text())
manifest={'edk2':subprocess.check_output(['git','-C',str(edk),'rev-parse','HEAD'],text=True).strip(),
 'edk2Libc':subprocess.check_output(['git','-C',str(libc),'rev-parse','HEAD'],text=True).strip(),
 'pocketjs':upstream_lock['dependencies']['pocketjs']['commit'],
 'upstream':upstream_lock,
 'rustArchiveSha256':hashlib.sha256(a.archive.read_bytes()).hexdigest(),
 'quickjsSources':{f.name:hashlib.sha256(f.read_bytes()).hexdigest() for f in a.quickjs.iterdir() if f.is_file()}}
(a.output/'source-manifest.json').write_text(json.dumps(manifest,indent=2)+'\n')
print(pkg)
