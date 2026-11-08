use crate::ffi::JsRuntime;
use alloc::boxed::Box;
use core::{
    cell::Cell,
    ffi::{c_int, c_void},
};

pub struct Budget {
    remaining: Cell<u32>,
    pub exhausted: Cell<bool>,
}
impl Budget {
    pub fn new() -> Box<Self> {
        Box::new(Self {
            remaining: Cell::new(0),
            exhausted: Cell::new(false),
        })
    }
    pub fn arm(&self, polls: u32) {
        self.remaining.set(polls);
        self.exhausted.set(false);
    }
}
// QuickJS invokes this on the owner thread only, including during job execution.
// Poll counts bound VM work; they are not a hard wall-clock deadline for native
// functions, parsing, allocation, layout, or rendering.
pub unsafe extern "C" fn interrupt(_: *mut JsRuntime, opaque: *mut c_void) -> c_int {
    let budget = unsafe { &*opaque.cast::<Budget>() };
    let left = budget.remaining.get();
    if left == 0 {
        budget.exhausted.set(true);
        1
    } else {
        budget.remaining.set(left - 1);
        0
    }
}
