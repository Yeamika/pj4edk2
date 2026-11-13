#![allow(clippy::missing_safety_doc)]

// The pinned embedding is generated into the build directory. This adapter
// selects nearest texture sampling for the pixel preview without editing it.
pub(crate) use pj4edk2_rendering::PixelResources;
#[allow(dead_code)]
mod upstream {
    include!(concat!(env!("OUT_DIR"), "/browser_embedding.rs"));
}
pub use upstream::*;
