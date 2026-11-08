//! Compile-time ABI layout checks, kept separate from types and dispatch.

use core::mem::{offset_of, size_of};

use crate::abi::{ContextTable, Guid, HeapRange, Request, Response};

const _: () = {
    assert!(size_of::<usize>() == 8);
    assert!(size_of::<Guid>() == 16);
    assert!(size_of::<HeapRange>() == 24);
    assert!(size_of::<Request>() == 32);
    assert!(size_of::<Response>() == 16);
    assert!(size_of::<ContextTable>() == 112);
    assert!(offset_of!(ContextTable, runtime_instance) == 48);
    assert!(offset_of!(ContextTable, invoke) == 72);
};
