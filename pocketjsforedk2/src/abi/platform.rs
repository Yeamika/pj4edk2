//! Optional firmware hardware service. Boot Services lifetime, single caller at
//! TPL_APPLICATION. No ownership transfer. Charging uses a separate optional protocol.
use super::{Guid, Status};

pub const PLATFORM_REVISION: u32 = 1;
pub const PLATFORM_GUID: Guid = Guid {
    data1: 0x38ff2cf1, data2: 0x825c, data3: 0x46d5,
    data4: [0x8f, 0x19, 0xf4, 0x31, 0x2e, 0x3b, 0x45, 0x70],
};
pub const POWER_PERCENT: u32 = 1;
pub const POWER_EXTERNAL: u32 = 2;
pub const POWER_VOLTAGE: u32 = 4;
pub const POWER_CURRENT: u32 = 8;
pub const POWER_TEMPERATURE: u32 = 16;
pub const POWER_CHARGING: u32 = 32;
pub const POWER_RAW_FGU: u32 = 64;
/// Qualifies POWER_PERCENT: based on retained state and may be stale.
pub const POWER_PERCENT_ESTIMATED: u32 = 128;

/// Zero valid bits means unknown, never zero-percent or disconnected. Units:
/// millivolts, signed milliamps (positive into battery), deci-degrees Celsius.
/// saved_soc_raw is diagnostic retained data. POWER_PERCENT_ESTIMATED marks
/// a percentage based on retained or modelled state, not an exact live reading.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PowerSample {
    pub size: u32,
    pub valid: u32,
    pub sample_ms: u64,
    pub percent: u32,
    pub external_power: u32,
    pub voltage_mv: u32,
    pub current_ma: i32,
    pub temperature_decic: i32,
    pub charging: u32,
    pub voltage_raw: u32,
    pub saved_soc_raw: u32,
}

pub type ReadPower = unsafe extern "efiapi" fn(
    protocol: *mut core::ffi::c_void, sample: *mut PowerSample,
) -> Status;
pub type SetQuiesced = unsafe extern "efiapi" fn(
    protocol: *mut core::ffi::c_void, quiesced: u32,
) -> Status;

/// Published with InstallProtocolInterface(PLATFORM_GUID). A successful
/// set_quiesced(1) stops polling and releases the hardware bus. set_quiesced(0)
/// resumes only before ExitBootServices, e.g. when StartImage returns. Both are
/// idempotent; callers must stop invoking this protocol after ExitBootServices.
#[repr(C)]
pub struct PlatformProtocol {
    pub revision: u32,
    pub size: u32,
    pub read_power: ReadPower,
    pub set_quiesced: SetQuiesced,
}

/// Independent optional service; does not change PlatformProtocol revision 1.
pub const CHARGE_CONTROL_REVISION: u32 = 1;
pub const CHARGE_CONTROL_GUID: Guid = Guid {
    data1: 0x69f253de, data2: 0x281e, data3: 0x4aca,
    data4: [0xa7, 0x88, 0x8c, 0x27, 0xce, 0xc4, 0x17, 0x10],
};
pub type ReadChargeEnabled = unsafe extern "efiapi" fn(
    protocol: *mut core::ffi::c_void, enabled: *mut u32,
) -> Status;
pub type SetChargeEnabled = unsafe extern "efiapi" fn(
    protocol: *mut core::ffi::c_void, enabled: u32,
) -> Status;
/// Boot Services only, single caller at TPL_APPLICATION. Values are 0/1.
/// read_enabled reads hardware enable state, not actual battery charge phase.
/// set_enabled succeeds only after readback; the board may reject enable or
/// stop later on faults. Callers refresh state; errors mean unknown state.
/// No automatic persistence. Quiesce/ExitBootServices revoke the enable request.
#[repr(C)]
pub struct ChargeControlProtocol {
    pub revision: u32,
    pub size: u32,
    pub read_enabled: ReadChargeEnabled,
    pub set_enabled: SetChargeEnabled,
    /// Optional tail (check size). Requested state may remain true while
    /// unplugged; explicit stop and real faults revoke it. No persistence.
    pub read_requested: ReadChargeEnabled,
}
