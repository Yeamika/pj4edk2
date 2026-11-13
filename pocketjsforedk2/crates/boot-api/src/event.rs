use alloc::string::String;
use serde::{Deserialize, Serialize};

/// Input delivered on the single UI execution thread; clicks use viewport pixels.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum UiEvent {
    /// Host work can animate the wait indicator independently of user input.
    Waiting {
        active: bool,
    },
    /// A platform USB transaction pauses startup and shutdown until complete.
    FastbootState { busy: bool },
    /// Battery telemetry is supplied by the BSP; it is not user interaction.
    PowerState {
        battery_percent: Option<u8>,
        external_power: Option<bool>,
        #[serde(default)]
        power: crate::PowerTelemetry,
    },
    ChargeState { available: bool, enabled: Option<bool>, #[serde(default)] requested: Option<bool> },
    ChargeConfigured { requested_enabled: bool, enabled: Option<bool>, success: bool },
    ShutdownFailed,
    BootFailed { status: String },
    /// Result of a host-owned maintenance operation. Text describes observed
    /// results; settings is present only after a verified persistent restore.
    RecoveryResult {
        operation: String,
        success: bool,
        lines: alloc::vec::Vec<String>,
        #[serde(default)]
        settings: Option<crate::services::BootSettings>,
    },
    SettingsSaved {
        seconds: u32,
    },
    SettingsSaveFailed,
    DebugState {
        enabled: bool,
    },
    DebugConfigured {
        requested_enabled: bool,
        enabled: bool,
        success: bool,
    },
    DefaultEntryRead {
        requested_id: String,
        id: Option<String>,
    },
    DefaultEntrySaveFailed {
        requested_id: String,
    },
    /// Hover and button state; movement alone does not cancel automatic boot.
    Pointer {
        x: f32,
        y: f32,
        down: bool,
    },
    Click {
        x: f32,
        y: f32,
    },
    Key {
        key: String,
    },
    Scroll {
        delta: f32,
    },
}

/// Requests from a page; the host decides whether and how to execute them.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum UiAction {
    Boot {
        id: String,
        path: String,
        #[serde(default)]
        display_regions: alloc::vec::Vec<crate::services::DisplayRegion>,
    },
    SetTimeout {
        seconds: u32,
    },
    SetDefaultEntry {
        id: String,
    },
    Shutdown,
    SetChargingEnabled { enabled: bool },
    SetDebugEnabled {
        enabled: bool,
    },
    Recovery {
        operation: String,
    },
    EnterBoot,
}
