#[path = "../src/boot_display.rs"]
mod boot_display;
use pocketjs_boot_ui::{default_config, services::DisplayRegion};
#[test]
fn validates_region_against_current_gop_and_preserves_coordinates() {
    let c=default_config();
    let r=DisplayRegion{id:"boot-output".into(),display_id:c.display_id.clone(),position:[400,60],size:[1520,1140]};
    let d=boot_display::prepare(&c,&[r.clone()],[1920,1200],true).unwrap().unwrap();
    assert_eq!((d.x,d.y,d.width,d.height,d.rotation),(400,60,1520,1140,90));
    assert!(boot_display::prepare(&c,&[],[1920,1200],true).unwrap().is_none());
    assert!(boot_display::prepare(&c,&[r.clone(),r.clone()],[1920,1200],true).is_err());
    for (position,size) in [([u32::MAX,60],[1520,1140]),([401,60],[1520,1140]),([400,60],[0,1140])] {
        assert!(boot_display::prepare(&c,&[DisplayRegion{position,size,..r.clone()}],[1920,1200],true).is_err());
    }
    assert!(boot_display::prepare(&c,&[r],[1280,800],true).is_err());
}
