//! Host-owned boot services. These Rust borrows are not a cross-loader ABI.
mod display;
mod parameters;
mod request;

use crate::BootEntry;
use alloc::{string::String, vec::Vec};
use serde::{Deserialize, Serialize};

pub use display::{DisplayDevice, DisplayRegion};
pub use parameters::{BootParameters, CustomParameters};
pub use request::BootRequest;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct BootSettings {
    pub timeout_seconds: u32,
    #[serde(default)]
    pub default_entry_id: Option<String>,
}

impl BootSettings {
    pub fn validate(&self) -> Result<(), &'static str> {
        if self.timeout_seconds > 99 {
            return Err("timeout must be 0..99 seconds");
        }
        if self
            .default_entry_id
            .as_ref()
            .is_some_and(|id| id.is_empty() || id.len() > 64)
        {
            return Err("default entry identifier must contain 1..64 bytes");
        }
        Ok(())
    }
}

/// Discovery and persistence belong to the host; pages only emit requests.
/// `Runtime` is the live desktop runtime, or the existing firmware ContextTable.
pub trait BootServices {
    type Error;
    type Runtime: ?Sized;

    fn read_entries(&mut self) -> Result<Vec<BootEntry>, Self::Error>;
    fn read_settings(&mut self) -> Result<BootSettings, Self::Error>;
    fn save_settings(&mut self, settings: &BootSettings) -> Result<(), Self::Error>;
    fn read_parameters(&mut self) -> Result<BootParameters, Self::Error>;
    fn boot(&mut self, request: BootRequest<'_, Self::Runtime>) -> Result<(), Self::Error>;
    fn shutdown(&mut self) -> Result<(), Self::Error>;
    /// The runtime has changed its data gate after an explicit UI choice.
    /// The BSP may attach/detach its transport; PJB opens no device or socket.
    fn set_debug_enabled(&mut self, enabled: bool) -> Result<(), Self::Error>;
}
