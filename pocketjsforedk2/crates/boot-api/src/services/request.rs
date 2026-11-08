use super::{BootParameters, DisplayRegion};
use crate::BootEntry;
use alloc::vec::Vec;

/// The original runtime remains owned by the caller, alive for the whole call.
/// Firmware adapters reuse ContextTable/invoke; never serialize runtime pointers.
pub struct BootRequest<'a, Runtime: ?Sized> {
    pub entry: BootEntry,
    pub parameters: BootParameters,
    pub runtime: &'a mut Runtime,
    pub display_regions: Vec<DisplayRegion>,
}

impl<Runtime: ?Sized> BootRequest<'_, Runtime> {
    pub fn validate(&self) -> Result<(), &'static str> {
        self.parameters.validate()?;
        if self.display_regions.len() > 16 {
            return Err("too many output regions");
        }
        for (i, region) in self.display_regions.iter().enumerate() {
            region.validate(&self.parameters.displays)?;
            if self.display_regions[..i]
                .iter()
                .any(|other| other.id == region.id)
            {
                return Err("duplicate output region");
            }
        }
        Ok(())
    }
}
