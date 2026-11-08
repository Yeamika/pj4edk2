//! Host-side asset preparation through the unmodified upstream UI/raster API.
//! This creates a small render tree, not another QuickJS instance.
use alloc::{vec, vec::Vec};
use pocketjs_boot_ui::NameCardSpec;
use crate::BakeError;
use pocketjs_core::{Ui, spec};

pub struct NameCardImage {
    pub pixels: Vec<u8>,
    pub width: u32,
    pub height: u32,
}

pub fn bake(card: &NameCardSpec, compact_fonts: bool) -> Result<NameCardImage, BakeError> {
    card.validate().map_err(BakeError::InvalidSpec)?;
    let [width, height] = card.size;
    let (texture_w, texture_h) = (width.next_power_of_two(), height.next_power_of_two());
    let mut ui = Ui::new();
    ui.set_viewport(texture_w as f32, texture_h as f32);
    if !ui.load_font_atlas(crate::atlases(compact_fonts)[card.font_slot as usize]) {
        return Err(BakeError::InvalidFont);
    }
    let node = ui.create_node(spec::NodeType::View as u8);
    for (prop, value) in [
        (spec::prop::WIDTH, width as f64),
        (spec::prop::HEIGHT, height as f64),
        (spec::prop::BG_COLOR, card.fill as f64),
        (spec::prop::BORDER_COLOR, card.border as f64),
        (spec::prop::BORDER_WIDTH, 1.0),
        (spec::prop::RADIUS, card.radius as f64),
        (spec::prop::OVERFLOW, spec::Overflow::Hidden as u8 as f64),
    ] {
        ui.set_prop(node, prop, value);
    }
    ui.insert_before(spec::ROOT_ID, node, 0);
    let text = ui.create_node(spec::NodeType::Text as u8);
    for (prop, value) in [
        (spec::prop::POS_TYPE, spec::PosType::Absolute as u8 as f64),
        (spec::prop::INSET_L, card.text_origin[0] as f64),
        (spec::prop::INSET_T, card.text_origin[1] as f64),
        (
            spec::prop::WIDTH,
            width.saturating_sub(card.text_origin[0] * 2) as f64,
        ),
        (spec::prop::TEXT_COLOR, card.ink as f64),
        (spec::prop::FONT_SLOT, card.font_slot as f64),
    ] {
        ui.set_prop(text, prop, value);
    }
    ui.set_text(text, &card.text);
    ui.insert_before(node, text, 0);
    ui.tick();
    ui.draw();
    let mut pixels = vec![0; (texture_w * texture_h * 4) as usize];
    pocket_raster::render_scaled_premultiplied(&ui, &ui.current_draw_list().words, &mut pixels, 1);
    // Texture uploads use straight alpha; preserve the baked edge coverage.
    let fill = card.fill.to_le_bytes();
    for pixel in pixels.chunks_exact_mut(4) {
        let alpha = pixel[3] as u32;
        for channel in 0..3 {
            pixel[channel] = if alpha == 0 {
                fill[channel]
            } else {
                ((pixel[channel] as u32 * 255 + alpha / 2) / alpha).min(255) as u8
            };
        }
    }
    Ok(NameCardImage {
        pixels,
        width: texture_w,
        height: texture_h,
    })
}

