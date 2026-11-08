//! Portable boot, input, configuration and output-region contracts.
//! Hosts own the runtime, storage, devices and policy. No engine is linked here.
#![no_std]
extern crate alloc;
mod config;
mod event;
pub mod services;
pub use config::{BootEntry, Mode, PowerTelemetry, UiConfig};
pub use event::{UiAction, UiEvent};
