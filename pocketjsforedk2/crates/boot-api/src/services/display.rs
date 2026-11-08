use alloc::string::String;
use serde::{Deserialize, Serialize};

/// A rendering device; the desktop adapter describes its window, not a monitor.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DisplayDevice {
    pub id: String,
    pub size: [u32; 2],
}

/// Output rectangle in physical pixels, relative to its display device.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DisplayRegion {
    pub id: String,
    pub display_id: String,
    pub position: [u32; 2],
    pub size: [u32; 2],
}

impl DisplayRegion {
    pub fn validate(&self, devices: &[DisplayDevice]) -> Result<(), &'static str> {
        if self.id.is_empty() || self.id.len() > 64 || self.size.contains(&0) {
            return Err("invalid output region");
        }
        let device = devices
            .iter()
            .find(|device| device.id == self.display_id)
            .ok_or("unknown output display")?;
        for axis in 0..2 {
            let edge = self.position[axis]
                .checked_add(self.size[axis])
                .ok_or("output region overflows")?;
            if edge > device.size[axis] {
                return Err("output region exceeds display");
            }
        }
        Ok(())
    }
}
