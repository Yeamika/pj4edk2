//! An optional coherent CPU framebuffer. This may wrap existing mapped memory
//! or a BSP-owned Vec/Box; it does not allocate or create a display device.
use super::{
    DisplayDescriptor, DisplayError, DisplayOutput, DisplayRole, PixelFormat, Rect, Surface,
    Target, TargetOwnership, VsyncSource,
};

pub struct FramebufferDisplay<B> {
    buffer: B,
    descriptor: DisplayDescriptor,
}

impl<B: AsRef<[u8]> + AsMut<[u8]>> FramebufferDisplay<B> {
    /// A single, coherent, CPU-writable buffer. After registration, its storage
    /// is owned exclusively by this display. MMIO mapping, cache attributes and
    /// scanout setup are BSP responsibilities; devices needing flush/DMA/cache
    /// operations implement DisplayOutput themselves or wrap this driver.
    ///
    /// Byte-addressable formats are accepted. The registered renderer must
    /// support the selected format; the stock raster supports RGBA, ARGB, RGB565.
    pub fn new(
        buffer: B,
        width: u32,
        height: u32,
        stride_bytes: usize,
        format: PixelFormat,
    ) -> Result<Self, DisplayError> {
        if !format.bits_per_pixel().is_multiple_of(8) {
            return Err(DisplayError::Unsupported);
        }
        let row = format.row_bytes(width).ok_or(DisplayError::InvalidTarget)?;
        let needed = stride_bytes
            .checked_mul(height.saturating_sub(1) as usize)
            .and_then(|n| n.checked_add(row))
            .ok_or(DisplayError::InvalidTarget)?;
        if width == 0 || height == 0 || stride_bytes < row || buffer.as_ref().len() < needed {
            return Err(DisplayError::InvalidTarget);
        }
        Ok(Self {
            buffer,
            descriptor: DisplayDescriptor::new(
                DisplayRole::Primary,
                width,
                height,
                format,
                stride_bytes,
                TargetOwnership::CpuVisible,
            ),
        })
    }

    /// Backing bytes, including any row padding. Useful to a BSP flush wrapper.
    pub fn pixels(&self) -> &[u8] {
        self.buffer.as_ref()
    }
}

impl<B: AsRef<[u8]> + AsMut<[u8]>> DisplayOutput for FramebufferDisplay<B> {
    type Token = ();
    fn descriptor(&self) -> DisplayDescriptor {
        self.descriptor
    }
    fn acquire(&mut self, region: Rect) -> Result<Target<'_>, DisplayError> {
        if region.width == 0
            || region.height == 0
            || region
                .x
                .checked_add(region.width)
                .is_none_or(|x| x > self.descriptor.width)
            || region
                .y
                .checked_add(region.height)
                .is_none_or(|y| y > self.descriptor.height)
        {
            return Err(DisplayError::InvalidTarget);
        }
        let offset = (region.y as usize)
            .checked_mul(self.descriptor.stride_bytes)
            .and_then(|n| {
                n.checked_add(region.x as usize * (self.descriptor.format.bits_per_pixel() / 8))
            })
            .ok_or(DisplayError::InvalidTarget)?;
        let buffer = self
            .buffer
            .as_mut()
            .get_mut(offset..)
            .ok_or(DisplayError::InvalidTarget)?;
        Ok(Target::CpuVisible(Surface::new(
            buffer,
            region,
            self.descriptor.stride_bytes,
            self.descriptor.format,
        )?))
    }
    // A direct coherent framebuffer is already visible after CPU rendering.
    fn flush(&mut self, _: &[Rect]) -> Result<(), DisplayError> {
        Ok(())
    }
    fn wait(&mut self, _: ()) -> Result<(), DisplayError> {
        Ok(())
    }
    fn vsync(&mut self) -> VsyncSource {
        VsyncSource::None
    }
}
