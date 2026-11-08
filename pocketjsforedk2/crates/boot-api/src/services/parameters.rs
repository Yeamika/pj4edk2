use super::DisplayDevice;
use alloc::{collections::BTreeMap, string::String, vec::Vec};
use serde::{Deserialize, Serialize};

pub type CustomParameters = BTreeMap<String, serde_json::Value>;

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct BootParameters {
    #[serde(default)]
    pub custom: CustomParameters,
    #[serde(default)]
    pub displays: Vec<DisplayDevice>,
}

impl BootParameters {
    pub fn validate(&self) -> Result<(), &'static str> {
        if serde_json::to_vec(&self.custom)
            .map_err(|_| "invalid custom parameters")?
            .len()
            > 16384
        {
            return Err("custom parameters exceed 16 KiB");
        }
        if self.displays.len() > 8 {
            return Err("too many display devices");
        }
        for (i, device) in self.displays.iter().enumerate() {
            if device.id.is_empty()
                || device.id.len() > 64
                || device.size.contains(&0)
                || self.displays[..i].iter().any(|other| other.id == device.id)
            {
                return Err("invalid or duplicate display device");
            }
        }
        Ok(())
    }
}
