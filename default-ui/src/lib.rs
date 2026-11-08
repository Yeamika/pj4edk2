//! Default pages and reference assets; portable contracts live in boot-api.
#![no_std]
extern crate alloc;
mod config;
mod name_card;
mod scripts;
mod snapshot;
pub use pocketjs_boot_api::*;
pub use pocketjs_debug_channel as debug;
pub use config::{default_config, default_settings};
pub use name_card::NameCardSpec;
pub use scripts::{Script, scripts};
pub use snapshot::UiSnapshot;
