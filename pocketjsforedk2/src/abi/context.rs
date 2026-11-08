//! Configuration-table contract and the resident service callback type.

use core::ffi::c_void;

use super::{
    efi::Status,
    heap::HeapRange,
    request::{Request, Response},
};

pub const SIGNATURE: u64 = 0x3130_5854_4353_4a50;
pub const ABI_MAJOR: u16 = 1;
pub const ABI_MINOR: u16 = 0;
pub const CAP_LIVE_INSTANCE: u64 = 1;
pub const CAP_HEAP_RANGES: u64 = 2;
pub const CAP_SINGLE_CONSUMER: u64 = 4;
pub const REQUIRED_CAPABILITIES: u64 = CAP_LIVE_INSTANCE | CAP_HEAP_RANGES | CAP_SINGLE_CONSUMER;

/// The callback stays in the original resident EFI image. Callers must
/// satisfy that owner's pointer, lease and Boot Services lifetime contract.
pub type Invoke = Option<
    unsafe extern "efiapi" fn(
        runtime_instance: *mut c_void,
        request: *const Request,
        response: *mut Response,
    ) -> Status,
>;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct ContextTable {
    pub signature: u64,
    pub struct_size: u32,
    pub abi_major: u16,
    pub abi_minor: u16,
    pub machine_type: u32,
    pub pointer_bits: u32,
    pub capabilities: u64,
    pub generation: u64,
    pub owner_image_handle: *mut c_void,
    pub runtime_instance: *mut c_void,
    pub heap_ranges: *const HeapRange,
    pub heap_range_count: u32,
    pub heap_range_entry_size: u32,
    pub invoke: Invoke,
    pub reserved: [u64; 4],
}
