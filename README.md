# PJ4EDK2 Web UI Viewer

Browser-only preview of the shared PJ4EDK2 UI, running the pinned PocketJS WASM core and software rasterizer. Builds require Cargo and the pinned Rust toolchain with `wasm32-unknown-unknown`; serving a prebuilt package only requires Python.

```sh
git submodule update --init components/pj4edk2
git -C components/pj4edk2 submodule update --init vendor/pocketjs
rustup target add wasm32-unknown-unknown --toolchain 1.93.0
python3 tools/build.py --target-dir <cargo-build-dir>
python3 tools/serve.py --port 1299
```

Open the printed address. Use `--bind 0.0.0.0` to serve on the network.

- `components/pj4edk2/`: source repository submodule (`uefi` branch), providing `default-ui/`; no duplicated UI sources.
- `viewer/`: `.pocket` reader, WASM framebuffer presentation, input and simulated host services.
- `wasm/`: Cargo integration reusing the upstream browser embedding with the shared PJ4EDK2 pixel presentation policy. Its small build adapter operates in Cargo output; the upstream submodule remains unchanged.
- `runtime/`: generated WASM binary, upstream JavaScript bindings and provenance; ignored by Git.
- `ui.pocket`: generated single-file UI package, ignored by Git.
- `index.html`: preview, top toolbar and collapsible settings drawer.
- `tools/serve.py`: standard-library HTTP server.

Submodules default to the recorded commit. `git submodule update --remote components/pj4edk2` explicitly selects the latest source branch; review compatibility and record the new gitlink in this branch. Development symlinks must not replace the published submodule entry. The selected runtime revision must contain the shared UI-package and rendering crates used by this viewer.

The default logical resolution is **1920×1200**, scaled to fit the browser. The toolbar shows the title on the left, with resolution, restart and a menu button on the right. The menu opens full settings for backend, scene, battery level, external power, charging state, voltage, current and the virtual disk. Restart applies the selected settings and starts a fresh preview clock.

The toolbar also provides simulated W202DS hardware keys in the order power, volume up, volume down, in the same row to the left of the resolution. Power sends Enter to confirm, volume up sends Up, and volume down sends Down. When the simulated device is off, power starts a fresh preview; volume keys are disabled. The toolbar scrolls horizontally on narrow screens. These keys do not change the computer's audio volume or power state.

F2 opens settings; clicking the delay field reveals its numeric keypad. Arrow keys and wheel scroll smoothly. Touch dragging follows the pointer and settles onto the nearest row on release; there is no inertia. Confirm twice to preview a boot transition; reload or apply to return.

The virtual ESP is an in-memory file tree. Import a folder, scan `.efi` paths, load/save `EFI/BOOT/pj4edk/config.json`, or export/import the tree as JSON. Default EFI files are labeled placeholders, not executables. Shutdown stops the simulated device and UI ticks; the power-on button restarts it. Charging toggles update simulated power telemetry; recovery shows simulated diagnostics/logs and can reset virtual startup settings. Nothing writes to a real partition or starts an operating system. Importing copies files into memory, without changing the source folder.

The viewer fetches `ui.pocket` once instead of loading individual page scripts and font files. The settings drawer can also load a local `.pocket`. This is the same upstream PocketJS v2 UI package accepted by UEFI at `EFI/BOOT/pj4edk/ui.pocket`. It contains source JS, a TTF and eight baked font atlases. The WASM core loads those same atlases for measurement and rasterization; this is not QuickJS bytecode. Bounds, target, host profile and integrity hashes are checked before execution.

The metrics overlay and `window.viewerStats` identify `pocketjs-wasm-software` and measure UI calls, core/raster/presentation CPU time, animation-frame intervals and WASM memory. UI nodes, layout, font measurement, transforms and pixels come from PocketJS; the browser canvas displays its framebuffer. The UI JavaScript uses the browser engine, while UEFI uses QuickJS. The menu host supplies its own events and clock over upstream direct UI operations. These measurements are neither firmware timings nor input-to-photon latency. Rendering takes place in the browser accessing the page, not on the HTTP server.

`tools/build.py` reads existing submodules and does not fetch PocketJS. `--ui DIR` selects an external UI directory; `--output FILE` selects the `.pocket` output and `--runtime-output DIR` selects the generated engine assets. `--target-dir DIR` controls Cargo output. An explicit `--build-std` can use installed `rust-src` instead of a preinstalled WASM standard library; normal builds use the installed WASM target. Serving prebuilt artifacts requires no compiler.

Settings offer two execution backends: **PocketJS (WASM)**, the default, and **Direct (browser DOM)**, a lightweight adapter. Both read the same `.pocket` and run its shared scripts. Changing backend restarts the preview and clears its clock and performance samples; resolution, power telemetry and the virtual ESP are retained. The metrics label always names the active backend. DOM timings cover JavaScript work; browser layout, paint and GPU composition are not included in its render timer. Glyph appearance and layout details can differ between the adapters.

The shared default UI uses the original bundled Inter font in both modes. Native UEFI and WASM use the same density-aware atlases, rasterized directly from the font with glyph coverage preserved; DOM loads the original font file. Page helpers suppress unchanged property/text writes and cache text measurements. Native UEFI and WASM both use `components/pj4edk2/crates/rendering`: transformed glyphs and images use nearest sampling, and paint/hit-test translations snap to the pixel grid. Canvas presentation uses nearest scaling. DOM uses its browser font renderer and remains a visual reference, with possible differences in glyph hints and corner coverage.

The WASM presenter continues to tick the upstream core. With only static UI operations, unchanged revisions skip draw/raster/canvas work; other upstream operations keep checking the draw hash so native animations continue. Its framebuffer view is reused until WASM memory changes. Animations and input still update on animation frames.

## Scope and verification

This is the `webui-viewer` component branch, separate from the `uefi` runtime. Its remote availability is determined by the repository branch list. Source availability, successful artifact generation and browser interaction are separate checks. Browser previews do not validate QEMU boot, firmware USB protocols or physical hardware.
