use alloc::{
    string::{String, ToString},
    vec::Vec,
};
use core::str::FromStr;
use serde::{Deserialize, Serialize};

/// An entry preset; the combined role shares the normal boot page.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Mode {
    Uefi,
    Loader,
    UefiLoader,
    Recovery,
    Charging,
    LowBattery,
}

impl Mode {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Uefi => "uefi",
            Self::Loader => "loader",
            Self::UefiLoader => "uefi-loader",
            Self::Recovery => "recovery",
            Self::Charging => "charging",
            Self::LowBattery => "low-battery",
        }
    }
}

impl FromStr for Mode {
    type Err = &'static str;
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "uefi" | "1" => Ok(Self::Uefi),
            "loader" | "boot" | "2" => Ok(Self::Loader),
            "uefi-loader" | "uefi+load" | "uefi+loader" | "3" => Ok(Self::UefiLoader),
            "recovery" | "4" => Ok(Self::Recovery),
            "charging" | "5" => Ok(Self::Charging),
            "low-battery" | "6" => Ok(Self::LowBattery),
            _ => Err("mode must be uefi, loader, uefi-loader, recovery, charging or low-battery"),
        }
    }
}

/// Display data supplied by a host that owns boot-entry discovery.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BootEntry {
    pub id: String,
    pub name: String,
    pub path: String,
    /// Host-selected icon name; unknown names use the generic system icon.
    #[serde(default = "default_icon")]
    pub icon: String,
    #[serde(default)]
    pub parameters: crate::services::CustomParameters,
}

/// Battery-side measurements. Positive current enters the battery; negative
/// current leaves it. External input alone does not establish charging.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct PowerTelemetry {
    pub charging: Option<bool>,
    pub voltage_mv: Option<u32>,
    pub current_ma: Option<i32>,
}

/// One-time page configuration; the host supplies monotonic time separately.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UiConfig {
    pub mode: Mode,
    pub version: String,
    pub viewport: [u32; 2],
    #[serde(default = "default_display_id")]
    pub display_id: String,
    pub timeout_seconds: u32,
    #[serde(default)]
    pub default_entry_id: Option<String>,
    pub entries: Vec<BootEntry>,
    pub battery_percent: Option<u8>,
    pub external_power: Option<bool>,
    #[serde(default)]
    pub power: PowerTelemetry,
}

impl UiConfig {
    /// Bound page input and ensure stable, unique entry identifiers.
    pub fn validate(&self) -> Result<(), &'static str> {
        let [w, h] = self.viewport;
        if !(640..=2560).contains(&w) || !(360..=2560).contains(&h) {
            return Err("viewport must be between 640x360 and 2560x2560");
        }
        self.validate_content()
    }

    /// Child output regions can be smaller than the top-level display.
    pub fn validate_embedded(&self) -> Result<(), &'static str> {
        let [w, h] = self.viewport;
        if !(320..=2560).contains(&w) || !(180..=2560).contains(&h) {
            return Err("embedded viewport must be between 320x180 and 2560x2560");
        }
        self.validate_content()
    }

    fn validate_content(&self) -> Result<(), &'static str> {
        if self
            .default_entry_id
            .as_ref()
            .is_some_and(|id| id.is_empty() || id.len() > 64)
        {
            return Err("default entry identifier must contain 1..64 bytes");
        }
        if self.display_id.is_empty() || self.display_id.len() > 64 {
            return Err("display identifier must contain 1..64 bytes");
        }
        if self.timeout_seconds > 99 {
            return Err("timeout must be 0..99 seconds");
        }
        if self.battery_percent.is_some_and(|p| p > 100) {
            return Err("battery must be 0..100");
        }
        if self.entries.len() > 8 {
            return Err("at most 8 boot entries are supported");
        }
        if self.version.is_empty() || self.version.len() > 24 || !self.version.is_ascii() {
            return Err("version must contain 1..24 ASCII bytes");
        }
        for (index, entry) in self.entries.iter().enumerate() {
            if serde_json::to_vec(&entry.parameters)
                .map_err(|_| "invalid entry parameters")?
                .len()
                > 16384
            {
                return Err("entry parameters exceed 16 KiB");
            }
            if entry.id.is_empty()
                || entry.id.len() > 64
                || entry.name.is_empty()
                || entry.name.len() > 96
                || entry.path.len() > 240
                || entry.icon.len() > 32
            {
                return Err("boot-entry field length is invalid");
            }
            if self.entries[..index]
                .iter()
                .any(|other| other.id == entry.id)
            {
                return Err("boot-entry identifiers must be unique");
            }
        }
        Ok(())
    }

    /// Encode configuration without embedding unescaped text into JavaScript.
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }
}

fn default_display_id() -> String {
    "main".to_string()
}

fn default_icon() -> String {
    "system".into()
}
