//! Request and response envelopes, without runtime operation implementations.

use core::ffi::c_void;

pub const OP_ACQUIRE: u32 = 1;
pub const OP_PUMP: u32 = 2;
pub const OP_RELEASE: u32 = 3;
pub const OP_QUIESCE: u32 = 4;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct Request {
    pub struct_size: u32,
    pub operation: u32,
    pub generation: u64,
    pub consumer_image_handle: *mut c_void,
    pub lease: u64,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct Response {
    pub struct_size: u32,
    pub reserved: u32,
    pub lease: u64,
}
