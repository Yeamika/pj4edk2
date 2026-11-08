//! Rust is the source for the generated pocketjs_boot_context.h. The resident engine,
//! allocator, UEFI host, package source and debug transport belong in Rust.
//! This crate defines their ABI only; it does not instantiate PocketJS.
#![no_std]
#![deny(unsafe_op_in_unsafe_fn)]

mod abi;
mod consumer;
mod layout;

pub use abi::*;
pub use consumer::pjb_call;
