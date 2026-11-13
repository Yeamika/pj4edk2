#!/usr/bin/env python3
"""Headless ARM64 virt test runner with screenshots and persistent disk support."""
import argparse,json,os,re,shutil,socket,struct,subprocess,time,zlib
from pathlib import Path

def qmp(run,command,arguments=None):
    with socket.socket(socket.AF_UNIX) as s:
        s.settimeout(10);s.connect(str(run/'qmp.sock'));f=s.makefile('rwb',buffering=0);f.readline()
        def send(name,args=None):
            data={'execute':name}
            if args is not None:data['arguments']=args
            f.write((json.dumps(data)+'\n').encode())
            while True:
                reply=json.loads(f.readline())
                if 'error' in reply:raise RuntimeError(reply['error'])
                if 'return' in reply:return reply['return']
        send('qmp_capabilities');return send(command,arguments)
def main():
    p=argparse.ArgumentParser(description=__doc__)
    p.add_argument('command',choices=['start','screen','key','stop','status','dump-dtb'])
    p.add_argument('--run',type=Path,required=True)
    p.add_argument('--qemu',default='qemu-system-aarch64')
    p.add_argument('--tools',type=Path,help='optional portable QEMU sysroot')
    p.add_argument('--build',type=Path,help='build output; start with --direct or dump-dtb')
    p.add_argument('--disk',type=Path,help='GPT disk image; written persistently unless --snapshot')
    p.add_argument('--firmware-code',type=Path)
    p.add_argument('--firmware-vars',type=Path)
    p.add_argument('--direct',action='store_true',help='direct kernel smoke test; skips EFI and pj4edk2')
    p.add_argument('--dtb',type=Path)
    p.add_argument('--snapshot',action='store_true')
    p.add_argument('--display-device',choices=['virtio','ramfb'],default=None,help='ramfb preserves the EFI GOP surface for boot-output logging')
    p.add_argument('--ready-timeout',type=float,default=60)
    p.add_argument('--size',default='1920x1200')
    p.add_argument('--key',default='ret',help='QEMU qcode, e.g. ret, f2, down, esc')
    p.add_argument('--output',type=Path,help='screenshot output, PPM or PNG (PNG uses the standard library)')
    a=p.parse_args();run=a.run.resolve()
    if a.command in ['screen','key','stop','status']:
        if a.command=='stop':qmp(run,'quit')
        elif a.command=='status':print(qmp(run,'query-status'))
        elif a.command=='key':qmp(run,'send-key',{'keys':[{'type':'qcode','data':a.key}],'hold-time':100})
        else:
            dest=(a.output or run/'screen.ppm').resolve();dest.parent.mkdir(parents=True,exist_ok=True)
            ppm=dest.with_suffix('.ppm') if dest.suffix.lower()=='.png' else dest
            qmp(run,'screendump',{'filename':str(ppm)})
            if ppm!=dest:
                data=ppm.read_bytes();header=re.match(rb'P6\s+(\d+)\s+(\d+)\s+255\r?\n',data)
                if not header:raise ValueError('unsupported QEMU PPM')
                width,height=map(int,header.groups());pixels=data[header.end():]
                if len(pixels)!=width*height*3:raise ValueError('truncated PPM')
                def chunk(kind,body):return struct.pack('>I',len(body))+kind+body+struct.pack('>I',zlib.crc32(kind+body))
                rows=b''.join(b'\0'+pixels[y*width*3:(y+1)*width*3] for y in range(height))
                dest.write_bytes(b'\x89PNG\r\n\x1a\n'+chunk(b'IHDR',struct.pack('>IIBBBBB',width,height,8,2,0,0,0))+chunk(b'IDAT',zlib.compress(rows))+chunk(b'IEND',b''))
            print(dest)
        return
    if run.exists() and any(run.iterdir()):p.error('use a fresh run directory')
    if a.direct and not a.build:p.error('--direct requires --build')
    if a.command=='start' and not a.direct and not all([a.disk,a.firmware_code,a.firmware_vars]):p.error('EFI start requires --disk, --firmware-code and --firmware-vars')
    run.mkdir(parents=True,exist_ok=True);env=os.environ.copy()
    qemu=a.qemu
    if a.tools:
        tools=a.tools.resolve();qemu=str(tools/'usr/bin/qemu-system-aarch64')
        env['LD_LIBRARY_PATH']=str(tools/'usr/lib/x86_64-linux-gnu')+':'+str(tools/'lib/x86_64-linux-gnu')
        env['QEMU_MODULE_DIR']=str(tools/'usr/lib/x86_64-linux-gnu/qemu')
    width,height=map(int,a.size.split('x'))
    display_device=a.display_device or ('virtio' if a.direct or a.command=='dump-dtb' else 'ramfb')
    args=[qemu,'-machine','virt,gic-version=3,acpi=off','-cpu','cortex-a72','-accel','tcg,thread=single','-smp','1','-m','512','-nodefaults',
          '-device',(f'virtio-gpu-pci,xres={width},yres={height}' if display_device=='virtio' else 'ramfb'),'-device','qemu-xhci','-device','usb-kbd','-device','usb-tablet',
          '-display','none','-serial',f'file:{run}/serial.log','-qmp',f'unix:{run}/qmp.sock,server=on,wait=off','-no-reboot']
    args+=['-fw_cfg',f'name=opt/pj4edk2/display,string={width}x{height}']
    if a.tools:args[1:1]=['-L',str(a.tools.resolve()/'usr/share/qemu')]
    if a.disk:
        args+=['-drive',f'if=virtio,format=raw,file={a.disk.resolve()}']
        if a.snapshot:args+=['-snapshot']
    if a.command=='dump-dtb':
        args[args.index('virt,gic-version=3,acpi=off')]=f'virt,gic-version=3,acpi=off,dumpdtb={run}/virt.dtb'
        subprocess.run(args,env=env,check=True);print(run/'virt.dtb');return
    if a.direct:
        args+=['-kernel',str(a.build.resolve()/'Image'),'-initrd',str(a.build.resolve()/'initramfs.cpio.gz'),'-append','console=ttyAMA0 console=tty0 rdinit=/init']
        if a.dtb:args+=['-dtb',str(a.dtb.resolve())]
    else:
        shutil.copy2(a.firmware_vars,run/'vars.fd')
        args+=['-drive',f'if=pflash,format=raw,readonly=on,file={a.firmware_code.resolve()}',
               '-drive',f'if=pflash,format=raw,file={run}/vars.fd']
    with (run/'process.log').open('wb') as log:process=subprocess.Popen(args,env=env,stdout=log,stderr=log,stdin=subprocess.DEVNULL,start_new_session=True)
    (run/'pid').write_text(str(process.pid)+'\n');(run/'command.json').write_text(json.dumps(args,indent=2)+'\n')
    deadline=time.monotonic()+a.ready_timeout
    while time.monotonic()<deadline:
        if process.poll() is not None:raise SystemExit((run/'process.log').read_text())
        serial=run/'serial.log';text=serial.read_text(errors='replace') if serial.exists() else ''
        ready='PJ4EDK2_SMALL_READY' in text or (not a.direct and 'PJ4EDK2_FRAME_OK' in text)
        if ready and (run/'qmp.sock').exists():print('READY',process.pid);return
        time.sleep(.2)
    process.terminate();raise SystemExit('Guest readiness timeout; inspect serial.log')
if __name__=='__main__':main()
