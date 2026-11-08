//! Pointer-only QuickJS-NG FFI. JSValue stays inside the official C guest.
use core::ffi::{c_char, c_int, c_void};
use pocket_drivers::abi::PocketGuestV1;

#[repr(C)]
pub struct JsRuntime {
    _private: [u8; 0],
}
#[repr(C)]
pub struct JsContext {
    _private: [u8; 0],
}
#[repr(C)]
pub struct Guest {
    _private: [u8; 0],
}

#[repr(C)]
pub struct MallocFunctions {
    pub calloc: unsafe extern "C" fn(*mut c_void, usize, usize) -> *mut c_void,
    pub malloc: unsafe extern "C" fn(*mut c_void, usize) -> *mut c_void,
    pub free: unsafe extern "C" fn(*mut c_void, *mut c_void),
    pub realloc: unsafe extern "C" fn(*mut c_void, *mut c_void, usize) -> *mut c_void,
    pub usable_size: unsafe extern "C" fn(*const c_void) -> usize,
}

unsafe extern "C" {
    pub fn JS_NewRuntime2(functions: *const MallocFunctions, opaque: *mut c_void)
    -> *mut JsRuntime;
    pub fn JS_FreeRuntime(runtime: *mut JsRuntime);
    pub fn JS_NewContext(runtime: *mut JsRuntime) -> *mut JsContext;
    pub fn JS_FreeContext(context: *mut JsContext);
    pub fn JS_SetMemoryLimit(runtime: *mut JsRuntime, limit: usize);
    pub fn JS_SetMaxStackSize(runtime: *mut JsRuntime, limit: usize);
    pub fn JS_UpdateStackTop(runtime: *mut JsRuntime);
    pub fn JS_SetInterruptHandler(
        runtime: *mut JsRuntime,
        callback: unsafe extern "C" fn(*mut JsRuntime, *mut c_void) -> c_int,
        opaque: *mut c_void,
    );
    pub fn JS_IsJobPending(runtime: *mut JsRuntime) -> bool;
    pub fn JS_RunGC(runtime: *mut JsRuntime);
    pub fn pocket_guest_quickjs_attach(context: *mut JsContext, profile: c_int) -> *mut Guest;
    pub fn pocket_guest_quickjs_vtable() -> *const PocketGuestV1;
    pub fn pocket_guest_quickjs_error(guest: *const Guest) -> *const c_char;
    pub fn pocket_guest_quickjs_destroy(guest: *mut Guest);
}
