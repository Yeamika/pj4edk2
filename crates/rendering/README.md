# Shared pixel presentation

`pj4edk2-rendering` provides one `no_std` presentation policy for native UEFI and the WASM viewer. `configure` enables the upstream pixel grid for paint and hit testing. `PixelResources` selects nearest texture sampling, including core-owned transformed glyph pages and baked name cards, while forwarding original resource bytes and revisions.

The native embedded runtime applies it before presentation; font/name-card preparation and the WASM capture adapter use the same crate. PocketJS core, layout, draw commands and rasterizer remain upstream. This controls texture interpolation and position snapping; analytic corner coverage and intentional opacity/fades still follow PocketJS. The default UI rasterizes density-aware font atlases directly from the selected original font and preserves glyph coverage.

For the same UI package, viewport, input sequence and timestamps, both PocketJS hosts should produce the same framebuffer. Browser zoom/display scaling can change how those framebuffer pixels appear on screen; the canvas uses nearest scaling. DOM is an independent browser rendering reference.
