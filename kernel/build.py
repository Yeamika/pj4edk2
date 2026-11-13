#!/usr/bin/env python3
"""Build a pinned minimal ARM64 Linux/DRM test image outside the repository."""
import argparse,gzip,hashlib,json,os,shutil,stat,subprocess,tarfile,time,urllib.request
from pathlib import Path
HERE=Path(__file__).resolve().parent
VERSION='6.12.116'
SHA256='cb3ed3dfdd47e6ade49dc933db334fd86e24cadee920da612c262b0e9b6a511b'
def run(args,**kw): subprocess.run(list(map(str,args)),check=True,**kw)
def cpio(files):
    result=bytearray()
    for ino,(name,mode,data) in enumerate(files+[('TRAILER!!!',0,b'')],1):
        name=name.encode()+b'\0'
        fields=[ino,mode,0,0,1,0,len(data),0,0,5 if name==b"dev/console\0" else 0,1 if name==b"dev/console\0" else 0,len(name),0]
        result.extend(b'070701'+''.join(f'{v:08x}' for v in fields).encode()+name)
        result.extend(b'\0'*(-len(result)%4));result.extend(data);result.extend(b'\0'*(-len(result)%4))
    return bytes(result)
def main():
    p=argparse.ArgumentParser(description=__doc__)
    p.add_argument('--output',type=Path,required=True)
    p.add_argument('--boot-display',type=Path,default=HERE.parent/'initramfs/common',help='kernel-boot-display repository')
    p.add_argument('--linux-efi',type=Path,default=HERE.parent/'efi/common',help='linux-efi repository')
    p.add_argument('--kernel-source',type=Path,help='optional pristine Linux 6.12.116 source tree')
    p.add_argument('--tarball',type=Path,help='offline pinned kernel archive')
    p.add_argument('--cross-compile',default='aarch64-linux-gnu-')
    p.add_argument('--jobs',type=int,default=min(os.cpu_count() or 1,8))
    a=p.parse_args();out=a.output.resolve();repo=HERE.parent
    if out==repo or repo in out.parents: p.error('output must be outside the repository')
    if a.jobs<1:p.error('jobs must be positive')
    out.mkdir(parents=True,exist_ok=True);start=time.monotonic()
    display=a.boot_display.resolve();efi=a.linux_efi.resolve()
    for root,names in [(display,['include/pj4edk2_boot_display.h']),
                       (efi,['include/pj4edk2_boot_display.h','include/pj4edk2_linux_chosen.h'])]:
        for name in names:
            if not (root/name).is_file():p.error(f'missing input: {root/name}')
    if (display/'include/pj4edk2_boot_display.h').read_bytes()!=(efi/'include/pj4edk2_boot_display.h').read_bytes():
        p.error('EFI/kernel display protocol headers differ')
    source=a.kernel_source.resolve() if a.kernel_source else (HERE/'source' if (HERE/'source/Makefile').exists() else out/f'linux-{VERSION}')
    if not source.exists():
        archive=a.tarball or out/f'linux-{VERSION}.tar.xz'
        if not archive.exists():
            url=f'https://www.kernel.org/pub/linux/kernel/v6.x/linux-{VERSION}.tar.xz'
            temp=archive.with_suffix('.partial');urllib.request.urlretrieve(url,temp);temp.rename(archive)
        if hashlib.sha256(archive.read_bytes()).hexdigest()!=SHA256:raise SystemExit('kernel archive SHA256 mismatch')
        with tarfile.open(archive) as stream:stream.extractall(out,filter='data')
    text=(source/'Makefile').read_text()
    for key,value in [('VERSION','6'),('PATCHLEVEL','12'),('SUBLEVEL','116')]:
        if f'{key} = {value}\n' not in text:raise SystemExit('unexpected Linux source version')
    # Keep the upstream submodule pristine. The external working tree uses
    # hardlinks for immutable upstream files; replace (unlink) the one Kbuild
    # file before editing it, so upstream never receives generated changes.
    upstream_source=source
    source=out/'console-source'
    if not source.exists():
        shutil.copytree(upstream_source,source,copy_function=os.link,symlinks=True,
                        ignore=shutil.ignore_patterns('.git'))
        makefile=source/'init/Makefile';content=makefile.read_bytes()
        makefile.unlink();makefile.write_bytes(content+b'\nobj-y += pj4edk2-console/\n')
    integration=source/'init/pj4edk2-console';integration.mkdir(exist_ok=True)
    (integration/'Makefile').write_text('obj-y += console.o\n')
    shutil.copy2(display/'linux/pj4edk2_console.c',integration/'console.c')
    for header in (display/'include').glob('*.h'):shutil.copy2(header,integration/header.name)
    target=out/'kernel';target.mkdir(exist_ok=True)
    env=os.environ.copy();env.update(KBUILD_BUILD_TIMESTAMP='2026-01-01 00:00:00 +0000',KBUILD_BUILD_VERSION='1',KBUILD_BUILD_USER='builder',KBUILD_BUILD_HOST='qemu-arm64-small')
    make=['make','-C',source,f'O={target}','ARCH=arm64',f'CROSS_COMPILE={a.cross_compile}']
    if not (target/'.config').exists():run(make+['allnoconfig',f'KCONFIG_ALLCONFIG={HERE / "kernel.config"}'],env=env)
    required=['EFI_STUB','DRM','DRM_VIRTIO_GPU','DRM_FBDEV_EMULATION','SERIAL_AMBA_PL011_CONSOLE','DEVTMPFS','BLK_DEV_INITRD','BINFMT_ELF']
    config=(target/'.config').read_text()
    if any(f'CONFIG_{name}=y\n' not in config for name in required):raise SystemExit('required config missing; inspect kernel/.config')
    run([a.cross_compile+'gcc','-Os','-static','-s','-Wall','-Wextra','-Werror','-I'+str(display/'include'),repo/'initramfs/init.c','-o',out/'init'])
    initrd=cpio([('dev',stat.S_IFDIR|0o755,b''),('dev/console',stat.S_IFCHR|0o600,b''),('proc',stat.S_IFDIR|0o755,b''),('sys',stat.S_IFDIR|0o755,b''),('init',stat.S_IFREG|0o755,(out/'init').read_bytes())])
    (out/'initramfs.cpio').write_bytes(initrd)
    (out/'initramfs.cpio.gz').write_bytes(gzip.compress(initrd,mtime=0))
    run([source/'scripts/config','--file',target/'.config','--set-str','INITRAMFS_SOURCE',out/'initramfs.cpio','--enable','INITRAMFS_COMPRESSION_GZIP','--enable','MODULES','--enable','DRM_VIRTIO_GPU_KMS','--enable','EFI_ARMSTUB_DTB_LOADER','--enable','FB_DEVICE','--enable','OF_RESERVED_MEM','--enable','FONT_8x16','--enable','FONT_SUPPORT'])
    run(make+['olddefconfig'],env=env)
    run(make+[f'-j{a.jobs}','Image','modules'],env=env)
    module=out/'module';module.mkdir(exist_ok=True);(module/'include').mkdir(exist_ok=True)
    for name in ['Makefile','pj4edk2_drm.c']:shutil.copy2(repo/'initramfs/drm'/name,module/name)
    for header in (display/'include').glob('*.h'):shutil.copy2(header,module/'include'/header.name)
    run(make+[f'-j{a.jobs}',f'M={module}','modules'],env=env)
    # Build the final embedded and external initramfs from the same payload.
    initrd=cpio([('dev',stat.S_IFDIR|0o755,b''),('dev/console',stat.S_IFCHR|0o600,b''),('proc',stat.S_IFDIR|0o755,b''),('sys',stat.S_IFDIR|0o755,b''),
        ('init',stat.S_IFREG|0o755,(out/'init').read_bytes()),('pj4edk2_drm.ko',stat.S_IFREG|0o644,(module/'pj4edk2_drm.ko').read_bytes())])
    (out/'initramfs.cpio').write_bytes(initrd)
    (out/'initramfs.cpio.gz').write_bytes(gzip.compress(initrd,mtime=0))
    run(make+[f'-j{a.jobs}','Image'],env=env)
    shutil.copy2(target/'arch/arm64/boot/Image',out/'Image')
    # Build the host-side producer against the kernel's own libfdt sources.
    libfdt=source/'scripts/dtc/libfdt'
    files=['fdt.c','fdt_ro.c','fdt_rw.c','fdt_sw.c','fdt_wip.c','fdt_empty_tree.c','fdt_strerror.c','fdt_addresses.c']
    run([os.environ.get('HOSTCC','cc'),'-O2','-I'+str(libfdt),'-I'+str(efi/'include'),repo/'efi/fixture.c',*[libfdt/f for f in files],'-o',out/'make-fixture'])
    report={'linux_version':VERSION,'tarball_sha256':SHA256,'build_seconds':round(time.monotonic()-start,2),'artifacts':{}}
    for name in ['Image','initramfs.cpio.gz','module/pj4edk2_drm.ko']:
        path=out/name;report['artifacts'][name]={'bytes':path.stat().st_size,'sha256':hashlib.sha256(path.read_bytes()).hexdigest()}
    (out/'build.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps(report,indent=2))
if __name__=='__main__':main()
