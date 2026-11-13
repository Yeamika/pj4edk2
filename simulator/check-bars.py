#!/usr/bin/env python3
"""Verify the actual baremetal screen contains eight full-height color bars."""
import argparse,importlib.util,re,time
from pathlib import Path
p=argparse.ArgumentParser(description=__doc__)
p.add_argument('--run',type=Path,required=True)
p.add_argument('--timeout',type=float,default=30)
a=p.parse_args();run=a.run.resolve()
spec=importlib.util.spec_from_file_location('qemu_runner',Path(__file__).with_name('run.py'))
runner=importlib.util.module_from_spec(spec);spec.loader.exec_module(runner)
expected=[(255,255,255),(255,255,0),(0,255,255),(0,255,0),(255,0,255),(255,0,0),(0,0,255),(0,0,0)]
deadline=time.monotonic()+a.timeout
while time.monotonic()<deadline:
    log=(run/'serial.log').read_text(errors='replace')
    if 'PJ4EDK2_BAREMETAL starting' not in log:time.sleep(.2);continue
    screen=run/'baremetal-check.ppm';runner.qmp(run,'screendump',{'filename':str(screen)})
    data=screen.read_bytes();head=re.match(rb'P6\s+(\d+)\s+(\d+)\s+255\r?\n',data)
    if not head:raise SystemExit('FAIL: invalid screenshot')
    w,h=map(int,head.groups());pixels=data[head.end():]
    if len(pixels)!=w*h*3:raise SystemExit('FAIL: truncated screenshot')
    def pixel(x,y):
        offset=(y*w+x)*3;return tuple(pixels[offset:offset+3])
    if all(pixel((i*2+1)*w//16,y)==color
           for i,color in enumerate(expected) for y in [1,h//2,h-2]):
        print(f'PJ4EDK2 BAREMETAL PASS {w}x{h}: eight direct framebuffer color bars');break
    time.sleep(.2)
else:raise SystemExit('TIMEOUT: baremetal color bars not visible')
