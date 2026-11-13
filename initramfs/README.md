# Tiny initramfs and custom DRM module

`init.c` is a static ARM64 PID1. It mounts proc/sysfs/devtmpfs, verifies the
virtio-gpu DRM driver and framebuffer console, loads `pj4edk2_drm.ko`,
and checks that the virtual DRM/KMS device registers (card1 in virtio mode,
card0 in ramfb mode). It then waits for inspection.
There is no BusyBox, shell, init system or root disk dependency.

`drm/` is a real out-of-tree GPL kernel module built against the test kernel.
It registers a virtual atomic DRM/KMS device with GEM shmem buffers and a
320×200 connector. It validates the public boot-display property if present.
It never maps the fixture's declared physical address and does not drive QEMU's
GPU hardware. Visible output comes from virtio-gpu in direct mode. In the complete ramfb EFI
path, the kernel built-in shared console has already been receiving and drawing
printk messages before PID1 starts. This module does not register that console.

Both the embedded initramfs and external `initramfs.cpio.gz` contain PID1 and
the module. The external form also exercises the EFI initrd-loading path. Module and kernel must be rebuilt together.
