# Linux EFI loader

Independent EDK II project producing `DisplayBridge.efi` and the direct-framebuffer
`Baremetal.efi` example (see `baremetal/README.md`). Sources and generic
handoff logic come from the `common/` submodule (`linux-efi` branch). The
BootDisplay ABI header comes from the pj4edk2 source checkout. Building this
application does not compile/link Rust, QuickJS or the menu.

```sh
python3 efi/build.py --output /path/to/build/efi --edk2 /path/to/edk2
# Reuse the selected EDK II checkout and BaseTools from a menu build:
python3 efi/build.py --output /path/to/build/efi --common-build /path/to/build/uefi/common
```

`--basetools` accepts existing tools; otherwise BaseTools are built from source.
The root build script builds kernel/initramfs, the UEFI menu, then this loader.
Use `simulator/disk.py --bridge-efi /path/to/build/efi/DisplayBridge.efi` to install it.

The loader validates the menu's optional display request against actual GOP,
publishes a reserved coherent framebuffer and boot-output rectangle in a copied
DT, then chainloads the Linux EFI stub. It restores the DT table if the child
returns. It does not initialize display hardware.
`fixture.c` is a separate host-side test producer for synthetic descriptors.
