# Small ARM64 Linux

Pinned upstream: Linux 6.12.116, commit
`5c40a1750f44dd6c7292d8c8d0b4ffff1734576e` (stable tag `v6.12.116`).
`kernel.config` starts from allnoconfig and enables EFI, ARM virt, PCI/virtio,
virtio-gpu DRM, fbdev console, modules, initramfs and basic input/filesystems.
No distribution kernel, desktop or GPU userspace stack is required.

`build.py` uses the initialized source submodule, or downloads a kernel.org archive and verifies its pinned
SHA-256. `--tarball` works offline. `--kernel-source` accepts a caller-owned,
source tree of this version. The `source/` submodule is used when initialized.
The output is always outside this repository. Incremental compilation reuses
`kernel/.config` and objects in the selected output directory.

Sources for the protocol receiver/producer are supplied through explicit
`--boot-display` and `--linux-efi` Git project paths. Header contents must match.
The external test DRM module and the static PID1 are built from `initramfs/`.

The shared `linux/pj4edk2_console.c` is built into the kernel, not loaded by
initramfs. It registers at console_initcall and starts worker rendering at
core_initcall. To keep upstream pristine, the builder creates `console-source/`
in the external output (hardlinks for immutable upstream files), replaces its
init/Makefile before adding one Kbuild entry, and stages the shared source/headers.
Original kernel/source files are never edited. Rebuild the kernel to apply shared
console changes; rebuilding only the DRM module does not update early logging.
