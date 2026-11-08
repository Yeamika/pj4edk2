//! Shared font and name-card preparation for reference hosts.
#![no_std]
extern crate alloc;
mod fonts;
mod name_card;
pub use fonts::atlases;
pub use name_card::{NameCardImage, bake};
#[derive(Debug)]
pub enum BakeError { InvalidSpec(&'static str), InvalidFont }
