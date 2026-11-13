#![cfg_attr(target_os = "none", no_std)]
extern crate alloc;

pub mod page;
pub mod boot_display;
pub mod platform_config;
pub mod rotation;
pub mod power;
pub mod settings;
pub mod discovery;
#[cfg(target_os = "none")]
mod firmware;
#[cfg(all(target_os = "none", feature = "settings-self-test"))]
mod settings_test;

#[cfg(target_os = "none")]
mod allocator {
    use core::{alloc::{GlobalAlloc, Layout}, ffi::c_void, ptr};
    unsafe extern "C" { fn malloc(size: usize) -> *mut c_void; fn free(pointer: *mut c_void); fn abort() -> !; }
    struct LibcHeap;
    unsafe impl GlobalAlloc for LibcHeap {
        unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
            let alignment = layout.align().max(align_of::<usize>());
            let Some(length) = layout.size().checked_add(alignment).and_then(|v| v.checked_add(size_of::<usize>())) else { return ptr::null_mut(); };
            let base = unsafe { malloc(length) }.cast::<u8>();
            if base.is_null() { return base; }
            let aligned = ((base as usize + size_of::<usize>() + alignment - 1) & !(alignment - 1)) as *mut u8;
            unsafe { aligned.cast::<usize>().sub(1).write(base as usize); }
            aligned
        }
        unsafe fn dealloc(&self, pointer: *mut u8, _: Layout) {
            unsafe { free(pointer.cast::<usize>().sub(1).read() as *mut c_void); }
        }
    }
    #[global_allocator]
    static HEAP: LibcHeap = LibcHeap;
    #[panic_handler]
    fn panic(_: &core::panic::PanicInfo<'_>) -> ! { unsafe { abort() } }
}
