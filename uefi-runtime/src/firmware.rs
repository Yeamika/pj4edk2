//! UEFI Boot Services host. All protocols and the VM expire before returning.
use alloc::{format, rc::Rc, string::String, vec::Vec};
use core::{cell::Cell, ffi::c_int, ptr};
use r_efi::{efi, protocols::{absolute_pointer as abs, graphics_output as gop, simple_text_input as key}};
use pocketjs_boot_ui::{UiAction, UiEvent};
use pocketjs_embedded_runtime::{Config, PocketJs, output::*};
use crate::page::BootPage;

unsafe extern "C" {
    fn pjb_log(text: *const u8, length: usize);
    fn pjb_log_read(text: *mut u8, capacity: usize) -> usize;
    fn pjb_platform_config(length: *mut usize) -> *const u8;
    fn pjb_start_entry(path:*const u16,options:*const u16,display:*const pocketjs_boot_context_abi::BootDisplay)->usize;
    fn pjb_charge_read(available:*mut u32,enabled:*mut u32,requested:*mut u32)->usize;
    fn pjb_shutdown(system:*const efi::SystemTable)->usize;
    fn pjb_charge_set(enabled:u32)->usize;
    fn pjb_read_power(sample:*mut pocketjs_boot_context_abi::PowerSample)->usize;
    fn pjb_settings_read(slot:usize, bytes:*mut u8, length:usize)->usize;
    fn pjb_settings_write(slot:usize, bytes:*const u8, length:usize)->usize;
}
fn log(text: &str) { unsafe { pjb_log(text.as_ptr(),text.len()); } }

struct EspSettings;
impl crate::settings::Storage for EspSettings {
    fn read(&mut self, slot:usize, bytes:&mut [u8;crate::settings::RECORD_BYTES])->Result<(),()> {
        let status=unsafe { pjb_settings_read(slot,bytes.as_mut_ptr(),bytes.len()) };
        if status==0 {Ok(())} else {Err(())}
    }
    fn write(&mut self, slot:usize, bytes:&[u8;crate::settings::RECORD_BYTES])->Result<(),()> {
        let status=unsafe { pjb_settings_write(slot,bytes.as_ptr(),bytes.len()) };
        log(&format!("PJB_SETTINGS_WRITE slot={slot} status=0x{status:x}"));
        if status==0 {Ok(())} else {Err(())}
    }
}

#[derive(Clone,Copy,Default)]
struct BlitStats { regions:usize, pixels:u64, calls:usize, rotate_ms:u64, blt_ms:u64 }
struct GopDisplay { protocol: *mut gop::Protocol, pixels: Vec<u8>, scanout: Vec<u8>, portrait:bool, physical_width:usize, descriptor: DisplayDescriptor, stats:Rc<Cell<BlitStats>> }
impl DisplayOutput for GopDisplay {
    type Token=();
    fn descriptor(&self)->DisplayDescriptor { self.descriptor }
    fn frame_identity(&mut self)->Result<Option<BufferIdentity>,DisplayError> {
        Ok(Some(BufferIdentity{buffer:0,generation:0}))
    }
    fn acquire(&mut self, region:Rect)->Result<Target<'_>,DisplayError> {
        let offset=region.y as usize*self.descriptor.stride_bytes+region.x as usize*4;
        let pixels=self.pixels.get_mut(offset..).ok_or(DisplayError::InvalidTarget)?;
        Ok(Target::CpuVisible(Surface::new(pixels,region,self.descriptor.stride_bytes,PixelFormat::Argb8888)?))
    }
    fn flush(&mut self,regions:&[Rect])->Result<(),DisplayError> {
        let mut stats=self.stats.get();
        let merged=crate::rotation::combine(regions);
        let regions=merged.as_ref().map_or(regions,core::slice::from_ref);
        for region in regions {
            stats.regions+=1;stats.pixels+=u64::from(region.width)*u64::from(region.height);
            if region.width==0 || region.height==0 {continue;}
            let strip_width=if self.portrait {(self.scanout.len()/(region.height as usize*4)) as u32} else {region.width};
            if strip_width==0 {return Err(DisplayError::InvalidTarget);}
            let mut left=region.x;
            while left<region.x+region.width {
                let part=Rect{x:left,width:strip_width.min(region.x+region.width-left),..*region};
                let (mapped,sx,sy,delta,buffer)=if self.portrait {
                    let started=counter();
                    crate::rotation::copy_compact(&self.pixels,self.descriptor.width as usize,part,&mut self.scanout);
                    stats.rotate_ms+=elapsed_ms(started);
                    (crate::rotation::region(self.descriptor.height,part),0,0,part.height as usize*4,self.scanout.as_mut_ptr())
                } else {(part,part.x as usize,part.y as usize,self.physical_width*4,self.pixels.as_mut_ptr())};
                let started=counter();
                let status=unsafe { ((*self.protocol).blt)(self.protocol,buffer.cast(),gop::BLT_BUFFER_TO_VIDEO,
                    sx,sy,mapped.x as usize,mapped.y as usize,mapped.width as usize,mapped.height as usize,delta) };
                stats.calls+=1;stats.blt_ms+=elapsed_ms(started);
                self.stats.set(stats);
                if status.is_error() { return Err(DisplayError::Unsupported); }
                left+=part.width;
            }
        }
        Ok(())
    }
    fn wait(&mut self,_:())->Result<(),DisplayError> { Ok(()) }
    fn vsync(&mut self)->VsyncSource { VsyncSource::None }
}

unsafe fn locate<T>(bs:&efi::BootServices,guid:&efi::Guid)->Option<*mut T> {
    let mut interface=ptr::null_mut();
    let status=unsafe { (bs.locate_protocol)(guid as *const _ as *mut _,ptr::null_mut(),&mut interface) };
    if status.is_error() || interface.is_null() { None } else { Some(interface.cast()) }
}

fn events(st:&efi::SystemTable,touch:Option<*mut abs::Protocol>,size:[u32;2],portrait:bool,last:&mut Option<[f32;2]>,origin:&mut Option<[f32;2]>,moved:&mut bool,trace:&mut usize)->Result<(Vec<UiEvent>,bool),String> {
    let mut output=Vec::new();
    if !st.con_in.is_null() {
        for _ in 0..16 {
            let mut input=key::InputKey::default();
            let status=unsafe { ((*st.con_in).read_key_stroke)(st.con_in,&mut input) };
            if status==efi::Status::NOT_READY { break; }
            if status.is_error() { return Err(format!("key error {status:?}")); }
            if input.scan_code==0x17 { return Ok((output,true)); }
            let name=match (input.scan_code,input.unicode_char) { (1,_)=>"ArrowUp",(2,_)=>"ArrowDown",
                (3,_)=>"ArrowRight",(4,_)=>"ArrowLeft",(0x0c,_)=>"F2",(_,13)=>"Enter",(_,9)=>"Tab",(_,8)=>"Backspace",_=>"Interaction" };
            output.push(UiEvent::Key{key:name.into()});
        }
    }
    if let Some(touch)=touch {
        for _ in 0..16 {
            let mut state=abs::State::default();
            let status=unsafe { ((*touch).get_state)(touch,&mut state) };
            if status==efi::Status::NOT_READY { break; }
            if status.is_error() {
                // A corrupt packet or overflow cancels the gesture, never a click.
                if let Some([x,y])=last.take() { output.push(UiEvent::Pointer{x,y,down:false}); }
                *origin=None; *moved=false;
                break;
            }
            if unsafe { (*touch).mode.is_null() } { return Err("absolute pointer mode unavailable".into()); }
            let mode=unsafe { &*(*touch).mode };
            if mode.absolute_max_x<=mode.absolute_min_x || mode.absolute_max_y<=mode.absolute_min_y { return Err("invalid absolute pointer axes".into()); }
            let point=[(state.current_x.saturating_sub(mode.absolute_min_x).min(mode.absolute_max_x-mode.absolute_min_x) as f64/(mode.absolute_max_x-mode.absolute_min_x) as f64*(size[0]-1) as f64) as f32,
                (state.current_y.saturating_sub(mode.absolute_min_y).min(mode.absolute_max_y-mode.absolute_min_y) as f64/(mode.absolute_max_y-mode.absolute_min_y) as f64*(size[1]-1) as f64) as f32];
            let point=if portrait { crate::rotation::touch(size[0],point) } else { point };
            if *trace<8 {
                log(&format!("PJB_POINTER raw={},{} buttons={} logical={},{}",state.current_x,state.current_y,state.active_buttons,point[0],point[1]));
                *trace+=1;
            }
            if state.active_buttons&abs::TOUCH_ACTIVE!=0 {
                if let Some(start)=origin { if (start[0]-point[0]).abs()>24. || (start[1]-point[1]).abs()>24. { *moved=true; } }
                else { *origin=Some(point);*moved=false; }
                *last=Some(point);output.push(UiEvent::Pointer{x:point[0],y:point[1],down:true});
            } else if let Some([x,y])=last.take() {
                output.push(UiEvent::Pointer{x,y,down:false});
                if !*moved { output.push(UiEvent::Click{x,y}); }
                *origin=None;
            }
        }
    }
    Ok((output,false))
}

struct BootRequest { path:String, options:String, display:Option<pocketjs_boot_context_abi::BootDisplay> }
fn pixel_buffer(bytes:usize)->Result<Vec<u8>,String> {
    let mut buffer=Vec::new();
    buffer.try_reserve_exact(bytes).map_err(|_|format!("display allocation failed bytes={bytes}"))?;
    buffer.resize(bytes,0);Ok(buffer)
}
fn run(st:&efi::SystemTable,child_status:Option<usize>)->Result<Option<BootRequest>,String> {
    let boot_started=counter();
    if st.boot_services.is_null() { return Err("Boot Services unavailable".into()); }
    let bs=unsafe { &*st.boot_services };
    let gop=unsafe { locate::<gop::Protocol>(bs,&gop::PROTOCOL_GUID) }.ok_or("GOP unavailable")?;
    if unsafe { (*gop).mode.is_null() || (*(*gop).mode).info.is_null() } { return Err("invalid GOP mode".into()); }
    let info=unsafe { *(*(*gop).mode).info };
    let physical=[info.horizontal_resolution,info.vertical_resolution];
    let portrait=physical[1]>physical[0];
    let [w,h]=if portrait { [physical[1],physical[0]] } else { physical };
    // Prefer the console splitter, which aggregates connected touch devices.
    let mut console_touch=ptr::null_mut();
    let touch_status=(bs.handle_protocol)(st.console_in_handle,&abs::PROTOCOL_GUID as *const _ as *mut _,&mut console_touch);
    let touch=if !touch_status.is_error()&&!console_touch.is_null() {Some(console_touch.cast::<abs::Protocol>())}
        else {unsafe {locate::<abs::Protocol>(bs,&abs::PROTOCOL_GUID)}};
    log(&format!("PJB_UEFI_READY GOP={}x{} UI={w}x{h} clockwise={portrait} absolute_pointer={} keyboard={} official_guest=true",physical[0],physical[1],touch.is_some(),!st.con_in.is_null()));
    let mut config_length = 0usize;
    let config_pointer = unsafe { pjb_platform_config(&mut config_length) };
    if config_pointer.is_null() || config_length == 0 || config_length > 65536 {
        return Err("missing or oversized platform configuration".into());
    }
    let config_bytes = unsafe { core::slice::from_raw_parts(config_pointer, config_length) };
    let mut config = crate::platform_config::parse(config_bytes, [w, h])?;
    let embedded_defaults=config.clone();
    let mut storage=EspSettings;
    let mut settings=crate::settings::Settings::load(&mut storage,&mut config);
    // A returning application (including a successful diagnostic) comes back
    // to a manual menu. This override never changes the settings on the ESP.
    if child_status.is_some() { config.timeout_seconds=0; }
    log(&format!("PJB_SETTINGS_LOAD sequence={} timeout={} default={:?}",settings.sequence(),config.timeout_seconds,config.default_entry_id));
    let initial=read_power(true);
    log(&format!("PJB_POWER initial={initial:?}"));
    config.battery_percent=initial.percent;
    config.external_power=initial.external;
    config.power=initial.telemetry;
    config.validate().map_err(String::from)?;
    let bytes=w as usize*h as usize*4;
    let mut vm=PocketJs::new(Config{width:w,height:h,tick_hz:30,js_heap_bytes:16*1024*1024,
        js_stack_bytes:512*1024,interrupt_polls:100_000,frame_buffer_budget:bytes}).map_err(|e|format!("VM {e:?}"))?;
    let mut descriptor=DisplayDescriptor::new(DisplayRole::Primary,w,h,PixelFormat::Argb8888,w as usize*4,TargetOwnership::CpuVisible);
    // The logical buffer and GOP retain untouched pixels between submissions.
    descriptor.partial_refresh=true;
    let blit_stats=Rc::new(Cell::new(BlitStats::default()));
    let pixels=pixel_buffer(bytes)?;
    let scanout=if portrait {pixel_buffer(bytes.min(2*1024*1024))?} else {Vec::new()};
    log(&format!("PJB_DISPLAY_BUFFERS logical={} rotation={}",pixels.len(),scanout.len()));
    vm.add_display(GopDisplay{protocol:gop,pixels,scanout,portrait,physical_width:physical[0] as usize,descriptor,stats:blit_stats.clone()},RendererId::SOFTWARE).map_err(|e|format!("display {e:?}"))?;
    let page=BootPage::load(&mut vm,&config).map_err(|e|format!("page {e:?}"))?;
    #[cfg(feature = "settings-self-test")]
    {
        let result=crate::settings_test::run(&page,&mut vm,&config,&mut settings,&mut storage)?;
        log(&result);
        vm.complete_presentation().map_err(|e|format!("settings test finish {e:?}"))?;
        return Ok(None);
    }
    #[cfg(feature = "charge-self-test")]
    {
        let test=charge_self_test(&page,&mut vm,bs);
        // A diagnostic failure must also request stop before returning.
        let stopped=configure_charge(false);
        log(&format!("PJB_CHARGE_TEST cleanup={stopped:?}"));
        test?;
        vm.complete_presentation().map_err(|e|format!("charge test finish {e:?}"))?;
        return Ok(None);
    }
    let mut estimated=initial.estimated;
    page.set_battery_estimated(&mut vm,estimated).map_err(|e|format!("power flag {e:?}"))?;
    let mut last=None;let mut origin=None;let mut moved=false;let mut trace=0;
    let started=counter();
    let mut next_power=0;
    let mut next_frame_log=0;
    let mut next_slow_log=0;
    let mut frames=0u64;
    let mut tick_total=0u64;
    let mut tick_max=0u64;
    let mut startup_ms=0u64;
    let mut settings_events=Vec::new();
    if let Some(status)=child_status.filter(|status|*status!=0) {
        settings_events.push(UiEvent::BootFailed{status:format!("0x{status:x}")});
    }
    // Stay interactive until an explicit exit/boot request. Recovery watchdog
    // policy belongs to the platform, not an invisible application deadline.
    loop {
        let now=elapsed_ms(started);
        let (mut input,exit)=events(st,touch,physical,portrait,&mut last,&mut origin,&mut moved,&mut trace)?;
        if exit { break; }
        input.append(&mut settings_events);
        if now>=next_power {
            let power=read_power(now>=next_frame_log);
            if power.estimated!=estimated {
                page.set_battery_estimated(&mut vm,power.estimated).map_err(|e|format!("power flag {e:?}"))?;
                estimated=power.estimated;
            }
            input.push(UiEvent::PowerState{battery_percent:power.percent,external_power:power.external,power:power.telemetry});
            let (available,enabled,requested)=read_charge();
            input.push(UiEvent::ChargeState{available,enabled,requested});
            next_power=now+1000;
        }
        let tick_started=counter();
        blit_stats.set(BlitStats::default());
        let actions=page.tick(&mut vm,now,&input).map_err(|e|format!("tick {e:?}"))?;
        let tick_ms=elapsed_ms(tick_started);
        if frames==0 { startup_ms=elapsed_ms(boot_started); }
        if tick_ms>=30 && now>=next_slow_log {
            let stats=blit_stats.get();
            log(&format!("PJB_FRAME_SLOW elapsed_ms={now} tick_ms={tick_ms} stages_ms={:?} input={} regions={} pixels={} blits={} rotate_ms={} blt_ms={}",page.timing.get(),input.len(),stats.regions,stats.pixels,stats.calls,stats.rotate_ms,stats.blt_ms));
            next_slow_log=now+1000;
        }
        frames+=1;tick_total+=tick_ms;tick_max=tick_max.max(tick_ms);
        if now>=next_frame_log {
            log(&format!("PJB_FRAME_OK elapsed_ms={now} frames={frames} tick_avg_ms={} tick_max_ms={tick_max} stages_ms={:?} run_to_first_submit_ms={startup_ms}",tick_total/frames,page.timing.get()));
            next_frame_log=now+10_000;
        }
        for action in actions {
            if let Some(event)=settings.action(&mut storage,&config,&action) {
                log(&format!("PJB_SETTINGS_RESULT sequence={} event={event:?}",settings.sequence()));
                settings_events.push(event);
                continue;
            }
            match action {
                UiAction::Recovery{operation}=>{
                    let mut lines=Vec::new();
                    let mut success=true;
                    let mut restored=None;
                    match operation.as_str() {
                        "diagnostics"=>{
                            let power=read_power(false);
                            let (available,enabled,requested)=read_charge();
                            lines.push(format!("UEFI revision: 0x{:08x}",st.hdr.revision));
                            lines.push(format!("Display: {} x {}",physical[0],physical[1]));
                            lines.push(format!("Touch protocol: {}",if touch.is_some(){"available"}else{"unavailable"}));
                            lines.push(format!("Battery: {:?}%, external power: {:?}",power.percent,power.external));
                            lines.push(format!("Power readings: {:?}",power.telemetry));
                            lines.push(format!("Charge control: {available}, enabled: {enabled:?}"));
                            lines.push(format!("Charge requested: {requested:?}"));
                            lines.push(format!("Loaded settings sequence: {}",settings.sequence()));
                            for slot in 0..2 {
                                let mut bytes=[0;crate::settings::RECORD_BYTES];
                                let status=unsafe {pjb_settings_read(slot,bytes.as_mut_ptr(),bytes.len())};
                                lines.push(format!("ESP settings bank {slot} read: 0x{status:x}"));
                            }
                        }
                        "boot-log"=>{
                            let mut bytes=[0u8;8192];
                            let length=unsafe {pjb_log_read(bytes.as_mut_ptr(),bytes.len())}.min(bytes.len());
                            lines.push("Recent messages from this UEFI application session:".into());
                            let text=String::from_utf8_lossy(&bytes[..length]);
                            // Keep the newest complete messages; oldest ring data may be partial.
                            let recent:Vec<_>=text.lines().rev().take(40).collect();
                            for line in recent.into_iter().rev() { lines.push(line.into()); }
                            if length==0 { lines.push("No application log is available.".into()); }
                        }
                        "repair"=>{
                            success=settings.restore_manual(&mut storage,&embedded_defaults).is_ok();
                            if success {
                                restored=Some(settings.value.clone());
                                lines.push("Startup settings saved and read back successfully.".into());
                                lines.push("Automatic startup is disabled (timeout 0).".into());
                                lines.push(format!("Default entry: {:?}",settings.value.default_entry_id));
                            } else {
                                lines.push("Could not verify the startup settings write.".into());
                                lines.push("The current menu settings have been kept.".into());
                            }
                        }
                        _=>{ success=false;lines.push("This recovery operation is not supported.".into()); }
                    }
                    log(&format!("PJB_RECOVERY operation={operation} success={success}"));
                    settings_events.push(UiEvent::RecoveryResult{operation,success,lines,settings:restored});
                }
                UiAction::Shutdown=>{
                    vm.complete_presentation().map_err(|e|format!("shutdown presentation {e:?}"))?;
                    let status=unsafe {pjb_shutdown(st)};
                    log(&format!("PJB_SHUTDOWN_RETURNED status=0x{status:x}"));
                    settings_events.push(UiEvent::ShutdownFailed);
                    next_power=0;
                }
                UiAction::SetChargingEnabled{enabled}=>{
                    settings_events.push(configure_charge(enabled));
                    next_power=0;
                }
                UiAction::Boot{id,path,display_regions}=>{
                    vm.complete_presentation().map_err(|e|format!("quiesce {e:?}"))?;
                    let entry=config.entries.iter().find(|e|e.id==id&&e.path==path).ok_or("unknown boot entry")?;
                    let options=entry.parameters.get("uefi_options").and_then(|v|v.as_str()).unwrap_or("").into();
                    let display=crate::boot_display::prepare(&config,&display_regions,[w,h],portrait)?;
                    log(&format!("PJB_BOOT_REQUEST id={id} path={path} display={display:?}"));
                    return Ok(Some(BootRequest{path,options,display}));
                }
                _=>{}
            }
        }
        // Rendering/input already consumed part of this frame's time budget.
        // Do not add another full frame of latency after a slow update.
        let spent=elapsed_ms(started).saturating_sub(now);
        if spent<16 {unsafe { (bs.stall)((16-spent) as usize*1000); }}
    }
    vm.complete_presentation().map_err(|e|format!("finish {e:?}"))?;
    Ok(None)
}

fn configure_charge(enabled:bool)->UiEvent {
    let status=unsafe {pjb_charge_set(u32::from(enabled))};
    let (_,actual,_)=read_charge();
    let success=status==0 && actual==Some(enabled);
    log(&format!("PJB_CHARGE_SET requested={enabled} actual={actual:?} status=0x{status:x}"));
    UiEvent::ChargeConfigured{requested_enabled:enabled,enabled:actual,success}
}

#[cfg(feature = "charge-self-test")]
fn charge_self_test(page:&BootPage,vm:&mut PocketJs,bs:&efi::BootServices)->Result<(),String> {
    let (available,enabled,requested)=read_charge();
    if !available || enabled!=Some(false) {return Err(format!("charge initial {available} {enabled:?}"));}
    // Use the visible button's keyboard path, then the normal host handler.
    page.tick(vm,0,&[UiEvent::ChargeState{available,enabled,requested},UiEvent::Key{key:"ArrowUp".into()}])
        .map_err(|e|format!("charge focus {e:?}"))?;
    for (step,want) in [true,false].into_iter().enumerate() {
        let actions=page.tick(vm,100+step as u64*4000,&[UiEvent::Key{key:"Enter".into()}]).map_err(|e|format!("charge click {e:?}"))?;
        if actions!=[UiAction::SetChargingEnabled{enabled:want}] {return Err(format!("charge actions {actions:?}"));}
        let event=configure_charge(want);
        if !matches!(event,UiEvent::ChargeConfigured{success:true,..}) {return Err(format!("charge rejected {event:?}"));}
        page.tick(vm,150+step as u64*4000,&[event]).map_err(|e|format!("charge ack {e:?}"))?;
        vm.evaluate(page.generation,"bootEmit(rimuksState());").map_err(|e|format!("charge state {e:?}"))?;
        let raw=vm.receive(page.generation).map_err(|e|format!("charge receive {e:?}"))?.ok_or("charge state absent")?;
        let state:serde_json::Value=serde_json::from_slice(&raw).map_err(|e|format!("charge JSON {e:?}"))?;
        if state["charge_enabled"]!=want || state["charge_pending"]!=false || state["charge_failed"]!=false {
            return Err(format!("charge page mismatch {state}"));
        }
        unsafe {(bs.stall)(3_000_000);}
        log(&format!("PJB_CHARGE_TEST enabled={want} power={:?} page_ack=true",read_power(true)));
    }
    log("PJB_CHARGE_TEST ON_OFF_OK");
    Ok(())
}

fn read_charge()->(bool,Option<bool>,Option<bool>) {
    let (mut available,mut enabled,mut requested)=(0,2,2);
    let status=unsafe {pjb_charge_read(&mut available,&mut enabled,&mut requested)};
    let actual=if status==0 && enabled<=1 {Some(enabled==1)} else {None};
    (available==1,actual,if requested<=1 {Some(requested==1)} else {actual})
}

fn read_power(trace:bool)->crate::power::PowerState {
    let mut sample=pocketjs_boot_context_abi::PowerSample::default();
    let status=unsafe { pjb_read_power(&mut sample) };
    if status!=0 { if trace { log(&format!("PJB_POWER status=0x{status:x}")); } return crate::power::PowerState::default(); }
    if trace { log(&format!("PJB_POWER valid=0x{:x} raw={} mv={} current_ma={} saved=0x{:04x} percent={} external={}",sample.valid,sample.voltage_raw,sample.voltage_mv,sample.current_ma,sample.saved_soc_raw,sample.percent,sample.external_power)); }
    crate::power::decode(&sample)
}

pub(crate) fn counter()->u64 { let value;unsafe { core::arch::asm!("mrs {}, cntvct_el0",out(reg)value,options(nomem,nostack)); }value }
pub(crate) fn elapsed_ms(start:u64)->u64 { let frequency:u64;unsafe { core::arch::asm!("mrs {}, cntfrq_el0",out(reg)frequency,options(nomem,nostack)); } counter().saturating_sub(start)/(frequency/1000).max(1) }

#[unsafe(no_mangle)]
pub extern "C" fn atanh(value:f64)->f64 { libm::atanh(value) }
#[unsafe(no_mangle)]
pub extern "C" fn acosh(value:f64)->f64 { libm::acosh(value) }
#[unsafe(no_mangle)]
pub extern "C" fn asinh(value:f64)->f64 { libm::asinh(value) }
#[unsafe(no_mangle)]
pub extern "C" fn lrint(value:f64)->isize { libm::rint(value) as isize }

#[unsafe(no_mangle)]
pub unsafe extern "C" fn pjb_main(_image:efi::Handle,table:*mut efi::SystemTable)->c_int {
    if table.is_null() { return 1; }
    // run() drops the VM and every output buffer before a child can exit Boot
    // Services. The strings below stay alive for the child's LoadOptions.
    let mut child_status=None;
    loop { match run(unsafe { &*table },child_status) {
        Ok(Some(request))=>{
            if request.path.contains('\0')||request.options.contains('\0') { return 1; }
            let path:Vec<u16>=request.path.encode_utf16().chain(Some(0)).collect();
            let options:Vec<u16>=request.options.encode_utf16().chain(Some(0)).collect();
            let display=request.display.as_ref().map_or(ptr::null(),|d|d as *const _);
            let status=unsafe { pjb_start_entry(path.as_ptr(),options.as_ptr(),display) };
            log(&format!("PJB_CHILD_RETURN status=0x{status:x}"));
            child_status=Some(status);
        },
        Ok(None)=>{ log("PJB_UEFI_EXIT_OK");return 0 },Err(error)=>{ log(&error);return 1 }
    } }
}
