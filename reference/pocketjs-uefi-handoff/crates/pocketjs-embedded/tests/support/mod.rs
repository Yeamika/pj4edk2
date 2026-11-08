use pocketjs_embedded::{Config, output::*};
use std::{cell::RefCell, rc::Rc};

pub type Readback = Rc<RefCell<Vec<u8>>>;
pub fn config() -> Config {
    Config {
        width: 240,
        height: 240,
        tick_hz: 30,
        js_heap_bytes: 2 * 1024 * 1024,
        js_stack_bytes: 256 * 1024,
        interrupt_polls: 64,
        frame_buffer_budget: 240 * 240 * 4,
    }
}

pub struct CaptureDisplay {
    pub framebuffer: FramebufferDisplay<Vec<u8>>,
    pub readback: Readback,
}
impl DisplayOutput for CaptureDisplay {
    type Token = ();
    fn descriptor(&self) -> DisplayDescriptor {
        self.framebuffer.descriptor()
    }
    fn acquire(&mut self, region: Rect) -> Result<Target<'_>, DisplayError> {
        self.framebuffer.acquire(region)
    }
    fn flush(&mut self, regions: &[Rect]) -> Result<(), DisplayError> {
        self.framebuffer.flush(regions)?;
        *self.readback.borrow_mut() = self.framebuffer.pixels().to_vec();
        Ok(())
    }
    fn wait(&mut self, token: ()) -> Result<(), DisplayError> {
        self.framebuffer.wait(token)
    }
    fn vsync(&mut self) -> VsyncSource {
        VsyncSource::None
    }
}
