//! EFI-compatible GUID layout and the PocketJS configuration-table identifier.

#[repr(C)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Guid {
    pub data1: u32,
    pub data2: u16,
    pub data3: u16,
    pub data4: [u8; 8],
}

/// cbindgen:ignore
pub const CONTEXT_GUID: Guid = Guid {
    data1: 0x6988_824e,
    data2: 0x3b93,
    data3: 0x4bd4,
    data4: [0xbf, 0x07, 0x65, 0x51, 0x7b, 0x0f, 0x90, 0x6d],
};
