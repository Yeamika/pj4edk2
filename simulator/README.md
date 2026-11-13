# ARM64 simulator

Requires QEMU `qemu-system-aarch64`, AAVMF code/variable firmware files,
Python 3.12+, mtools and dosfstools. PNG/PPM screenshots use only the Python standard library. Linux host; KVM is not
required. This emulates the QEMU `virt` machine.

`run.py` supports persistent GPT disk files, optional snapshot writes, keyboard
input, screenshots, machine DTB export and a direct-kernel smoke mode. Each run
uses a fresh directory with its own NVRAM, serial log, QMP socket and PID.
Stop a run with `run.py stop --run RUN`; stop it before editing its disk image.
The copied variable store is disposable, but disk writes persist by default.

The default resolution is 1920×1200 (`--size` overrides it). EFI mode defaults
to ramfb for retained UI logging; direct mode uses virtio-gpu. The pj4edk2 menu
and Linux display loader are built separately from `uefi/` and `efi/`.
