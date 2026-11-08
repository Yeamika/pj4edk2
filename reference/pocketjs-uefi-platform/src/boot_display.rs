//! Validate page geometry before publishing the optional EFI boot-output table.
use pocketjs_boot_context_abi::{BootDisplay, BOOT_DISPLAY_REVISION};
use pocketjs_boot_ui::{UiConfig, services::{DisplayDevice, DisplayRegion}};

pub fn prepare(config: &UiConfig, regions: &[DisplayRegion], display: [u32; 2],
               portrait: bool) -> Result<Option<BootDisplay>, &'static str> {
    if regions.len() > 16 { return Err("too many boot regions"); }
    let devices=[DisplayDevice { id:config.display_id.clone(), size:display }];
    for (i, region) in regions.iter().enumerate() {
        region.validate(&devices)?;
        if regions[..i].iter().any(|r| r.id == region.id) { return Err("duplicate boot region"); }
    }
    let Some(r) = regions.iter().find(|r| r.id == "boot-output") else { return Ok(None); };
    if r.display_id != config.display_id { return Err("boot region is not on primary GOP"); }
    if r.position[0].checked_add(r.size[0]).is_none_or(|x| x > display[0]) ||
       r.position[1].checked_add(r.size[1]).is_none_or(|y| y > display[1]) {
        return Err("boot region exceeds actual GOP");
    }
    Ok(Some(BootDisplay { revision: BOOT_DISPLAY_REVISION, size: size_of::<BootDisplay>() as u32,
        display_width: display[0], display_height: display[1],
        x: r.position[0], y: r.position[1], width: r.size[0], height: r.size[1],
        rotation: if portrait {90} else {0} }))
}
