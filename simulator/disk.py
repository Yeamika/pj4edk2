#!/usr/bin/env python3
"""Create an ordinary GPT disk file containing a FAT32 ESP; no root/mount needed."""
import argparse,json,os,shutil,struct,subprocess,tempfile,uuid,zlib
from pathlib import Path
SECTOR=512
ESP_TYPE=uuid.UUID('c12a7328-f81f-11d2-ba4b-00a0c93ec93b')
def write_disk(output,partition,mib):
    sectors=mib*2048;first,last=2048,sectors-34
    if partition.stat().st_size>(last-first+1)*512:raise ValueError('ESP does not fit')
    disk_guid=uuid.uuid4().bytes_le
    entries=bytearray(128*128)
    struct.pack_into('<16s16sQQQ72s',entries,0,ESP_TYPE.bytes_le,uuid.uuid4().bytes_le,first,last,0,'EFI System'.encode('utf-16-le').ljust(72,b'\0'))
    crc=zlib.crc32(entries)
    def header(current,backup,table):
        h=bytearray(512)
        struct.pack_into('<8sIIIIQQQQ16sQIII',h,0,b'EFI PART',0x10000,92,0,0,current,backup,34,sectors-34,disk_guid,table,128,128,crc)
        struct.pack_into('<I',h,16,zlib.crc32(h[:92]));return h
    with output.open('xb') as f:
        f.truncate(mib*1024*1024)
        mbr=bytearray(512);struct.pack_into('<B3sB3sII',mbr,446,0,b'\0\2\0',0xee,b'\xff'*3,1,min(sectors-1,0xffffffff));mbr[510:]=b'\x55\xaa'
        f.write(mbr);f.write(header(1,sectors-1,2));f.write(entries)
        f.seek((sectors-33)*512);f.write(entries);f.write(header(sectors-1,1,sectors-33))
        f.seek(first*512)
        with partition.open('rb') as src:shutil.copyfileobj(src,f)
def main():
    p=argparse.ArgumentParser(description=__doc__)
    p.add_argument('--output',type=Path,required=True)
    p.add_argument('--build',type=Path,required=True,help='build.py output')
    p.add_argument('--menu-efi',type=Path,help='optional generic ARM64 PocketJsBoot.efi; otherwise boot the Linux EFI stub directly')
    p.add_argument('--bridge-efi',type=Path,help='Linux display bridge for retained RAM framebuffer logging')
    p.add_argument('--baremetal-efi',type=Path,help='optional standalone color-bars demo')
    p.add_argument('--ui-package',type=Path,help='optional shared ui.pocket installed into EFI/BOOT/pj4edk')
    p.add_argument('--dtb',type=Path,help='optional fixture DTB generated from this QEMU machine')
    p.add_argument('--size-mib',type=int,default=128)
    a=p.parse_args();out=a.output.resolve();build=a.build.resolve()
    if a.size_mib<64:p.error('disk must be at least 64 MiB for FAT32')
    if out.exists():p.error('output exists; choose a new file')
    out.parent.mkdir(parents=True,exist_ok=True)
    with tempfile.TemporaryDirectory(prefix='esp-',dir=out.parent) as temp:
        temp=Path(temp);tree=temp/'tree';boot=tree/'EFI/BOOT';linux=tree/'EFI/SmallLinux';cfg=boot/'pj4edk'
        cfg.mkdir(parents=True);linux.mkdir(parents=True)
        if a.ui_package:
            if not a.menu_efi:p.error('--ui-package requires --menu-efi')
            if not 0<a.ui_package.stat().st_size<=16*1024*1024:p.error('UI package must be 1 byte to 16 MiB')
            shutil.copy2(a.ui_package,cfg/'ui.pocket')
        shutil.copy2(build/'Image',linux/'BOOTAA64.EFI');shutil.copy2(build/'initramfs.cpio.gz',linux/'initramfs.cpio.gz')
        options='console=ttyAMA0 console=tty0 rdinit=/init initrd=\\EFI\\SmallLinux\\initramfs.cpio.gz'
        entry_path='\\EFI\\SmallLinux\\BOOTAA64.EFI'
        if a.bridge_efi:
            if not a.menu_efi or a.dtb:p.error('--bridge-efi requires --menu-efi and firmware DTB (no --dtb)')
            shutil.copy2(a.bridge_efi,linux/'DisplayBridge.efi')
            options='\\EFI\\SmallLinux\\BOOTAA64.EFI '+options+' video=efifb:off quiet loglevel=7'
            entry_path='\\EFI\\SmallLinux\\DisplayBridge.efi'
        if a.dtb:shutil.copy2(a.dtb,linux/'fixture.dtb');options+=' dtb=\\EFI\\SmallLinux\\fixture.dtb'
        # Use one canonical menu for embedded defaults and generated disks.
        config=json.loads((Path(__file__).resolve().parents[1]/'uefi/config.json').read_text())
        for entry in config['entries'][:2]:
            entry['path']=entry_path
            entry['parameters']['uefi_options']=options if entry['id']=='linux-a' else options.replace('loglevel=7','loglevel=6')
        if a.baremetal_efi:
            baremetal=tree/'EFI/Baremetal';baremetal.mkdir(parents=True)
            shutil.copy2(a.baremetal_efi,baremetal/'BOOTAA64.EFI')
        (cfg/'config.json').write_text(json.dumps(config,indent=2)+'\n')
        if a.menu_efi:shutil.copy2(a.menu_efi,boot/'BOOTAA64.EFI')
        else:
            # A test-only fallback boot path; production firmware policy is unaffected.
            shutil.copy2(build/'Image',boot/'BOOTAA64.EFI')
            (tree/'startup.nsh').write_text('fs0:\\EFI\\SmallLinux\\BOOTAA64.EFI '+options+'\n')
        partition=temp/'esp.img';sectors=a.size_mib*2048
        with partition.open('wb') as f:f.truncate((sectors-34-2048+1)*512)
        subprocess.run(['mkfs.fat','-F','32','-n','PJ4EDK2',str(partition)],check=True)
        subprocess.run(['mcopy','-i',str(partition),'-s',str(tree/'EFI'),'::/'],check=True)
        if (tree/'startup.nsh').exists():subprocess.run(['mcopy','-i',str(partition),str(tree/'startup.nsh'),'::/'],check=True)
        write_disk(out,partition,a.size_mib)
    print(out)
if __name__=='__main__':main()
