//! Shared bare-metal PocketJS embedding: official QuickJS guest, retained VM,
//! fresh user realms, bounded host messages, and registered display/render drivers.
#![no_std]
extern crate alloc;

mod budget;
mod ffi;
mod input;
mod memory;
pub mod output;
mod services;
mod vm;

pub use pocket_drivers::{Contact, InputFrame, InputFrameBuilder, Pointer, TextEvent};
pub use pocketjs_debug_channel as debug;
pub use vm::PocketJs;

/// BSP-owned policy. There is no CPU/board/firmware selector in this crate.
#[derive(Clone, Copy, Debug)]
pub struct Config {
    /// Logical UI dimensions, independent of the display's pixel format.
    pub width: u32,
    pub height: u32,
    pub tick_hz: u32,
    /// QuickJS heap limit only; UI, framebuffers and C guest use additional RAM.
    pub js_heap_bytes: usize,
    pub js_stack_bytes: usize,
    /// Maximum interrupt polls per evaluation/frame, not milliseconds.
    pub interrupt_polls: u32,
    /// CPU-visible display storage reservation across all registered outputs.
    /// Zero permits device-owned targets without allocating a CPU framebuffer.
    pub frame_buffer_budget: usize,
}
impl Config {
    pub(crate) fn validate(self) -> Result<(), Error> {
        // DrawList uses signed 16-bit pixel coordinates.
        if !(1..=32767).contains(&self.width)
            || !(1..=32767).contains(&self.height)
            || !(1..=pocketjs_core::MAX_TICK_HZ).contains(&self.tick_hz)
            || self.js_heap_bytes == 0
            || self.js_stack_bytes == 0
            || self.interrupt_polls == 0
        {
            return Err(Error::InvalidConfig);
        }
        Ok(())
    }
}

/// An owner-local page token; never serialize it or treat it as an EFI handle.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct UserGeneration(pub(crate) u64);
impl UserGeneration {
    pub fn get(self) -> u64 {
        self.0
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum Error {
    InvalidConfig,
    OutOfMemory,
    Runtime,
    Script(alloc::string::String),
    BudgetExceeded,
    PendingJobs,
    Poisoned,
    NoUser,
    StaleGeneration,
    GenerationExhausted,
    QueueFull,
    InvalidInput,
    Presentation(output::PresentError),
}

impl From<output::PresentError> for Error {
    fn from(error: output::PresentError) -> Self {
        Self::Presentation(error)
    }
}
