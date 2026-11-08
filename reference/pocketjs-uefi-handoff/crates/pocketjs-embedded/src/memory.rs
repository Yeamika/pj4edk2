//! QuickJS allocations use the BSP's Rust GlobalAlloc. Guest C scratch memory
//! still uses the BSP C allocator. Both allocations need a real backing heap.
use crate::ffi::MallocFunctions;
use alloc::alloc::{alloc, dealloc, realloc};
use core::{alloc::Layout, ffi::c_void, mem::size_of, ptr};

// QuickJS's portable allocator contract requires max_align_t alignment.
#[repr(C, align(16))]
struct Header {
    size: usize,
}
const PREFIX: usize = size_of::<Header>();

fn layout(size: usize) -> Option<Layout> {
    Layout::from_size_align(size.checked_add(PREFIX)?, 16).ok()
}
unsafe extern "C" fn malloc(_: *mut c_void, size: usize) -> *mut c_void {
    let Some(layout) = layout(size.max(1)) else {
        return ptr::null_mut();
    };
    let raw = unsafe { alloc(layout) };
    if raw.is_null() {
        return ptr::null_mut();
    }
    unsafe {
        raw.cast::<Header>().write(Header { size: size.max(1) });
        raw.add(PREFIX).cast()
    }
}
unsafe extern "C" fn calloc(opaque: *mut c_void, count: usize, size: usize) -> *mut c_void {
    let Some(size) = count.checked_mul(size) else {
        return ptr::null_mut();
    };
    let raw = unsafe { malloc(opaque, size) };
    if !raw.is_null() {
        unsafe { ptr::write_bytes(raw, 0, size) };
    }
    raw
}
unsafe extern "C" fn free(_: *mut c_void, raw: *mut c_void) {
    if raw.is_null() {
        return;
    }
    let header = unsafe { raw.cast::<u8>().sub(PREFIX).cast::<Header>() };
    let layout = layout(unsafe { (*header).size }).unwrap();
    unsafe { dealloc(header.cast(), layout) };
}
unsafe extern "C" fn resize(opaque: *mut c_void, raw: *mut c_void, size: usize) -> *mut c_void {
    if raw.is_null() {
        return unsafe { malloc(opaque, size) };
    }
    if size == 0 {
        unsafe { free(opaque, raw) };
        return ptr::null_mut();
    }
    let Some(new_layout) = layout(size) else {
        return ptr::null_mut();
    };
    let header = unsafe { raw.cast::<u8>().sub(PREFIX).cast::<Header>() };
    let old_layout = layout(unsafe { (*header).size }).unwrap();
    let next = unsafe { realloc(header.cast(), old_layout, new_layout.size()) };
    if next.is_null() {
        return ptr::null_mut();
    }
    unsafe {
        next.cast::<Header>().write(Header { size });
        next.add(PREFIX).cast()
    }
}
unsafe extern "C" fn usable_size(raw: *const c_void) -> usize {
    if raw.is_null() {
        return 0;
    }
    unsafe { (*raw.cast::<u8>().sub(PREFIX).cast::<Header>()).size }
}
pub static ALLOCATOR: MallocFunctions = MallocFunctions {
    calloc,
    malloc,
    free,
    realloc: resize,
    usable_size,
};
