#!/usr/bin/env python3
"""Wait for the complete initramfs/DRM integration result in the serial log."""
import argparse,time
from pathlib import Path
p=argparse.ArgumentParser(description=__doc__)
p.add_argument('--run',type=Path,required=True)
p.add_argument('--timeout',type=float,default=60)
p.add_argument('--require-fixture',action='store_true')
p.add_argument('--require-handoff',action='store_true')
a=p.parse_args();deadline=time.monotonic()+a.timeout
required=['PJ4EDK2_DRM PASS driver=virtio_gpu','PJ4EDK2_MODULE PASS virtual DRM/KMS registered','PJ4EDK2_FB PASS','PJ4EDK2_SMALL_READY']
if a.require_fixture:required+=['PJ4EDK2_DT PASS']
if a.require_handoff:
    required=[s for s in required if not s.startswith(('PJ4EDK2_DRM PASS','PJ4EDK2_FB PASS'))]
    required+=['PJ4EDK2_HANDOFF PASS early','PJ4EDK2_DT PASS']
while time.monotonic()<deadline:
    log=a.run/'serial.log';text=log.read_text(errors='replace') if log.exists() else ''
    if any(marker in text for marker in [' FAIL','Kernel panic','Unable to handle kernel']):raise SystemExit('FAIL: inspect serial.log')
    if all(marker in text for marker in required):
        if a.require_handoff and text.index('PJ4EDK2_HANDOFF PASS early')>text.index('Run /init'):
            raise SystemExit('FAIL: display console was registered after initramfs')
        print('PJ4EDK2 PASS kernel + initramfs + DRM'+(' + early retained display' if a.require_handoff else ''));break
    time.sleep(.2)
else:raise SystemExit('TIMEOUT: missing '+', '.join(marker for marker in required if marker not in text))
