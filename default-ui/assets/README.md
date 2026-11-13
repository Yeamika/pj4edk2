# UI fonts

The default font is the bundled `Inter-Regular.ttf`, licensed under SIL OFL 1.1; see [LICENSE-Inter.txt](LICENSE-Inter.txt). The original font file is distributed unchanged.

`ui.json` selects the font and license. Builds rasterize eight ASCII font atlases directly from the original font at twice their logical density, preserving glyph coverage. They do not enlarge low-resolution glyph bitmaps. The UI package also bundles the original font for DOM previews. Native UEFI and WASM use the same atlases and logical font sizes; viewport and browser scaling can still affect presentation.
