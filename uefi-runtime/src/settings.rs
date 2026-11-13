//! Two independently checksummed settings records. File I/O belongs to the host.
//! A successful save requires flush and an independent readback of the new slot.
use pocketjs_boot_ui::{services::BootSettings, UiAction, UiConfig, UiEvent};

pub const RECORD_BYTES: usize = 512;
const HEADER: usize = 24;
const MAGIC: &[u8; 8] = b"PJSSET01";

pub trait Storage {
    fn read(&mut self, slot: usize, record: &mut [u8; RECORD_BYTES]) -> Result<(), ()>;
    /// Return only after file flush and close have succeeded.
    fn write(&mut self, slot: usize, record: &[u8; RECORD_BYTES]) -> Result<(), ()>;
}

fn checksum(bytes: &[u8]) -> u32 {
    let mut crc = !0u32;
    for (index, &byte) in bytes.iter().enumerate() {
        crc ^= u32::from(if (20..24).contains(&index) { 0 } else { byte });
        for _ in 0..8 { crc = (crc >> 1) ^ (0xedb88320 & 0u32.wrapping_sub(crc & 1)); }
    }
    !crc
}

fn decode(bytes: &[u8; RECORD_BYTES]) -> Option<(u64, BootSettings)> {
    if &bytes[..8] != MAGIC { return None; }
    let sequence = u64::from_le_bytes(bytes[8..16].try_into().ok()?);
    let length = u32::from_le_bytes(bytes[16..20].try_into().ok()?) as usize;
    let crc = u32::from_le_bytes(bytes[20..24].try_into().ok()?);
    if sequence == 0 || length == 0 || length > RECORD_BYTES - HEADER || crc != checksum(bytes) {
        return None;
    }
    let value: BootSettings = serde_json::from_slice(&bytes[HEADER..HEADER + length]).ok()?;
    value.validate().ok()?;
    Some((sequence, value))
}

fn encode(sequence: u64, value: &BootSettings) -> Result<[u8; RECORD_BYTES], ()> {
    value.validate().map_err(|_| ())?;
    let json = serde_json::to_vec(value).map_err(|_| ())?;
    if sequence == 0 || json.len() > RECORD_BYTES - HEADER { return Err(()); }
    let mut bytes = [0; RECORD_BYTES];
    bytes[..8].copy_from_slice(MAGIC);
    bytes[8..16].copy_from_slice(&sequence.to_le_bytes());
    bytes[16..20].copy_from_slice(&(json.len() as u32).to_le_bytes());
    bytes[HEADER..HEADER + json.len()].copy_from_slice(&json);
    let crc = checksum(&bytes);
    bytes[20..24].copy_from_slice(&crc.to_le_bytes());
    Ok(bytes)
}

pub struct Settings {
    pub value: BootSettings,
    sequence: u64,
    slot: Option<usize>,
}

impl Settings {
    pub fn load(storage: &mut impl Storage, config: &mut UiConfig) -> Self {
        let mut settings = Self {
            value: BootSettings { timeout_seconds: config.timeout_seconds, default_entry_id: config.default_entry_id.clone() },
            sequence: 0, slot: None,
        };
        for slot in 0..2 {
            let mut bytes = [0; RECORD_BYTES];
            if storage.read(slot, &mut bytes).is_err() { continue; }
            if let Some((sequence, value)) = decode(&bytes) {
                if sequence > settings.sequence { settings = Self { value, sequence, slot: Some(slot) }; }
            }
        }
        // A removed boot entry must never become a new implicit boot target.
        if settings.value.default_entry_id.as_ref().is_some_and(|id| !config.entries.iter().any(|e| &e.id == id)) {
            settings.value.default_entry_id = config.default_entry_id.clone();
        }
        config.timeout_seconds = settings.value.timeout_seconds;
        config.default_entry_id = settings.value.default_entry_id.clone();
        settings
    }

    pub fn sequence(&self) -> u64 { self.sequence }

    /// Restore the embedded entry selection with automatic boot disabled.
    /// Uses the same inactive-slot write, flush and readback as a normal save.
    pub fn restore_manual(&mut self, storage: &mut impl Storage, defaults: &UiConfig) -> Result<(), ()> {
        self.save(storage, BootSettings { timeout_seconds: 0, default_entry_id: defaults.default_entry_id.clone() })
    }

    fn save(&mut self, storage: &mut impl Storage, value: BootSettings) -> Result<(), ()> {
        let sequence = self.sequence.checked_add(1).ok_or(())?;
        let slot = self.slot.map_or(0, |slot| 1 - slot);
        let bytes = encode(sequence, &value)?;
        storage.write(slot, &bytes)?;
        let mut readback = [0; RECORD_BYTES];
        storage.read(slot, &mut readback)?;
        if readback != bytes { return Err(()); }
        self.value = value;
        self.sequence = sequence;
        self.slot = Some(slot);
        Ok(())
    }

    pub fn action(&mut self, storage: &mut impl Storage, config: &UiConfig, action: &UiAction) -> Option<UiEvent> {
        let mut value = self.value.clone();
        Some(match action {
            UiAction::SetTimeout { seconds } => {
                value.timeout_seconds = *seconds;
                if self.save(storage, value).is_ok() { UiEvent::SettingsSaved { seconds: *seconds } }
                else { UiEvent::SettingsSaveFailed }
            }
            UiAction::SetDefaultEntry { id } => {
                value.default_entry_id = Some(id.clone());
                if config.entries.iter().any(|e| &e.id == id) && self.save(storage, value).is_ok() {
                    UiEvent::DefaultEntryRead { requested_id: id.clone(), id: self.value.default_entry_id.clone() }
                } else { UiEvent::DefaultEntrySaveFailed { requested_id: id.clone() } }
            }
            _ => return None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[derive(Default)]
    struct Disk { slots: [Option<[u8; RECORD_BYTES]>; 2], fail: bool, corrupt: bool, writes: usize }
    impl Storage for Disk {
        fn read(&mut self, slot: usize, bytes: &mut [u8; RECORD_BYTES]) -> Result<(), ()> {
            *bytes = self.slots[slot].ok_or(())?; Ok(())
        }
        fn write(&mut self, slot: usize, bytes: &[u8; RECORD_BYTES]) -> Result<(), ()> {
            self.writes += 1;
            if self.fail { return Err(()); }
            self.slots[slot] = Some(*bytes);
            if self.corrupt { self.slots[slot].as_mut().unwrap()[40] ^= 1; }
            Ok(())
        }
    }
    fn config() -> UiConfig { pocketjs_boot_ui::default_config() }
    #[test]
    fn reload_preserves_both_settings_and_corrupt_latest_falls_back() {
        let mut disk = Disk::default(); let mut cfg = config();
        let id = cfg.entries[1].id.clone();
        let mut state = Settings::load(&mut disk, &mut cfg);
        assert!(matches!(state.action(&mut disk, &cfg, &UiAction::SetTimeout { seconds: 27 }), Some(UiEvent::SettingsSaved { seconds: 27 })));
        assert!(matches!(state.action(&mut disk, &cfg, &UiAction::SetDefaultEntry { id: id.clone() }), Some(UiEvent::DefaultEntryRead { .. })));
        let mut reboot = config(); let loaded = Settings::load(&mut disk, &mut reboot);
        assert_eq!(reboot.timeout_seconds, 27); assert_eq!(reboot.default_entry_id, Some(id)); assert_eq!(loaded.sequence(), 2);
        disk.slots[1].as_mut().unwrap()[30] ^= 1;
        let fallback = Settings::load(&mut disk, &mut config());
        assert_eq!(fallback.sequence(), 1); assert_eq!(fallback.value.timeout_seconds, 27);
    }
    #[test]
    fn failures_and_invalid_requests_never_acknowledge_a_save() {
        let mut disk = Disk::default(); let mut cfg = config();
        let mut state = Settings::load(&mut disk, &mut cfg);
        let before = state.value.clone();
        for (fail, corrupt) in [(true, false), (false, true)] {
            disk.fail = fail; disk.corrupt = corrupt;
            assert!(matches!(state.action(&mut disk, &cfg, &UiAction::SetTimeout { seconds: 19 }), Some(UiEvent::SettingsSaveFailed)));
            assert_eq!(state.value, before); assert_eq!(state.sequence(), 0);
        }
        let writes = disk.writes;
        assert!(matches!(state.action(&mut disk, &cfg, &UiAction::SetTimeout { seconds: 100 }), Some(UiEvent::SettingsSaveFailed)));
        assert!(matches!(state.action(&mut disk, &cfg, &UiAction::SetDefaultEntry { id: "missing".into() }), Some(UiEvent::DefaultEntrySaveFailed { .. })));
        assert_eq!(disk.writes, writes);
    }
    #[test]
    fn malformed_and_torn_records_are_rejected() {
        let valid = encode(1, &BootSettings { timeout_seconds: 9, default_entry_id: Some("test".into()) }).unwrap();
        for index in 0..RECORD_BYTES {
            let mut broken = valid; broken[index] ^= 1; assert!(decode(&broken).is_none());
        }
        for cut in 1..HEADER + 40 {
            let mut torn = valid; torn[cut..].fill(0); assert!(decode(&torn).is_none());
        }
    }
    #[test]
    fn recovery_restores_manual_defaults_and_failed_write_keeps_previous_state() {
        let mut disk=Disk::default(); let mut cfg=config();
        let mut state=Settings::load(&mut disk,&mut cfg);
        state.action(&mut disk,&cfg,&UiAction::SetTimeout { seconds: 19 });
        disk.fail=true;
        assert!(state.restore_manual(&mut disk,&cfg).is_err());
        assert_eq!(state.value.timeout_seconds,19);
        disk.fail=false;
        state.restore_manual(&mut disk,&cfg).unwrap();
        let mut reboot=config(); Settings::load(&mut disk,&mut reboot);
        assert_eq!(reboot.timeout_seconds,0);
        assert_eq!(reboot.default_entry_id,cfg.default_entry_id);
    }
}
