# Baremetal color bars

`Baremetal.c` builds an ARM64 EFI application. It copies the active GOP surface
information, disables the firmware watchdog, obtains the memory map and exits
Boot Services. It then disables interrupts and writes eight vertical color bars
directly to the inherited framebuffer, followed by a CPU loop. No Linux,
initramfs, GOP BLT calls or firmware services are used after ExitBootServices.

Supports standard RGBX/BGRX 32-bit GOP and RGB565 bitmask surfaces, honoring pitch
and framebuffer capacity. Bar order: white, yellow, cyan, green, magenta, red,
blue, black. Returning to the menu requires restarting the VM.

Built by `efi/build.py` as `Baremetal.efi`; install using
`simulator/disk.py --baremetal-efi /path/to/build/efi/Baremetal.efi`.
Select the Baremetal menu entry, then verify with
`python3 simulator/check-bars.py --run /path/to/run`.
