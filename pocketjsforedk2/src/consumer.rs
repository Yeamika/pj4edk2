//! Consumer-side ABI validation and dispatch into the original resident service.

use core::mem::size_of;

use crate::abi::{
    ABI_MAJOR, ContextTable, HeapRange, INVALID_PARAMETER, MACHINE, REQUIRED_CAPABILITIES, Request,
    Response, SIGNATURE, Status, UNSUPPORTED,
};

/// Validate the public ABI header and dispatch into the original resident service.
/// Runtime state, leases and operation semantics remain with that service.
///
/// # Safety
/// Before Boot Services exit, table must reference a readable, aligned header
/// through struct_size, and the full table when that size is sufficient.
/// Request/response follow the same size contract; response must be writable.
/// All pointers, heap descriptors and callback code must remain live for the
/// call. Buffers must not alias data mutated by the callback. Null is rejected.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pjb_call(
    table: *const ContextTable,
    request: *const Request,
    response: *mut Response,
) -> Status {
    if table.is_null() || request.is_null() || response.is_null() {
        return INVALID_PARAMETER;
    }
    if unsafe { (*table).signature } != SIGNATURE
        || unsafe { (*table).struct_size } < size_of::<ContextTable>() as u32
    {
        return UNSUPPORTED;
    }
    let table = unsafe { &*table };
    if table.abi_major != ABI_MAJOR
        || table.machine_type != MACHINE
        || table.pointer_bits != 64
        || table.capabilities & REQUIRED_CAPABILITIES != REQUIRED_CAPABILITIES
    {
        return UNSUPPORTED;
    }
    if unsafe { (*request).struct_size } < size_of::<Request>() as u32
        || unsafe { (*response).struct_size } < size_of::<Response>() as u32
    {
        return INVALID_PARAMETER;
    }
    let request_ref = unsafe { &*request };
    if table.generation == 0
        || request_ref.generation != table.generation
        || table.owner_image_handle.is_null()
        || table.runtime_instance.is_null()
        || table.heap_ranges.is_null()
        || table.heap_range_count == 0
        || table.heap_range_entry_size < size_of::<HeapRange>() as u32
    {
        return INVALID_PARAMETER;
    }
    match table.invoke {
        Some(invoke) => unsafe { invoke(table.runtime_instance, request, response) },
        None => INVALID_PARAMETER,
    }
}
