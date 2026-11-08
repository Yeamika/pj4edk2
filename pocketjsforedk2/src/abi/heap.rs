//! Heap range descriptions; allocation and ownership stay with the resident service.

pub const HEAP_MIXED: u32 = 0;
pub const HEAP_QUICKJS: u32 = 1;
pub const HEAP_UI: u32 = 2;
pub const HEAP_ASSETS: u32 = 3;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct HeapRange {
    pub base_address: u64,
    pub byte_length: u64,
    pub kind: u32,
    pub reserved: u32,
}
