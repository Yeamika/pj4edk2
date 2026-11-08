#!/usr/bin/env python3
"""Run the ARM64 EFI in a disposable AAVMF guest, without system installs."""
import argparse, json, os, shutil, socket, subprocess, time
from pathlib import Path

p=argparse.ArgumentParser()
p.add_argument('--run',type=Path,required=True)
p.add_argument('--efi',type=Path,help='EFI application input for start')
p.add_argument('--tools',type=Path,help='portable QEMU sysroot for start')
p.add_argument('--size',default='2160x1080')
p.add_argument('--cpu',default='cortex-a72',help='QEMU CPU instruction model; not a real-hardware speed rating')
p.add_argument('--cpus',type=int,default=1)
p.add_argument('--memory',type=int,default=1024,help='Guest memory in MiB')
p.add_argument('--ready-timeout',type=float,default=45)
p.add_argument('--paused',action='store_true',help='Start at reset with vCPUs paused; return when QMP is available')
p.add_argument('--trace-display',action='store_true',help='Trace virtual GPU display flushes in QEMU only; leaves the EFI unchanged')
p.add_argument('--gdb',action='store_true',help='Expose the emulator debug stub on a local Unix socket for guest-stack sampling')
p.add_argument('--self-chain',action='store_true',help='Use this same app as a child-image fixture; this is not a Linux boot test')
p.add_argument('--command',choices=['start','screen','tap','key','enter','escape','quit'],default='start')
p.add_argument('--key',choices=['up','down','left','right','f2','ret','esc'],default='down')
p.add_argument('--point',default='659,900',help='Physical panel x,y for a tap')
a=p.parse_args()
root=Path(__file__).resolve().parents[1]
run=a.run.resolve();run.mkdir(parents=True,exist_ok=True)

def qmp(command,arguments=None):
    with socket.socket(socket.AF_UNIX) as s:
        s.settimeout(8);s.connect(str(run/'qmp.sock'))
        f=s.makefile('rwb',buffering=0);f.readline()
        def send(name,args=None):
            req={'execute':name}
            if args is not None:req['arguments']=args
            f.write((json.dumps(req)+'\n').encode())
            while True:
                response=json.loads(f.readline())
                if 'return' in response or 'error' in response:return response
        send('qmp_capabilities')
        return send(command,arguments)

if a.command=='start':
    if (run/'pid').exists():raise SystemExit('Use a fresh run directory')
    if not a.efi or not a.tools: raise SystemExit('start requires --efi and --tools')
    tools=a.tools.resolve()
    esp=run/'esp/EFI/BOOT';esp.mkdir(parents=True,exist_ok=True)
    shutil.copy2(a.efi.resolve(),esp/'BOOTAA64.EFI')
    if a.self_chain:
        child=run/'esp/EFI/RIMUKS/linux.efi';child.parent.mkdir(parents=True,exist_ok=True)
        shutil.copy2(a.efi.resolve(),child)
    shutil.copy2(tools/'usr/share/AAVMF/AAVMF_VARS.fd',run/'vars.fd')
    width,height=map(int,a.size.split('x'))
    env=os.environ.copy()
    env['LD_LIBRARY_PATH']=str(tools/'usr/lib/x86_64-linux-gnu')+':'+str(tools/'lib/x86_64-linux-gnu')
    env['QEMU_MODULE_DIR']=str(tools/'usr/lib/x86_64-linux-gnu/qemu')
    args=[str(tools/'usr/bin/qemu-system-aarch64'),'-L',str(tools/'usr/share/qemu'),
          '-machine','virt,gic-version=3','-accel','tcg,thread=single',
          '-cpu',a.cpu,'-smp',str(a.cpus),'-m',str(a.memory),'-nodefaults',
          '-drive',f'if=pflash,format=raw,readonly=on,file={tools}/usr/share/AAVMF/AAVMF_CODE.fd',
          '-drive',f'if=pflash,format=raw,file={run}/vars.fd',
          '-device',f'virtio-gpu-pci,xres={width},yres={height}',
          '-device','qemu-xhci','-device','usb-kbd','-device','usb-tablet',
          '-device','virtio-tablet-pci',
          '-drive',f'if=virtio,format=raw,file=fat:rw:{run}/esp',
          '-display','none','-serial',f'file:{run}/serial.log',
          '-qmp',f'unix:{run}/qmp.sock,server=on,wait=off','-no-reboot']
    if a.paused:args.append('-S')
    if a.trace_display:
        args.extend(['-trace',f'enable=virtio_gpu_cmd_res_flush,file={run}/display-trace.log'])
    if a.gdb:
        args.extend(['-chardev',f'socket,path={run}/gdb.sock,server=on,wait=off,id=profile-gdb',
                     '-gdb','chardev:profile-gdb'])
    with (run/'process.log').open('wb') as log:
        process=subprocess.Popen(args,env=env,stdin=subprocess.DEVNULL,stdout=log,stderr=log,start_new_session=True)
    (run/'pid').write_text(str(process.pid)+'\n')
    (run/'command.json').write_text(json.dumps(args,indent=2)+'\n')
    deadline=time.monotonic()+a.ready_timeout
    while time.monotonic()<deadline:
        if process.poll() is not None:raise SystemExit((run/'process.log').read_text())
        if a.paused and (run/'qmp.sock').exists():
            try:status=qmp('query-status')
            except (OSError,ValueError):status={}
            if status.get('return',{}).get('status') in ['prelaunch','paused']:
                print('PAUSED_AT_RESET',process.pid,flush=True)
                break
        log=(run/'serial.log').read_text(errors='replace') if (run/'serial.log').exists() else ''
        if not a.paused and 'PJB_UEFI_READY' in log:
            print(log[log.index('PJB_UEFI_READY'):].strip(),flush=True)
            print('PID',process.pid,flush=True)
            break
        time.sleep(.25)
    else:
        process.terminate()
        try:process.wait(timeout=5)
        except subprocess.TimeoutExpired:process.kill();process.wait()
        marker='paused QMP' if a.paused else 'READY marker'
        raise SystemExit(f'No {marker} within {a.ready_timeout:g} seconds; inspect serial.log')
elif a.command=='screen':
    print(qmp('screendump',{'filename':str(run/'screen.ppm')}))
elif a.command in ['key','enter','escape']:
    code=a.key if a.command=='key' else 'ret' if a.command=='enter' else 'esc'
    print(qmp('send-key',{'keys':[{'type':'qcode','data':code}],'hold-time':100}))
elif a.command=='tap':
    args=json.loads((run/'command.json').read_text())
    gpu=next(x for x in args if x.startswith('virtio-gpu-pci,'))
    mode=dict(x.split('=') for x in gpu.split(',')[1:])
    x,y=map(float,a.point.split(','))
    print(qmp('input-send-event',{'events':[
        {'type':'abs','data':{'axis':'x','value':round(x/(int(mode['xres'])-1)*32767)}},
        {'type':'abs','data':{'axis':'y','value':round(y/(int(mode['yres'])-1)*32767)}},
        {'type':'btn','data':{'button':'left','down':True}}]}))
    time.sleep(.15)
    print(qmp('input-send-event',{'events':[{'type':'btn','data':{'button':'left','down':False}}]}))
else:
    print(qmp('quit'))
