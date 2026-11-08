//! Integration through real JS/UI with a strided CPU fb and an instrumented
//! asynchronous accelerator. The latter tests the driver contract, not a GPU.
extern crate rquickjs_sys;
use pocketjs_embedded_runtime::{Error, InputFrameBuilder, PocketJs, output::*};
use std::{cell::RefCell, rc::Rc};
mod support;
use support::{CaptureDisplay, config};

#[test]
fn strided_32_bit_framebuffers_preserve_padding_and_can_present_separately() {
    for (format, red) in [
        (PixelFormat::Rgba8888, [255, 0, 0, 255]),
        (PixelFormat::Argb8888, [0, 0, 255, 255]),
    ] {
        let mut config = config();
        config.width = 13;
        config.height = 7;
        let stride = 64;
        config.frame_buffer_budget = stride * config.height as usize;
        let readback = support::Readback::default();
        let mut vm = PocketJs::new(config).unwrap();
        vm.add_display(
            CaptureDisplay {
                framebuffer: FramebufferDisplay::new(
                    vec![0xa5; config.frame_buffer_budget],
                    13,
                    7,
                    stride,
                    format,
                )
                .unwrap(),
                readback: readback.clone(),
            },
            RendererId::SOFTWARE,
        )
        .unwrap();
        let page = vm
            .replace_user(&["ui.setProp(1, 64, 0xff0000ff);"])
            .unwrap();
        assert!(readback.borrow().is_empty());
        vm.advance(page, &InputFrameBuilder::new().frame()).unwrap();
        assert!(readback.borrow().is_empty());
        vm.present(page).unwrap();
        let bytes = readback.borrow();
        for row in bytes.chunks_exact(stride) {
            assert!(row[..52].chunks_exact(4).all(|pixel| pixel == red));
            assert!(row[52..].iter().all(|byte| *byte == 0xa5));
        }
    }
}

struct Pending {
    id: u64,
    expected: Vec<u8>,
    borrowed: Option<(*const u8, usize)>,
    stopped: bool,
}
#[derive(Default)]
struct Device {
    events: Vec<&'static str>,
    next: u64,
    pending: Option<Pending>,
    texture: Option<i32>,
    fail_wait: bool,
}
type Shared = Rc<RefCell<Device>>;

struct Accelerator(Shared);
impl Accelerator {
    fn submit(&mut self, frame: &mut FrameInput<'_>, zero_copy: bool) -> Result<u64, RenderError> {
        let Target::DeviceOwned(target) = &frame.target else {
            return Err(RenderError::Unsupported);
        };
        if target.handle != 42 || target.format != PixelFormat::Rgba8888 {
            return Err(RenderError::InvalidFrame);
        }
        assert!(!frame.list.words.is_empty()); // actual UI DrawList reached driver
        assert_eq!(frame.resources.viewport(), (240.0, 240.0));
        assert_eq!(target.region, Rect::new(0, 0, 1600, 900)); // hardware maps viewport
        let mut device = self.0.borrow_mut();
        assert!(device.pending.is_none());
        let texture = device.texture.and_then(|id| frame.resources.texture(id));
        device.next += 1;
        let id = device.next;
        device.pending = Some(Pending {
            id,
            expected: texture.map_or_else(Vec::new, |t| t.pixels.to_vec()),
            borrowed: texture
                .filter(|_| zero_copy)
                .map(|t| (t.pixels.as_ptr(), t.pixels.len())),
            stopped: false,
        });
        device.events.push(if zero_copy {
            "render-zero-copy"
        } else {
            "render-copy"
        });
        Ok(id)
    }
}
impl Renderer for Accelerator {
    type Submission = u64;
    fn descriptor(&self) -> RendererDescriptor {
        RendererDescriptor::new(
            &[PixelFormat::Rgba8888],
            TargetOwnership::DeviceOwned,
            Synchronization::Deferred,
            pocket_spec::DRAW_LIST_VERSION,
            true,
        )
        .unwrap()
    }
    fn render(&mut self, frame: &mut FrameInput<'_>) -> Result<u64, RenderError> {
        self.submit(frame, false)
    }
    unsafe fn render_zero_copy(&mut self, frame: &mut FrameInput<'_>) -> Result<u64, RenderError> {
        self.submit(frame, true)
    }
    fn wait(&mut self, id: &u64) -> Result<(), RenderError> {
        let mut device = self.0.borrow_mut();
        let pending = device
            .pending
            .as_mut()
            .filter(|p| p.id == *id)
            .ok_or(RenderError::InvalidFrame)?;
        if let Some((pointer, length)) = pending.borrowed {
            // The unsafe render_zero_copy contract keeps this allocation live
            // and unchanged until retirement, even after guest freeTexture.
            assert_eq!(
                unsafe { std::slice::from_raw_parts(pointer, length) },
                pending.expected
            );
        }
        pending.stopped = true;
        device.events.push("render-wait");
        if device.fail_wait {
            device.fail_wait = false;
            Err(RenderError::Failed)
        } else {
            Ok(())
        }
    }
    fn retire(&mut self, id: u64) {
        let mut device = self.0.borrow_mut();
        if device
            .pending
            .as_ref()
            .is_some_and(|p| p.id == id && p.stopped)
        {
            device.pending = None;
            device.events.push("retire");
        }
    }
}
impl Drop for Accelerator {
    fn drop(&mut self) {
        assert!(self.0.borrow().pending.is_none());
    }
}

struct Scanout {
    device: Shared,
    queued: bool,
}
impl DisplayOutput for Scanout {
    type Token = ();
    fn descriptor(&self) -> DisplayDescriptor {
        let mut descriptor = DisplayDescriptor::new(
            DisplayRole::Primary,
            1600,
            900,
            PixelFormat::Rgba8888,
            1600 * 4,
            TargetOwnership::DeviceOwned,
        );
        descriptor.queued_flush = self.queued;
        descriptor
    }
    fn acquire(&mut self, region: Rect) -> Result<Target<'_>, DisplayError> {
        if region != Rect::new(0, 0, 1600, 900) {
            return Err(DisplayError::InvalidTarget);
        }
        self.device.borrow_mut().events.push("acquire");
        Ok(Target::DeviceOwned(DeviceTarget::new(
            42,
            region,
            PixelFormat::Rgba8888,
        )))
    }
    fn flush(&mut self, _: &[Rect]) -> Result<(), DisplayError> {
        self.device.borrow_mut().events.push("flush");
        Ok(())
    }
    fn wait(&mut self, _: ()) -> Result<(), DisplayError> {
        self.device.borrow_mut().events.push("display-wait");
        Ok(())
    }
    fn vsync(&mut self) -> VsyncSource {
        VsyncSource::None
    }
}
fn accelerated(queued: bool) -> (PocketJs, Shared) {
    let mut config = config();
    config.frame_buffer_budget = 0; // no CPU pixels allocated or reserved
    let mut vm = PocketJs::new(config).unwrap();
    let device = Shared::default();
    let renderer = vm.add_renderer(Accelerator(device.clone())).unwrap();
    vm.add_display(
        Scanout {
            device: device.clone(),
            queued,
        },
        renderer,
    )
    .unwrap();
    (vm, device)
}

const TEXTURE: &str = r#"
    ui.setProp(1, 64, 0xff0000ff);
    globalThis.marker = 'old';
    globalThis.tex = ui.uploadTexture(new Uint8Array([1, 2, 3, 255]), 1, 1, 3);
    ui.svcSend(String(tex));
    globalThis.frame = () => ui.svcSend('guest-ran');
"#;

#[test]
fn deferred_accelerator_keeps_textures_and_drains_before_user_replacement_or_drop() {
    for queued in [false, true] {
        let (mut vm, device) = accelerated(queued);
        let engine = vm.engine_handle();
        let page = vm.replace_user(&[TEXTURE]).unwrap();
        device.borrow_mut().texture = Some(
            String::from_utf8(vm.receive(page).unwrap().unwrap())
                .unwrap()
                .parse()
                .unwrap(),
        );
        vm.present(page).unwrap();
        assert!(device.borrow().pending.is_some());
        assert_eq!(
            device.borrow().pending.as_ref().unwrap().expected,
            [1, 2, 3, 255]
        );
        vm.advance(page, &InputFrameBuilder::new().frame()).unwrap();
        assert_eq!(vm.receive(page).unwrap().unwrap(), b"guest-ran");
        assert!(device.borrow().pending.is_some());
        vm.evaluate(page, "ui.freeTexture(tex);").unwrap();
        let next = vm.replace_user(&["if (typeof marker !== 'undefined') throw Error('old context'); ui.setProp(1, 64, 0xff00ff00);"]).unwrap();
        assert_eq!(vm.engine_handle(), engine);
        assert!(device.borrow().pending.is_none());
        let expected = if queued {
            vec![
                "acquire",
                "render-zero-copy",
                "flush",
                "render-wait",
                "display-wait",
                "retire",
            ]
        } else {
            vec![
                "acquire",
                "render-zero-copy",
                "render-wait",
                "flush",
                "display-wait",
                "retire",
            ]
        };
        assert_eq!(device.borrow().events, expected);
        device.borrow_mut().texture = None;
        vm.present(next).unwrap();
        assert!(device.borrow().pending.is_some());
        drop(vm);
        assert!(device.borrow().pending.is_none());
        assert_eq!(device.borrow().events.last(), Some(&"retire"));
    }
}

#[test]
fn renderer_failure_is_terminal_and_prevents_releasing_page_before_error_is_reported() {
    let (mut vm, device) = accelerated(false);
    let page = vm
        .replace_user(&["globalThis.marker = 'old'; ui.setProp(1, 64, 0xff0000ff);"])
        .unwrap();
    vm.present(page).unwrap();
    device.borrow_mut().fail_wait = true;
    assert!(matches!(
        vm.replace_user(&[]),
        Err(Error::Presentation(PresentError::RenderWait(
            _,
            RenderError::Failed
        )))
    ));
    assert!(device.borrow().pending.is_none());
    vm.evaluate(
        page,
        "if (marker !== 'old') throw Error('premature teardown');",
    )
    .unwrap();
    assert!(vm.replace_user(&[]).unwrap().get() > page.get());
}

#[test]
fn mismatched_targets_and_cpu_budget_are_rejected_at_registration() {
    let mut config = config();
    config.frame_buffer_budget = 0;
    let mut vm = PocketJs::new(config).unwrap();
    let device = Shared::default();
    assert!(matches!(
        vm.add_display(
            Scanout {
                device,
                queued: false
            },
            RendererId::SOFTWARE
        ),
        Err(Error::Presentation(PresentError::IncompatibleRenderer))
    ));
    let display = FramebufferDisplay::new(
        vec![0; 240 * 240 * 4],
        240,
        240,
        240 * 4,
        PixelFormat::Argb8888,
    )
    .unwrap();
    assert!(matches!(
        vm.add_display(display, RendererId::SOFTWARE),
        Err(Error::Presentation(PresentError::BudgetExceeded))
    ));
}
