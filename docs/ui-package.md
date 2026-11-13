# Shared UI package

`ui.pocket` uses the upstream PocketJS **v2 `.pocket` container**, not ZIP or a new archive format. The builder calls upstream `pocket_package::embedded::encode` and verifies the result with the upstream reader. This is a source-JavaScript UI host profile, not QuickJS bytecode or an OS image.

The container holds identity, a PJ4EDK2 UI host descriptor, concatenated JavaScript, and a PAK asset directory. Assets include the original TTF and eight baked DCFA v3 font atlases. The target is `pj4edk2-ui`, host ABI 1, with an independently defined profile hash. Both readers check profile, bounds and FNV-1a64 container/variant integrity. Hashes detect corruption; they are not signatures or authentication.

```sh
python3 tools/pack-ui.py --output <build-dir>/ui.pocket --target-dir <cargo-build-dir>
```

Optional `--ui <ui-directory>` selects another `ui.json` page tree and font. It must implement the same page entry points and host UI/service API. Cargo, the pinned Rust toolchain and initialized upstream sources are required. The build embeds selected scripts and bakes their font before constructing one package; generated outputs stay in the chosen build directories.

UEFI reads `\EFI\BOOT\pj4edk\ui.pocket` from its selected ESP. A missing package uses the embedded default. A present invalid or incompatible package is rejected before execution; I/O failures are not treated as missing files. The maximum package size is 16 MiB. UEFI uses packaged atlases for both live text and baked name cards.

The browser viewer reads the same `ui.pocket`, loading the packaged atlases into the upstream PocketJS WASM software renderer. It can also select a local `.pocket` through its settings panel. The browser JavaScript engine and WASM execution differ from the firmware QuickJS and native ARM64 execution.

The plan section describes this UI host profile; it is not a compiler-generated hardware resource plan for unrelated PocketJS application hosts. Arbitrary `.pocket` applications targeting other profiles cannot be loaded as this UI.
