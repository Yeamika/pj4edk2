//! The same baked atlases feed the live page and the one-shot name-card asset.

const ATLASES: [&[u8]; 8] = [
    include_bytes!(concat!(env!("OUT_DIR"), "/font-0.atlas")),
    include_bytes!(concat!(env!("OUT_DIR"), "/font-1.atlas")),
    include_bytes!(concat!(env!("OUT_DIR"), "/font-2.atlas")),
    include_bytes!(concat!(env!("OUT_DIR"), "/font-3.atlas")),
    include_bytes!(concat!(env!("OUT_DIR"), "/font-4.atlas")),
    include_bytes!(concat!(env!("OUT_DIR"), "/font-5.atlas")),
    include_bytes!(concat!(env!("OUT_DIR"), "/font-6.atlas")),
    include_bytes!(concat!(env!("OUT_DIR"), "/font-7.atlas")),
];

const COMPACT: [&[u8]; 8] = [
    include_bytes!(concat!(env!("OUT_DIR"), "/font-0-compact.atlas")),
    include_bytes!(concat!(env!("OUT_DIR"), "/font-1-compact.atlas")),
    include_bytes!(concat!(env!("OUT_DIR"), "/font-2-compact.atlas")),
    include_bytes!(concat!(env!("OUT_DIR"), "/font-3-compact.atlas")),
    include_bytes!(concat!(env!("OUT_DIR"), "/font-4-compact.atlas")),
    include_bytes!(concat!(env!("OUT_DIR"), "/font-5-compact.atlas")),
    include_bytes!(concat!(env!("OUT_DIR"), "/font-6-compact.atlas")),
    include_bytes!(concat!(env!("OUT_DIR"), "/font-7-compact.atlas")),
];

pub fn atlases(compact: bool) -> [&'static [u8]; 8] {
    if compact { COMPACT } else { ATLASES }
}

