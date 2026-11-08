use alloc::string::String;
use serde::{Deserialize, Serialize};

/// One compact card baked by the host and rotated through ordinary image nodes.
/// Colors use PocketJS ABGR; dimensions and origins are logical pixels.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NameCardSpec {
    pub text: String,
    pub size: [u32; 2],
    pub radius: u32,
    pub fill: u32,
    pub border: u32,
    pub ink: u32,
    pub font_slot: u8,
    pub text_origin: [u32; 2],
}

impl NameCardSpec {
    pub fn validate(&self) -> Result<(), &'static str> {
        let [width, height] = self.size;
        if !(1..=512).contains(&width) || !(1..=128).contains(&height) {
            return Err("name card exceeds the texture budget");
        }
        if self.text.len() > 96
            || self.font_slot >= 7
            || self.radius > 32
            || self.radius * 2 > width.min(height)
            || self.text_origin[0] >= width
            || self.text_origin[1] >= height
        {
            return Err("invalid name card metrics");
        }
        Ok(())
    }
}
