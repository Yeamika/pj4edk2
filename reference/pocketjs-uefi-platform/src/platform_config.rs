//! Board policy is supplied by the firmware glue using the shared UiConfig schema.
use alloc::string::String;
use pocketjs_boot_ui::UiConfig;

pub fn parse(bytes: &[u8], viewport: [u32; 2]) -> Result<UiConfig, String> {
    if bytes.len() > 65536 { return Err("platform configuration exceeds 64 KiB".into()); }
    let mut config: UiConfig = serde_json::from_slice(bytes).map_err(|e| alloc::format!("platform configuration: {e}"))?;
    config.viewport = viewport;
    config.validate().map_err(String::from)?;
    Ok(config)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn board_policy_uses_actual_gop_size_and_preserves_target() {
        let mut input = pocketjs_boot_ui::default_config();
        input.timeout_seconds = 10;
        input.default_entry_id = Some(input.entries[0].id.clone());
        let bytes = serde_json::to_vec(&input).unwrap();
        let config = parse(&bytes, [1920, 1200]).unwrap();
        assert_eq!(config.viewport, [1920, 1200]);
        assert_eq!(config.default_entry_id, input.default_entry_id);
        assert_eq!(config.timeout_seconds, 10);
    }
    #[test]
    fn rejects_invalid_policy() {
        assert!(parse(b"{}", [1920, 1200]).is_err());
        assert!(parse(&alloc::vec![b' '; 65537], [1920, 1200]).is_err());
    }
}
