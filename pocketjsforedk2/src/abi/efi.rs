//! EFI machine identifiers and status representation used by the public contract.

#[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
compile_error!("Draft 1.0 supports UEFI X64 and AArch64 only");

#[cfg(target_arch = "x86_64")]
pub const MACHINE: u32 = 0x8664;
#[cfg(target_arch = "aarch64")]
pub const MACHINE: u32 = 0xaa64;

pub type Status = usize;
pub const SUCCESS: Status = 0;
pub const INVALID_PARAMETER: Status = 0x8000_0000_0000_0002;
pub const UNSUPPORTED: Status = 0x8000_0000_0000_0003;
