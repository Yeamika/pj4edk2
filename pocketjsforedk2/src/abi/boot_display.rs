//! Optional configuration table, valid only during the launched EFI child.
use super::Guid;

/// cbindgen:ignore
pub const BOOT_DISPLAY_GUID: Guid = Guid {
    data1: 0xd1a604f6, data2: 0xa2bf, data3: 0x4b83,
    data4: [0x93, 0x62, 0x15, 0x40, 0x82, 0x1e, 0x9f, 0xa1],
};
pub const BOOT_DISPLAY_REVISION: u32 = 1;

/// The boot-output rectangle on the primary GOP, in logical landscape pixels.
/// Rotation is clockwise from logical coordinates to GOP pixels (0 or 90).
/// The child must copy this record before ExitBootServices; it is not runtime
/// memory, a framebuffer address, nor permission to access arbitrary scanout.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BootDisplay {
    pub revision: u32,
    pub size: u32,
    pub display_width: u32,
    pub display_height: u32,
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
    pub rotation: u32,
}
