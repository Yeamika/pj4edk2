# Generic UEFI project

This directory builds the menu for the `qemu-arm64-small` branch. Its runtime source is the
`pj4edk2/` Git submodule tracking the `uefi` branch; the recorded gitlink pins the actual revision.
`build.py` uses that source, the pinned generic upstream EDK II and edk2-libc to
link `PocketJsBoot.efi`. The independent Linux loader is built by `../efi/build.py`.

The runtime submodule provides `build.sh` and its source-lock controls. Initialize its PocketJS and edk2-libc submodules before building. EDK II and QuickJS can be prepared by the runtime build entry. EDK II source versions are recorded in `sources.lock.json` and the
build's `upstream.lock.json`; `--edk2` can use a separately initialized checkout.
`config.json` supplies the default ARM64 menu and external initramfs options.
The simulator additionally needs an AAVMF firmware build (for example the
`qemu-efi-aarch64` package); this application is not a replacement for AAVMF.

`QemuDisplay.c` selects the requested GOP mode before the first menu frame.
For ramfb firmware lacking the requested mode, it allocates reserved RAM and
configures QEMU via fw_cfg. The fw_cfg implementation is compiled unchanged
from the selected upstream EDK II sources. The default is 1920×1200, overridden
by the simulator's `--size` option. This QEMU hardware adapter stays in this
project; generic protocol code contains no virtual-machine addresses.

`../efi/DisplayBridge.inf` builds `DisplayBridge.efi` from the linux-efi shared project. It validates
the optional BootDisplay request and actual GOP, copies the firmware DT, reserves
the coherent scanout, and starts Linux through its upstream ARM64 EFI stub.
Linux's built-in console uses the same retained surface and boot-output region.
The initramfs DRM module is separate and does not implement early logging.

The optional FDT fixture producer exercises linux-efi's common wire-format/range helpers;
it is separate from the live GOP handoff and does not access a live framebuffer.
