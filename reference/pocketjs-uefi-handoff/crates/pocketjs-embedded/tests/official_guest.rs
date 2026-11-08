//! Native full-path tests. The dev dependency supplies QuickJS-NG C symbols;
//! production BSPs supply that library themselves, without rquickjs or std.
extern crate rquickjs_sys;
use pocketjs_embedded::{
    Contact, Error, InputFrameBuilder, PocketJs, TextEvent, output::PixelFormat,
};
mod support;
use support::config;

fn software(format: PixelFormat) -> (PocketJs, support::Readback) {
    use pocketjs_embedded::output::{FramebufferDisplay, RendererId};
    let config = config();
    let mut vm = PocketJs::new(config).unwrap();
    let readback = support::Readback::default();
    let stride = format.row_bytes(config.width).unwrap();
    vm.add_display(
        support::CaptureDisplay {
            framebuffer: FramebufferDisplay::new(
                vec![0; stride * config.height as usize],
                config.width,
                config.height,
                stride,
                format,
            )
            .unwrap(),
            readback: readback.clone(),
        },
        RendererId::SOFTWARE,
    )
    .unwrap();
    (vm, readback)
}
const RED: &str = r#"
    globalThis.secret = 'old-page';
    ui.setProp(1, 64, 0xff0000ff);
    globalThis.box = ui.createNode(0);
    ui.setProp(box, 24, 1); ui.setProp(box, 28, 10); ui.setProp(box, 25, 10);
    ui.setProp(box, 1, 20); ui.setProp(box, 2, 20); ui.setProp(box, 64, 0xff00ff00);
    ui.insertBefore(1, box, 0);
    globalThis.frame = () => ui.svcSend('old-tick');
"#;
fn message(vm: &mut PocketJs, generation: pocketjs_embedded::UserGeneration) -> String {
    String::from_utf8(vm.receive(generation).unwrap().unwrap()).unwrap()
}

#[test]
fn official_guest_renders_mcu_frame_and_replaces_real_user_state_64_times() {
    let (mut vm, readback) = software(PixelFormat::Argb8888);
    let engine = vm.engine_handle();
    let runtime = vm.runtime_handle();
    for _ in 0..32 {
        let old = vm.replace_user(&[RED]).unwrap();
        vm.present(old).unwrap();
        let pixels = readback.borrow();
        assert_eq!(pixels.len(), 240 * 240 * 4);
        assert_eq!(&pixels[..4], &[0, 0, 255, 255]);
        let offset = (15 * 240 + 15) * 4;
        assert_eq!(&pixels[offset..offset + 4], &[0, 255, 0, 255]);
        drop(pixels);
        vm.send(old, "old-incoming").unwrap();
        vm.tick(old, &InputFrameBuilder::new().frame()).unwrap();
        let new = vm.replace_user(&[r#"
            if (typeof secret !== 'undefined' || typeof box !== 'undefined') throw Error('old global');
            if (ui.svcPoll() !== undefined) throw Error('old incoming');
            globalThis.frame = () => ui.svcSend('new-tick');
        "#]).unwrap();
        assert!(new.get() > old.get());
        assert!(matches!(vm.present(old), Err(Error::StaleGeneration)));
        assert_eq!(vm.receive(new).unwrap(), None); // no old outgoing
        vm.present(new).unwrap();
        let pixels = readback.borrow();
        assert!(pixels.chunks_exact(4).all(|p| p == [0, 0, 0, 255])); // old UI gone
        drop(pixels);
        vm.tick(new, &InputFrameBuilder::new().frame()).unwrap();
        assert_eq!(message(&mut vm, new), "new-tick"); // no old frame callback
        assert_eq!(vm.receive(new).unwrap(), None);
        assert_eq!(vm.engine_handle(), engine);
        assert_eq!(vm.runtime_handle(), runtime);
    }
}

#[test]
fn rgb565_contacts_keyboard_buttons_and_host_parameters_use_official_wire() {
    let (mut vm, readback) = software(PixelFormat::Rgb565);
    let page = vm.replace_user(&[RED, r#"
        globalThis.frame = (buttons, analog, touches, hits, surfaces, right, relative, motion, text) => {
            if (buttons !== 17 || hits[0] !== box || touches.length !== 1 || surfaces[0] !== 0)
                throw Error('input facts');
            if (text.length !== 1) throw Error('keyboard');
            const parameters = JSON.parse(ui.svcPoll());
            if (parameters.runtime !== 'retained' || parameters.outputs[0].width !== 240)
                throw Error('parameters');
            ui.svcSend(JSON.stringify({ buttons, text, parameters }));
        };
    "#]).unwrap();
    vm.present(page).unwrap();
    let pixels = readback.borrow();
    assert_eq!(pixels.len(), 240 * 240 * 2);
    assert_eq!(&pixels[..2], &0xf800u16.to_le_bytes());
    assert_eq!(
        &pixels[(15 * 240 + 15) * 2..][..2],
        &0x07e0u16.to_le_bytes()
    );
    drop(pixels);
    vm.send(
        page,
        r#"{"runtime":"retained","outputs":[{"width":240,"height":240,"position":[0,0]}]}"#,
    )
    .unwrap();
    let mut input = InputFrameBuilder::new();
    input.buttons = 17;
    input.contacts.push(Contact {
        surface: 0,
        id: 1000,
        x: 15.0,
        y: 15.0,
        cancelled: false,
    });
    input.text.push(TextEvent::Commit("中".into()));
    vm.tick(page, &input.frame()).unwrap();
    let response = message(&mut vm, page);
    assert!(response.contains('中'));
    assert!(response.contains("retained"));
}

#[test]
fn failed_script_pending_jobs_prevent_reusing_the_old_realm() {
    let mut vm = PocketJs::new(config()).unwrap();
    let page = vm.replace_user(&[]).unwrap();
    assert!(matches!(
        vm.evaluate(
            page,
            "Promise.resolve().then(() => ui.svcSend('old-job')); throw Error('stop');"
        ),
        Err(Error::Script(_))
    ));
    assert_eq!(vm.replace_user(&[]), Err(Error::PendingJobs));
    assert_eq!(vm.replace_user(&[]), Err(Error::Poisoned));
    // Drop invalidates guest callbacks, then frees runtime-owned pending jobs.
}

#[test]
fn vm_interrupt_budget_stops_infinite_script_and_job_chain() {
    for script in [
        "while (true) {}",
        "function again() { Promise.resolve().then(again); } again();",
    ] {
        let mut vm = PocketJs::new(config()).unwrap();
        let page = vm.replace_user(&[]).unwrap();
        assert_eq!(vm.evaluate(page, script), Err(Error::BudgetExceeded));
        assert!(matches!(vm.present(page), Err(Error::Poisoned)));
    }
}

#[test]
fn failure_cleanup_and_message_bounds_allow_clean_replacement() {
    let mut vm = PocketJs::new(config()).unwrap();
    assert!(matches!(
        vm.replace_user(&["throw Error('failed load')"]),
        Err(Error::Script(_))
    ));
    let page = vm.replace_user(&[]).unwrap();
    assert_eq!(vm.send(page, &"x".repeat(4097)), Err(Error::QueueFull));
    for _ in 0..16 {
        vm.send(page, "queued").unwrap();
    }
    assert_eq!(vm.send(page, "overflow"), Err(Error::QueueFull));
    vm.unload_user().unwrap();
    assert!(matches!(vm.present(page), Err(Error::NoUser)));
    let next = vm
        .replace_user(&["if (ui.svcPoll() !== undefined) throw Error('not empty');"])
        .unwrap();
    assert!(next.get() > page.get());
}

#[test]
fn debug_is_opt_in_runtime_state_and_survives_user_context_replacement() {
    let mut vm = PocketJs::new(config()).unwrap();
    assert!(!vm.debug_channel().enabled());
    let old = vm.replace_user(&[RED]).unwrap();
    assert!(vm.debug_channel().session().is_none());
    vm.debug_channel().set_enabled(true).unwrap();
    let session = vm.debug_channel().session().unwrap();
    let engine = vm.engine_handle();
    vm.debug_channel().send(session, &[0, 255, 8]).unwrap();
    vm.debug_channel().deliver(session, b"opaque data").unwrap();
    let next = vm.replace_user(&[]).unwrap();
    assert!(next.get() > old.get());
    assert_eq!(vm.engine_handle(), engine);
    assert_eq!(vm.debug_channel().session(), Some(session));
    assert_eq!(
        vm.debug_channel().take_outgoing(session).unwrap().unwrap(),
        [0, 255, 8]
    );
    assert_eq!(
        vm.debug_channel().receive(session).unwrap().unwrap(),
        b"opaque data"
    );
    vm.debug_channel().set_enabled(false).unwrap();
    assert!(vm.debug_channel().session().is_none());
    assert!(!PocketJs::new(config()).unwrap().debug_channel().enabled());
}
