//! Rasterize the original UI font into density-aware upstream font atlases.
use fontdue::{Font, FontSettings};
use std::{env, fs, path::PathBuf};

#[path = "../../tools/ui-input.rs"]
mod ui_input;

fn main() {
    let source = ui_input::load(concat!(env!("CARGO_MANIFEST_DIR"), "/../../default-ui")).font;
    let font = Font::from_bytes(fs::read(source).expect("UI font"), FontSettings::default())
        .expect("valid UI font");
    let out = PathBuf::from(env::var_os("OUT_DIR").expect("Cargo output directory"));
    for compact in [false, true] {
        // Rasterize the TTF directly at twice the logical size, preserving
        // coverage. Never enlarge a small bitmap to create a larger atlas.
        let density = if compact { 1 } else { 2 };
        for (slot, size) in [14.0, 18.0, 24.0, 32.0, 54.0, 12.0, 20.0, 18.0]
            .into_iter()
            .enumerate()
        {
            // Stage labels use a heavier face baked here, without renderer changes.
            let embolden = usize::from(slot == 7 || compact);
            let line = font
                .horizontal_line_metrics(size)
                .expect("horizontal font metrics");
            let baseline = line.ascent.ceil() as usize + 2;
            let cell_h = line.new_line_size.ceil() as usize + 4;
            let glyphs = (32_u8..=126)
                .map(|code| {
                    let logical = font.metrics(code as char, size);
                    let (metrics, pixels) = font.rasterize(code as char, size * density as f32);
                    (code, logical, metrics, pixels)
                })
                .collect::<Vec<_>>();
            let cell_w = glyphs
                .iter()
                .map(|(_, m, _, _)| m.width + m.xmin.unsigned_abs() as usize + 4 + embolden)
                .max()
                .expect("ASCII glyphs");
            assert!(cell_w < 256 && cell_h < 256 && baseline < 256);
            let count = glyphs.len() + 1;
            let mut atlas = Vec::new();
            atlas.extend_from_slice(&0x4146_4344_u32.to_le_bytes());
            atlas.extend_from_slice(&3_u16.to_le_bytes());
            atlas.extend_from_slice(&(count as u16).to_le_bytes());
            atlas.extend_from_slice(&[
                cell_w as u8,
                cell_h as u8,
                baseline as u8,
                cell_h as u8,
                slot as u8,
                0,
                density as u8,
                0,
            ]);
            atlas.extend_from_slice(&[0; 8]);
            for (index, (code, metrics, _, _)) in glyphs.iter().enumerate() {
                atlas.extend_from_slice(&(*code as u32).to_le_bytes());
                atlas.extend_from_slice(&((index + 1) as u16).to_le_bytes());
                atlas.push(
                    (metrics.advance_width.round() + embolden as f32).clamp(1.0, 255.0) as u8,
                );
                atlas.push((-metrics.xmin).max(0) as u8 + 2);
            }
            let raster_w = cell_w * density;
            let raster_h = cell_h * density;
            atlas.resize(atlas.len() + raster_w * raster_h, 0);
            for (_, logical, metrics, pixels) in glyphs {
                let mut cell = vec![0; raster_w * raster_h];
                let xoff = ((-logical.xmin).max(0) + 2) * density as i32;
                let top = (baseline * density) as i32 - metrics.height as i32 - metrics.ymin;
                for y in 0..metrics.height {
                    for x in 0..metrics.width {
                        let px = metrics.xmin + xoff + x as i32;
                        let py = top + y as i32;
                        if px >= 0
                            && py >= 0
                            && (px as usize) < raster_w
                            && (py as usize) < raster_h
                        {
                            let index = py as usize * raster_w + px as usize;
                            let sample = pixels[y * metrics.width + x];
                            let coverage = if !compact {
                                sample
                            } else if sample >= 128 {
                                255
                            } else {
                                0
                            };
                            cell[index] = cell[index].max(coverage);
                            for dx in 1..=embolden * density {
                                if (px as usize) + dx < raster_w {
                                    cell[index + dx] = cell[index + dx].max(coverage);
                                }
                            }
                            if compact && (py as usize) + 1 < raster_h {
                                // Two-pixel strokes survive nearest reduction to a 360p frame.
                                cell[index + raster_w] = cell[index + raster_w].max(coverage);
                                if (px as usize) + 1 < raster_w {
                                    cell[index + raster_w + 1] =
                                        cell[index + raster_w + 1].max(coverage);
                                }
                            }
                        }
                    }
                }
                atlas.extend_from_slice(&cell);
            }
            let suffix = if compact { "-compact" } else { "" };
            fs::write(out.join(format!("font-{slot}{suffix}.atlas")), atlas)
                .expect("write font atlas");
        }
    }
}
