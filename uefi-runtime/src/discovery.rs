//! Host-owned directory discovery, merged with board names and launch parameters.
use alloc::{format, string::String, vec::Vec};
use pocketjs_boot_ui::UiConfig;
use pocketjs_boot_ui::BootEntry;

pub fn enabled(bytes: &[u8]) -> Result<bool, String> {
    let value: serde_json::Value=serde_json::from_slice(bytes).map_err(|e|format!("discovery policy: {e}"))?;
    match value.get("esp_discovery") {
        None => Ok(false),
        Some(v) => v.as_bool().ok_or_else(||"esp_discovery must be boolean".into()),
    }
}

pub fn merge(config: &mut UiConfig, mut paths: Vec<String>, mut exists: impl FnMut(&str)->bool) {
    // Known entries retain stable IDs, names, icons and all board parameters.
    config.entries.retain(|e| e.path.starts_with("fv:") || exists(&e.path));
    let mut seen: Vec<String>=Vec::new();
    config.entries.retain(|e| {
        if seen.iter().any(|p|p.eq_ignore_ascii_case(&e.path)) { false }
        else { seen.push(e.path.clone()); true }
    });
    paths.sort_by_key(|p|p.to_ascii_lowercase());
    for path in paths {
        if config.entries.len()==8 { break; }
        if config.entries.iter().any(|e|e.path.eq_ignore_ascii_case(&path)) { continue; }
        let parts: Vec<_>=path.split('\\').filter(|p|!p.is_empty()).collect();
        if path.len()>240 || parts.len()!=3 || !parts[0].eq_ignore_ascii_case("EFI") ||
            parts[1].len()>96 || parts[1].starts_with('.') ||
            ["BOOT","pj4edk2","Tools"].iter().any(|p|parts[1].eq_ignore_ascii_case(p)) ||
            !parts[2].to_ascii_lowercase().starts_with("boot") || !parts[2].to_ascii_lowercase().ends_with(".efi") {
            continue;
        }
        let mut hash=0xcbf29ce484222325u64;
        for byte in path.to_ascii_lowercase().bytes() { hash=(hash ^ u64::from(byte)).wrapping_mul(0x100000001b3); }
        let id=format!("esp-{hash:016x}");
        if config.entries.iter().any(|e|e.id==id) { continue; }
        config.entries.push(BootEntry { id, name:parts[1].into(), path:path.clone(), icon:"system".into(), parameters:Default::default() });
    }
    if config.default_entry_id.as_ref().is_some_and(|id|!config.entries.iter().any(|e|&e.id==id)) {
        config.default_entry_id=None;
        config.timeout_seconds=0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn cfg()->UiConfig { pocketjs_boot_ui::default_config() }
    #[test]
    fn known_entries_preserve_metadata_and_paths_are_deduplicated() {
        let mut c=cfg();let known=c.entries[0].clone();
        merge(&mut c,alloc::vec![known.path.to_ascii_lowercase(),"\\EFI\\Debian\\bootx64.efi".into()],|p|p==known.path);
        assert_eq!(c.entries.len(),2);assert_eq!(c.entries[0].id,known.id);
        assert_eq!(c.entries[0].name,known.name);assert_eq!(c.entries[0].path,known.path);
        assert_eq!(c.entries[1].name,"Debian");c.validate().unwrap();
    }
    #[test]
    fn missing_default_disables_autoboot_and_discovery_is_sorted_and_stable() {
        let mut c=cfg();c.default_entry_id=Some(c.entries[0].id.clone());
        let paths=alloc::vec!["\\EFI\\Zed\\bootaa64.efi".into(),"\\EFI\\Alpha\\bootaa64.efi".into()];
        merge(&mut c,paths.clone(),|_|false);
        assert_eq!(c.timeout_seconds,0);assert_eq!(c.default_entry_id,None);assert_eq!(c.entries[0].name,"Alpha");
        let id=c.entries[0].id.clone();merge(&mut c,paths,|_|true);assert_eq!(c.entries[0].id,id);
    }
    #[test]
    fn ignores_helpers_and_bounds_menu_size() {
        let mut c=cfg();let mut paths=Vec::new();
        for n in 0..20 { paths.push(format!("\\EFI\\OS{n}\\bootaa64.efi")); }
        paths.extend(["\\EFI\\BOOT\\BOOTAA64.EFI".into(),"\\EFI\\Tools\\Shell.efi".into(),"\\EFI\\pj4edk2\\bootaa64.efi".into()]);
        merge(&mut c,paths,|_|false);assert_eq!(c.entries.len(),8);
        assert!(c.entries.iter().all(|e|e.name.starts_with("OS")));c.validate().unwrap();
    }
    #[test]
    fn policy_is_opt_in_and_rejects_mistyped_flags() {
        assert!(!enabled(b"{}").unwrap());assert!(enabled(br#"{"esp_discovery":true}"#).unwrap());
        assert!(enabled(br#"{"esp_discovery":"yes"}"#).is_err());
    }
}
