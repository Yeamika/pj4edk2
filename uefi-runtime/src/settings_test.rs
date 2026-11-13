//! Opt-in two-boot RAM diagnostic; never enabled in a normal runtime archive.
use alloc::{format, string::String, vec::Vec};
use pocketjs_boot_ui::{UiConfig, UiEvent, UiSnapshot};
use pocketjs_embedded_runtime::PocketJs;
use crate::{page::BootPage, settings::{Settings, Storage}};

fn snapshot(page:&BootPage, vm:&mut PocketJs)->Result<UiSnapshot,String> {
    vm.evaluate(page.generation,"bootEmit(rimuksState());").map_err(|e|format!("snapshot {e:?}"))?;
    let bytes=vm.receive(page.generation).map_err(|e|format!("snapshot receive {e:?}"))?.ok_or("missing snapshot")?;
    serde_json::from_slice(&bytes).map_err(|e|format!("snapshot JSON {e:?}"))
}

pub fn run(page:&BootPage, vm:&mut PocketJs, config:&UiConfig, settings:&mut Settings, storage:&mut impl Storage)->Result<String,String> {
    let original=crate::platform_config::parse(unsafe {
        // The regular firmware host already validated this immutable input.
        unsafe extern "C" { fn pjb_platform_config(n:*mut usize)->*const u8; }
        let mut n=0;let p=pjb_platform_config(&mut n);
        if p.is_null() || n==0 || n>65536 { return Err("test config unavailable".into()); }
        core::slice::from_raw_parts(p,n)
    },config.viewport)?;
    let second=config.entries.get(1).ok_or("self-test needs two entries")?;
    let restoring=settings.sequence()==2;
    if settings.sequence()!=0 && !restoring { return Err("self-test refuses existing settings".into()); }
    let before=snapshot(page,vm)?;
    if restoring && (before.timeout_seconds!=19 || before.default_entry_id.as_deref()!=Some(second.id.as_str())) {
        return Err("PJ4EDK2_SETTINGS_TEST reload mismatch".into());
    }
    let seconds=if restoring {original.timeout_seconds} else {19};
    let index=if restoring {original.entries.iter().position(|e|Some(&e.id)==original.default_entry_id.as_ref()).unwrap_or(0)} else {1};
    let mut input=Vec::new();
    for key in ["F2","SelectAll"] {input.push(UiEvent::Key{key:key.into()});}
    for digit in format!("{seconds}").chars() {input.push(UiEvent::Key{key:format!("{digit}")});}
    input.push(UiEvent::Key{key:"Enter".into()});
    for step in 0..2 {
        if step==1 {
            vm.evaluate(page.generation,&format!("Rimuks.requestDefault({index});")).map_err(|e|format!("default request {e:?}"))?;
        }
        let actions=page.tick(vm,1000+step*100,&input).map_err(|e|format!("test input {e:?}"))?;
        input.clear();
        if actions.len()!=1 {return Err(format!("self-test expected one action, got {actions:?}"));}
        let event=settings.action(storage,config,&actions[0]).ok_or("unexpected self-test action")?;
        page.tick(vm,1050+step*100,&[event]).map_err(|e|format!("test acknowledgement {e:?}"))?;
    }
    let after=snapshot(page,vm)?;
    if after.timeout_seconds!=seconds || after.default_entry_id.as_deref()!=Some(config.entries[index].id.as_str()) ||
        after.settings_dirty || after.settings_saving || after.default_saving || after.default_save_failed {
        return Err(format!("PJ4EDK2_SETTINGS_TEST page mismatch {after:?}"));
    }
    Ok(format!("PJ4EDK2_SETTINGS_TEST {} sequence={} timeout={} default={:?} page_ack=true",
        if restoring {"RELOAD_AND_RESTORE_OK"} else {"SAVE_OK"},settings.sequence(),after.timeout_seconds,after.default_entry_id))
}
