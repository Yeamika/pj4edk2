//! Upstream rendering/output contracts for software fb, GPU, blitter or DMA.
//! Formats and storage belong to drivers; the VM allocates no pixel buffer.
use pocket_present::Coordinator;
use pocketjs_core::Ui;

pub mod framebuffer;
pub use framebuffer::FramebufferDisplay;
pub use pocket_drivers::{
    BufferIdentity, DeviceTarget, DisplayDescriptor, DisplayError, DisplayOutput, DisplayRole,
    DrawList, FrameInput, PixelFormat, Rect, RenderError, RenderResources, Renderer,
    RendererDescriptor, Surface, Synchronization, Target, TargetOwnership, VsyncSource,
};
pub use pocket_present::{DisplayId, PresentError, RendererId};
pub use pocket_raster::{RasterRenderer, render_operation};

pub(crate) struct Presentation(Coordinator);

impl Presentation {
    pub fn new(budget: usize) -> Result<Self, PresentError> {
        let mut coordinator = Coordinator::new(1, budget)?;
        // Every present below pins the Ui's referenced textures before its
        // borrow ends. Completion and page replacement release only retired
        // versions. No caller receives direct access to this coordinator.
        unsafe { coordinator.enable_deferred_zero_copy() };
        Ok(Self(coordinator))
    }

    pub fn add_renderer<R: Renderer + 'static>(
        &mut self,
        renderer: R,
    ) -> Result<RendererId, PresentError>
    where
        R::Submission: 'static,
    {
        self.0.add_renderer(renderer)
    }
    pub fn add_display<D: DisplayOutput + 'static>(
        &mut self,
        display: D,
        renderer: RendererId,
    ) -> Result<DisplayId, PresentError>
    where
        D::Token: 'static,
    {
        self.0.add_display(display, renderer)
    }
    pub fn present(&mut self, ui: &mut Ui) -> Result<(), PresentError> {
        ui.draw();
        let result = self.0.present(ui.current_draw_list(), None, &pj4edk2_rendering::PixelResources(&*ui));
        self.track_textures(ui);
        result
    }
    pub fn complete(&mut self, ui: &mut Ui) -> Result<(), PresentError> {
        let result = self.0.complete();
        self.track_textures(ui);
        result
    }
    pub fn invalidate(&mut self) {
        self.0.invalidate();
    }

    // Same texture ownership protocol as the official runtime. In-flight GPU
    // reads survive guest freeTexture/overwrites; page teardown waits first.
    fn track_textures(&self, ui: &mut Ui) {
        unsafe { ui.release_textures(self.0.retired_frame()) };
        if let Some((frame, handles)) = self.0.borrowed_textures() {
            ui.retain_textures(frame, handles);
        }
    }
}
