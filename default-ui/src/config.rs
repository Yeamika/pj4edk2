use alloc::{string::ToString, vec::Vec};
use pocketjs_boot_api::{BootEntry, Mode, UiConfig, services::BootSettings};

/// Demonstration policy; hosts supply their own actual entries and power facts.
pub fn default_config() -> UiConfig {
    let entries = [
        ("rimuks", "RIMUKS OS", "\\EFI\\RIMUKS\\BOOTX64.EFI"),
        (
            "windows",
            "Windows Boot Manager",
            "\\EFI\\Microsoft\\Boot\\bootmgfw.efi",
        ),
        ("shell", "UEFI Shell", "\\EFI\\Tools\\Shell.efi"),
        ("recovery", "Recovery", "\\EFI\\RIMUKS\\recovery.efi"),
    ]
    .map(|(id, name, path)| BootEntry {
        id: id.to_string(),
        name: name.to_string(),
        path: path.to_string(),
        icon: match id {
            "windows" => "windows",
            "shell" => "terminal",
            "recovery" => "recovery",
            _ => "system",
        }
        .into(),
        parameters: Default::default(),
    });
    UiConfig {
        mode: Mode::Loader,
        version: "0.1.0".to_string(),
        viewport: [1280, 720],
        display_id: "main".into(),
        timeout_seconds: 8,
        default_entry_id: None,
        entries: Vec::from(entries),
        battery_percent: Some(76),
        external_power: Some(true),
        power: Default::default(),
    }
}

pub fn default_settings() -> BootSettings {
    BootSettings { timeout_seconds: 8, default_entry_id: None }
}
