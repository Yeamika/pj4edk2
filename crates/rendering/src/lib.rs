//! Shared pixel presentation policy for native firmware and WASM hosts.
#![no_std]
use pocket_drivers::{FontView, RenderResources, TexView};

/// Snap paint and hit-test translations to the same physical pixel grid.
pub fn configure(ui: &mut pocketjs_core::Ui) {
    ui.set_stable_motion_paint(true);
}

/// Preserve source texels, including transformed glyphs and baked name cards.
/// The original resource bytes, revisions and lifetime remain with PocketJS.
pub struct PixelResources<'a, R: RenderResources + ?Sized>(pub &'a R);

impl<R: RenderResources + ?Sized> RenderResources for PixelResources<'_, R> {
    fn viewport(&self) -> (f32, f32) { self.0.viewport() }
    fn raster_revision(&self) -> u64 { self.0.raster_revision() }
    fn resource_revision(&self) -> u64 { self.0.resource_revision() }
    fn texture(&self, handle: i32) -> Option<TexView<'_>> {
        self.0.texture(handle).map(|texture| TexView { linear: false, ..texture })
    }
    fn font_atlas(&self, slot: u8) -> Option<FontView<'_>> { self.0.font_atlas(slot) }
    fn texture_revision(&self, handle: i32) -> Option<u64> { self.0.texture_revision(handle) }
    fn font_revision(&self, slot: u8) -> Option<u64> { self.0.font_revision(slot) }
    fn font_texture_width(&self, slot: u8) -> Option<u32> { self.0.font_texture_width(slot) }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pocketjs_core::{Ui, spec};

    #[test]
    fn scaled_binary_texels_keep_binary_coverage() {
        let mut ui = Ui::new();
        ui.set_viewport(3.0, 3.0);
        let pixels = [255,255,255,255, 255,255,255,0, 255,255,255,0, 255,255,255,0];
        let texture = ui.upload_texture_flags(&pixels, 2, 2, spec::psm::PSM_8888, spec::img::FLAG_LINEAR);
        assert!(texture >= 0);
        let words = [spec::draw_op::TEX_QUAD, texture as u32, 0, 3 | (3 << 16),
            0f32.to_bits(), 0f32.to_bits(), 1f32.to_bits(), 1f32.to_bits(), 0xffffffff];
        let mut filtered = [0u8; 36];
        let mut nearest = [0u8; 36];
        pocket_raster::render_scaled(&ui, &words, &mut filtered, 1);
        pocket_raster::render_scaled(&PixelResources(&ui), &words, &mut nearest, 1);
        assert!(filtered.chunks_exact(4).any(|p| p[0] != 0 && p[0] != 255));
        assert!(nearest.chunks_exact(4).all(|p| p[0] == 0 || p[0] == 255));
        assert_eq!(PixelResources(&ui).texture_revision(texture), ui.texture_revision(texture));
        assert!(ui.texture(texture).unwrap().linear, "source resource must stay unchanged");
    }
}
