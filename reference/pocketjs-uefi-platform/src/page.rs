//! The default boot pages on the official no_std guest, without a desktop VM.
use alloc::{format, string::String, vec::Vec};
use pocketjs_boot_ui::{NameCardSpec, UiAction, UiConfig, UiEvent};
use pocketjs_core::{Ui, spec};
use pocketjs_embedded::{Error, InputFrameBuilder, PocketJs, UserGeneration};

/// Bake names once using the same UI/raster API as the desktop. Synchronous JS
/// name requests subsequently retrieve these prepared textures over a JS map.
fn prepare(ui: &mut Ui, config: &UiConfig, names: &mut serde_json::Map<String, serde_json::Value>) -> Result<(), Error> {
    for atlas in pocketjs_boot_assets::atlases(false) { if !ui.load_font_atlas(atlas) { return Err(Error::Runtime); } }
    for entry in &config.entries {
        let width = ui.measure_text(&entry.name, 6) + 40.;
        let width = width.clamp(184., 340.) as u32;
        let card = NameCardSpec { text: entry.name.clone(), size: [width, 48],
            radius: 8, fill: 0xff_ff_fc_f7, border: 0xff_af_7e_32,
            ink: 0xff_69_4d_21, font_slot: 6, text_origin: [20, 8] };
        let image = pocketjs_boot_assets::bake(&card, false).map_err(|_| Error::Runtime)?;
        let (pixels, w, h) = (image.pixels, image.width, image.height);
        let handle = ui.upload_texture(&pixels, w, h, spec::psm::PSM_8888);
        if handle < 0 { return Err(Error::OutOfMemory); }
        names.insert(entry.name.clone(), serde_json::json!({"handle":handle,"width":w,"height":h}));
    }
    Ok(())
}

pub struct BootPage {
    pub generation: UserGeneration,
    #[cfg(target_os="none")]
    pub timing:core::cell::Cell<[u64;3]>,
}
impl BootPage {
    pub fn load(vm: &mut PocketJs, config: &UiConfig) -> Result<Self, Error> {
        config.validate().map_err(|_| Error::InvalidConfig)?;
        let mut names = serde_json::Map::new();
        let generation = vm.replace_user_with(&[], |ui| prepare(ui, config, &mut names))?;
        let shim = format!(r#"globalThis.bootEmit = text => ui.svcSend(text);
            globalThis.bootNameImages = {};
            globalThis.bootBakeName = text => {{
                const request = JSON.parse(text), value = bootNameImages[request.text];
                if (!value) throw Error('unprepared boot name');
                return JSON.stringify(value);
            }};
            globalThis.bootBatteryEstimate = false;
            globalThis.frame = () => {{}};"#, serde_json::Value::Object(names));
        vm.evaluate(generation, &shim)?;
        for script in pocketjs_boot_ui::scripts() { vm.evaluate(generation, script.source)?; }
        let config = config.to_json().map_err(|_| Error::InvalidConfig)?;
        let quoted = serde_json::to_string(&config).map_err(|_| Error::InvalidConfig)?;
        vm.evaluate(generation, &format!("rimuksInit({quoted});"))?;
        Ok(Self { generation, #[cfg(target_os="none")] timing:core::cell::Cell::new([0;3]) })
    }

    pub fn set_battery_estimated(&self,vm:&mut PocketJs,estimated:bool)->Result<(),Error> {
        vm.evaluate(self.generation,if estimated {"globalThis.bootBatteryEstimate=true;"} else {"globalThis.bootBatteryEstimate=false;"})
    }

    pub fn tick(&self, vm: &mut PocketJs, now_ms: u64, events: &[UiEvent]) -> Result<Vec<UiAction>, Error> {
        let events = serde_json::to_string(events).map_err(|_| Error::InvalidInput)?;
        let quoted = serde_json::to_string(&events).map_err(|_| Error::InvalidInput)?;
        #[cfg(target_os="none")] let t0=crate::firmware::counter();
        vm.evaluate(self.generation, &format!("rimuksTick({now_ms},{quoted});"))?;
        #[cfg(target_os="none")] let eval=crate::firmware::elapsed_ms(t0);
        #[cfg(target_os="none")] let t1=crate::firmware::counter();
        vm.advance(self.generation, &InputFrameBuilder::new().frame())?;
        #[cfg(target_os="none")] let advance=crate::firmware::elapsed_ms(t1);
        #[cfg(target_os="none")] let t2=crate::firmware::counter();
        vm.present(self.generation)?;
        #[cfg(target_os="none")] self.timing.set([eval,advance,crate::firmware::elapsed_ms(t2)]);
        let mut actions = Vec::new();
        while let Some(bytes) = vm.receive(self.generation)? {
            actions.push(serde_json::from_slice(&bytes).map_err(|_| Error::Runtime)?);
        }
        Ok(actions)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pocketjs_embedded::{Config, output::*};
    // Force the dev-only QuickJS library into the final native test executable.
    use rquickjs_sys as _;
    #[test]
    fn failed_launch_returns_to_a_manual_menu_and_can_retry() {
        let (width,height)=(1280,800);
        let mut vm=PocketJs::new(Config{width,height,tick_hz:30,js_heap_bytes:16*1024*1024,
            js_stack_bytes:512*1024,interrupt_polls:100_000,frame_buffer_budget:width as usize*height as usize*4}).unwrap();
        vm.add_display(FramebufferDisplay::new(alloc::vec![0;width as usize*height as usize*4],
            width,height,width as usize*4,PixelFormat::Argb8888).unwrap(),RendererId::SOFTWARE).unwrap();
        let mut config=pocketjs_boot_ui::default_config();config.viewport=[width,height];config.timeout_seconds=3;
        // Deliver the host's failure event to a new menu, even if persistent
        // settings would otherwise enable auto-start on that menu.
        let page=BootPage::load(&mut vm,&config).unwrap();
        assert!(page.tick(&mut vm,0,&[UiEvent::BootFailed{status:"0x800000000000000e".into()}]).unwrap().is_empty());
        for now in [3000,6000,9000] {assert!(page.tick(&mut vm,now,&[]).unwrap().is_empty());}
        vm.evaluate(page.generation,r#"const returned=JSON.parse(rimuksState());
            if(returned.auto_active||returned.booting) throw Error('failed launch repeated');
            if(returned.timeout_seconds!==3) throw Error('failure modified configured timeout');
            if(!Rimuks.bootError.includes('0x800000000000000e')) throw Error('missing boot error');"#).unwrap();
        assert!(page.tick(&mut vm,9016,&[UiEvent::Key{key:"F2".into()}]).unwrap().is_empty());
        vm.evaluate(page.generation,"if(!JSON.parse(rimuksState()).settings_open) throw Error('menu input stopped');").unwrap();
        page.tick(&mut vm,9032,&[UiEvent::Key{key:"F2".into()}]).unwrap();
        // The closing overlay consumes input until its fade finishes.
        for now in (9048..=10008).step_by(16) {page.tick(&mut vm,now,&[]).unwrap();}
        page.tick(&mut vm,10024,&[UiEvent::Key{key:"Enter".into()}]).unwrap();
        page.tick(&mut vm,10040,&[UiEvent::Key{key:"Enter".into()}]).unwrap();
        assert!(matches!(page.tick(&mut vm,12000,&[]).unwrap().as_slice(),[UiAction::Boot{..}]));
        assert!(page.tick(&mut vm,13000,&[]).unwrap().is_empty());
    }
    #[test]
    fn live_battery_readings_repaint_only_a_small_area() {
        use std::{cell::Cell,rc::Rc};
        struct Capture {fb:FramebufferDisplay<Vec<u8>>,area:Rc<Cell<u64>>}
        impl DisplayOutput for Capture {
            type Token=();
            fn descriptor(&self)->DisplayDescriptor {let mut d=self.fb.descriptor();d.partial_refresh=true;d}
            fn frame_identity(&mut self)->Result<Option<BufferIdentity>,DisplayError> {Ok(Some(BufferIdentity{buffer:0,generation:0}))}
            fn acquire(&mut self,r:Rect)->Result<Target<'_>,DisplayError>{self.fb.acquire(r)}
            fn flush(&mut self,rs:&[Rect])->Result<(),DisplayError>{
                self.area.set(self.area.get()+rs.iter().map(|r|u64::from(r.width)*u64::from(r.height)).sum::<u64>());Ok(())
            }
            fn wait(&mut self,_:())->Result<(),DisplayError>{Ok(())}
            fn vsync(&mut self)->VsyncSource{VsyncSource::None}
        }
        let (width,height)=(1920,1200);
        let area=Rc::new(Cell::new(0));
        let mut vm=PocketJs::new(Config{width,height,tick_hz:30,js_heap_bytes:16*1024*1024,
            js_stack_bytes:512*1024,interrupt_polls:100_000,frame_buffer_budget:width as usize*height as usize*4}).unwrap();
        vm.add_display(Capture{fb:FramebufferDisplay::new(alloc::vec![0;width as usize*height as usize*4],
            width,height,width as usize*4,PixelFormat::Argb8888).unwrap(),area:area.clone()},RendererId::SOFTWARE).unwrap();
        let mut config=pocketjs_boot_ui::default_config();config.viewport=[width,height];config.timeout_seconds=0;
        let page=BootPage::load(&mut vm,&config).unwrap();
        let power=|voltage,current|UiEvent::PowerState{battery_percent:Some(72),external_power:Some(true),
            power:pocketjs_boot_ui::PowerTelemetry{charging:Some(true),voltage_mv:Some(voltage),current_ma:Some(current)}};
        page.tick(&mut vm,0,&[]).unwrap();
        page.tick(&mut vm,16,&[power(4152,1620)]).unwrap();
        let before=area.get();
        for (now,voltage,current) in [(32,4151,1619),(48,4149,1613),(64,4150,1616)] {
            page.tick(&mut vm,now,&[power(voltage,current)]).unwrap();
        }
        let repainted=area.get()-before;
        assert!(repainted>0 && repainted<u64::from(width)*u64::from(height)/4,
            "battery updates repainted {repainted} pixels");
        vm.evaluate(page.generation,"const p=JSON.parse(rimuksState());if(p.power_readings!=='4.150 V  +1616 mA'||p.power_status!=='Charging')throw Error('stale power readout');").unwrap();
    }
    #[test]
    fn blinking_settings_caret_damages_only_a_small_area() {
        use std::{cell::Cell,rc::Rc};
        struct Capture {fb:FramebufferDisplay<Vec<u8>>,area:Rc<Cell<u64>>}
        impl DisplayOutput for Capture {
            type Token=();
            fn descriptor(&self)->DisplayDescriptor {let mut d=self.fb.descriptor();d.partial_refresh=true;d}
            fn frame_identity(&mut self)->Result<Option<BufferIdentity>,DisplayError> {Ok(Some(BufferIdentity{buffer:0,generation:0}))}
            fn acquire(&mut self,r:Rect)->Result<Target<'_>,DisplayError>{self.fb.acquire(r)}
            fn flush(&mut self,rs:&[Rect])->Result<(),DisplayError>{
                self.area.set(self.area.get()+rs.iter().map(|r|u64::from(r.width)*u64::from(r.height)).sum::<u64>());Ok(())
            }
            fn wait(&mut self,_:())->Result<(),DisplayError>{Ok(())}
            fn vsync(&mut self)->VsyncSource{VsyncSource::None}
        }
        let (width,height)=(1280,800);
        let area=Rc::new(Cell::new(0));
        let mut vm=PocketJs::new(Config{width,height,tick_hz:30,js_heap_bytes:16*1024*1024,
            js_stack_bytes:512*1024,interrupt_polls:100_000,frame_buffer_budget:width as usize*height as usize*4}).unwrap();
        vm.add_display(Capture{fb:FramebufferDisplay::new(alloc::vec![0;width as usize*height as usize*4],
            width,height,width as usize*4,PixelFormat::Argb8888).unwrap(),area:area.clone()},RendererId::SOFTWARE).unwrap();
        let mut config=pocketjs_boot_ui::default_config();config.viewport=[width,height];config.timeout_seconds=0;
        let page=BootPage::load(&mut vm,&config).unwrap();
        page.tick(&mut vm,0,&[]).unwrap();
        page.tick(&mut vm,16,&[UiEvent::Key{key:"F2".into()}]).unwrap();
        for now in (32..=1024).step_by(16) {page.tick(&mut vm,now,&[]).unwrap();}
        let before=area.get();
        for now in [1100,1490,1510,1900,2010,2100] {page.tick(&mut vm,now,&[]).unwrap();}
        let repainted=area.get()-before;
        assert!(repainted<u64::from(width)*u64::from(height)*2,"caret blink repainted {repainted} pixels");
    }
    #[test]
    fn settings_shade_is_updated_only_when_its_visibility_or_opacity_changes() {
        let (width,height)=(1280,800);
        let mut vm=PocketJs::new(Config{width,height,tick_hz:30,js_heap_bytes:16*1024*1024,
            js_stack_bytes:512*1024,interrupt_polls:100_000,frame_buffer_budget:width as usize*height as usize*4}).unwrap();
        vm.add_display(FramebufferDisplay::new(alloc::vec![0;width as usize*height as usize*4],
            width,height,width as usize*4,PixelFormat::Argb8888).unwrap(),RendererId::SOFTWARE).unwrap();
        let mut config=pocketjs_boot_ui::default_config();config.viewport=[width,height];config.timeout_seconds=0;
        let page=BootPage::load(&mut vm,&config).unwrap();
        vm.evaluate(page.generation,r#"globalThis.shadeWrites=0;
            const savedProp=Rimuks.prop;
            Rimuks.prop=(id,key,value)=>{if(id===Rimuks.settingsShade) shadeWrites++;return savedProp(id,key,value);};"#).unwrap();
        page.tick(&mut vm,16,&[]).unwrap();
        vm.evaluate(page.generation,"if(shadeWrites!==0) throw Error('hidden shade was redrawn');").unwrap();
        page.tick(&mut vm,32,&[UiEvent::Key{key:"F2".into()}]).unwrap();
        for now in (48..=1008).step_by(16) {page.tick(&mut vm,now,&[]).unwrap();}
        vm.evaluate(page.generation,"if(shadeWrites===0) throw Error('opening shade did not update');shadeWrites=0;").unwrap();
        page.tick(&mut vm,1024,&[]).unwrap();
        vm.evaluate(page.generation,"if(shadeWrites!==0) throw Error('unchanged shade was redrawn');").unwrap();
        page.tick(&mut vm,1040,&[UiEvent::Key{key:"F2".into()}]).unwrap();
        vm.evaluate(page.generation,"if(shadeWrites===0) throw Error('closing shade did not update');").unwrap();
    }
    #[test]
    fn curve_is_reused_when_static_and_refreshed_for_animation_and_recreation() {
        let (width,height)=(1280,800);
        let mut vm=PocketJs::new(Config{width,height,tick_hz:30,js_heap_bytes:16*1024*1024,
            js_stack_bytes:512*1024,interrupt_polls:100_000,frame_buffer_budget:width as usize*height as usize*4}).unwrap();
        vm.add_display(FramebufferDisplay::new(alloc::vec![0;width as usize*height as usize*4],
            width,height,width as usize*4,PixelFormat::Argb8888).unwrap(),RendererId::SOFTWARE).unwrap();
        let mut config=pocketjs_boot_ui::default_config();config.viewport=[width,height];config.timeout_seconds=0;
        let page=BootPage::load(&mut vm,&config).unwrap();
        vm.evaluate(page.generation,r#"globalThis.curveWrites=0;
            globalThis.trackedLines=new Set(Rimuks.lines);
            const originalProp=Rimuks.prop;
            Rimuks.prop=(id,key,value)=>{if(trackedLines.has(id)) curveWrites++;return originalProp(id,key,value);};"#).unwrap();
        page.tick(&mut vm,16,&[]).unwrap();
        vm.evaluate(page.generation,"if(curveWrites!==0) throw Error('static curve was redrawn'); Rimuks.beginBoot();").unwrap();
        page.tick(&mut vm,350,&[]).unwrap();
        vm.evaluate(page.generation,"if(curveWrites===0) throw Error('animated curve was not updated');curveWrites=0;").unwrap();
        page.tick(&mut vm,350,&[]).unwrap();
        vm.evaluate(page.generation,r#"if(curveWrites!==0) throw Error('unchanged animation was redrawn');
            Rimuks.makeBootCurve();trackedLines=new Set(Rimuks.lines);curveWrites=0;
            Rimuks.updateBootCurve();if(curveWrites===0) throw Error('new curve was not drawn');
            curveWrites=0;Rimuks.updateBootCurve();if(curveWrites!==0) throw Error('new static curve was redrawn');"#).unwrap();
    }
    #[test]
    fn retained_damage_matches_full_frames_during_heartbeat_and_navigation() {
        use std::{cell::RefCell,rc::Rc};
        struct Capture {
            fb:FramebufferDisplay<Vec<u8>>, partial:bool,
            pixels:Rc<RefCell<Vec<u8>>>, area:Rc<RefCell<u64>>,
        }
        impl DisplayOutput for Capture {
            type Token=();
            fn descriptor(&self)->DisplayDescriptor {
                let mut d=self.fb.descriptor();d.partial_refresh=self.partial;d
            }
            fn frame_identity(&mut self)->Result<Option<BufferIdentity>,DisplayError> {
                Ok(self.partial.then_some(BufferIdentity{buffer:0,generation:0}))
            }
            fn acquire(&mut self,r:Rect)->Result<Target<'_>,DisplayError>{self.fb.acquire(r)}
            fn flush(&mut self,rs:&[Rect])->Result<(),DisplayError>{
                *self.area.borrow_mut()+=rs.iter().map(|r|u64::from(r.width)*u64::from(r.height)).sum::<u64>();
                *self.pixels.borrow_mut()=self.fb.pixels().to_vec();Ok(())
            }
            fn wait(&mut self,_:())->Result<(),DisplayError>{Ok(())}
            fn vsync(&mut self)->VsyncSource{VsyncSource::None}
        }
        let render=|partial| {
            let (width,height)=(1280,800);
            let pixels=Rc::new(RefCell::new(Vec::new()));let area=Rc::new(RefCell::new(0));
            let mut vm=PocketJs::new(Config{width,height,tick_hz:30,js_heap_bytes:16*1024*1024,
                js_stack_bytes:512*1024,interrupt_polls:100_000,frame_buffer_budget:width as usize*height as usize*4}).unwrap();
            vm.add_display(Capture{fb:FramebufferDisplay::new(alloc::vec![0;width as usize*height as usize*4],width,height,width as usize*4,PixelFormat::Argb8888).unwrap(),
                partial,pixels:pixels.clone(),area:area.clone()},RendererId::SOFTWARE).unwrap();
            let mut config=pocketjs_boot_ui::default_config();config.viewport=[width,height];config.timeout_seconds=0;
            let page=BootPage::load(&mut vm,&config).unwrap();
            let mut frames=Vec::new();
            let mut normal_area=0;
            for (now,events) in [(0,vec![]),(500,vec![]),(1000,vec![UiEvent::Key{key:"ArrowDown".into()}]),
                (1100,vec![]),(1500,vec![UiEvent::Key{key:"ArrowUp".into()}]),(2000,vec![]),
                (2100,vec![UiEvent::Key{key:"F2".into()}]),(2200,vec![])] {
                page.tick(&mut vm,now,&events).unwrap();frames.push(pixels.borrow().clone());
                if now==2000 {normal_area=*area.borrow();}
            }
            // Opening the settings overlay deliberately changes most pixels.
            (frames,normal_area)
        };
        let (full,full_area)=render(false);let (partial,partial_area)=render(true);
        for (a,b) in full.iter().zip(partial.iter()) { assert!(a==b,"partial presentation changed pixels"); }
        assert!(partial_area<full_area/2,"damage path did not reduce copied pixels");
    }
    #[test]
    fn official_guest_runs_portrait_default_page_and_emits_only_after_two_taps() {
        let mut vm = PocketJs::new(Config { width:720,height:1440,tick_hz:30,
            js_heap_bytes:16*1024*1024,js_stack_bytes:512*1024,interrupt_polls:100_000,
            frame_buffer_budget:720*1440*4 }).unwrap();
        vm.add_display(FramebufferDisplay::new(alloc::vec![0;720*1440*4],720,1440,720*4,PixelFormat::Argb8888).unwrap(), RendererId::SOFTWARE).unwrap();
        let mut config = pocketjs_boot_ui::default_config(); config.viewport=[720,1440];config.timeout_seconds=0;
        let page = BootPage::load(&mut vm,&config).unwrap();
        assert!(page.tick(&mut vm,0,&[]).unwrap().is_empty());
        // At portrait logical width 720, the first card is around x=100..680.
        assert!(page.tick(&mut vm,16,&[UiEvent::Click{x:300.,y:280.}]).unwrap().is_empty());
        assert!(page.tick(&mut vm,32,&[UiEvent::Click{x:300.,y:280.}]).unwrap().is_empty());
        let actions=page.tick(&mut vm,1200,&[]).unwrap();
        assert!(matches!(actions.as_slice(),[UiAction::Boot{id,..}] if id=="rimuks"));
        assert!(page.tick(&mut vm,1300,&[]).unwrap().is_empty());
    }

    #[test]
    fn panel_touch_confirms_landscape_entry_on_the_official_guest() {
        let (width,height)=(2160,1080);
        let mut vm=PocketJs::new(Config{width,height,tick_hz:30,
            js_heap_bytes:16*1024*1024,js_stack_bytes:512*1024,interrupt_polls:100_000,
            frame_buffer_budget:width as usize*height as usize*4}).unwrap();
        vm.add_display(FramebufferDisplay::new(alloc::vec![0;width as usize*height as usize*4],
            width,height,width as usize*4,PixelFormat::Argb8888).unwrap(),RendererId::SOFTWARE).unwrap();
        let mut config=pocketjs_boot_ui::default_config();config.viewport=[width,height];config.timeout_seconds=0;
        let page=BootPage::load(&mut vm,&config).unwrap();
        assert!(page.tick(&mut vm,0,&[]).unwrap().is_empty());
        let [x,y]=crate::rotation::touch(1080,[659.,900.]);
        assert!(page.tick(&mut vm,16,&[UiEvent::Click{x,y}]).unwrap().is_empty());
        assert!(page.tick(&mut vm,32,&[UiEvent::Click{x,y}]).unwrap().is_empty());
        assert!(matches!(page.tick(&mut vm,1200,&[]).unwrap().as_slice(),[UiAction::Boot{id,..}] if id=="rimuks"));
        assert!(page.tick(&mut vm,1300,&[]).unwrap().is_empty());
    }
}
