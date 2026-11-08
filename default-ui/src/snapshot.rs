use alloc::string::String;
use serde::{Deserialize, Serialize};

/// Observable page state for host diagnostics and interaction verification.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UiSnapshot {
    pub mode: crate::Mode,
    pub viewport: [u32; 2],
    pub viewport_origin: [u32; 2],
    pub waiting: bool,
    pub battery_percent: Option<u8>,
    pub external_power: Option<bool>,
    #[serde(default)]
    pub power: crate::PowerTelemetry,
    #[serde(default)]
    pub power_status: String,
    #[serde(default)]
    pub power_readings: String,
    pub settings_dirty: bool,
    pub settings_saving: bool,
    pub debug_enabled: bool,
    pub debug_pending: bool,
    pub debug_failed: bool,
    pub default_entry_id: Option<String>,
    pub default_saving: bool,
    pub default_save_failed: bool,
    #[serde(default)]
    pub recovery_pending: Option<String>,
    #[serde(default)]
    pub recovery_result: bool,
    #[serde(default)]
    pub recovery_success: Option<bool>,
    #[serde(default)]
    pub recovery_page: usize,
    pub scene: String,
    pub selected: Option<usize>,
    pub armed: Option<usize>,
    pub auto_active: bool,
    pub settings_open: bool,
    pub timeout_seconds: u32,
    pub booting: bool,
    pub animation_progress: f32,
    pub emitted: bool,
}
