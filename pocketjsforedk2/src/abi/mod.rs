//! Public data contract, independent of consumer validation and engine internals.

mod efi;
mod context;
mod guid;
mod heap;
mod request;
mod platform;
mod boot_display;
pub use boot_display::*;
pub use platform::*;

pub use context::{
    ABI_MAJOR, ABI_MINOR, CAP_HEAP_RANGES, CAP_LIVE_INSTANCE, CAP_SINGLE_CONSUMER, ContextTable,
    Invoke, REQUIRED_CAPABILITIES, SIGNATURE,
};
pub use efi::{INVALID_PARAMETER, MACHINE, SUCCESS, Status, UNSUPPORTED};
pub use guid::{CONTEXT_GUID, Guid};
pub use heap::{HEAP_ASSETS, HEAP_MIXED, HEAP_QUICKJS, HEAP_UI, HeapRange};
pub use request::{OP_ACQUIRE, OP_PUMP, OP_QUIESCE, OP_RELEASE, Request, Response};
