# Firmware hardware services

Status: implemented contract and reference consumer; board hardware validation
is separate. The project name is `pocketjsforedk2`.

`PLATFORM_GUID` identifies an optional firmware protocol, independent of the
resident runtime context ABI. Revision 1 is a 24-byte table on X64/AArch64:
revision, byte size, `read_power`, `set_quiesced`. Both callbacks are mandatory,
use EFI calling convention, and receive the protocol address as `this`. The Rust
declarations in `src/abi/platform.rs` generate the C header; firmware includes
it with `PJB_EDK2` defined. No board constants belong to this contract.

The caller owns the 48-byte `PowerSample` buffer and initializes `size`. The
provider writes only that prefix and never retains the pointer. Unknown
measurements have their validity bit clear. `sample_ms` is a monotonic timestamp
in the provider's boot epoch. Units and current sign are documented in the type.
The saved FGU register is diagnostic data, not a current percentage. An external
input being present does not prove battery charging or bypass operation. No
callback permits changing charger state.

All calls occur before ExitBootServices, on one execution thread at
TPL_APPLICATION; callbacks and buffers must remain live for the call. A consumer
checks revision, size and callbacks before use. Power fields are independently
validated, and a failed read clears prior telemetry. The UI JSON represents
unknown battery level and external power with `null`; existing true/false
configuration remains accepted. Rust consumers use `Option<bool>` for external
power. A successful external-power reading does not change boot protection mode.

`UiConfig.power` and the `PowerState` UI event carry `PowerTelemetry` with
independent nullable `charging`, `voltage_mv` and signed `current_ma` fields.
The object defaults to unknown when omitted, preserving existing JSON inputs.
Positive current flows into the battery; negative current flows out of it, and
zero is a valid measurement. The reference consumer checks each validity bit
separately and rejects a charging value other than 0 or 1 and a zero voltage.
The default page shows the measured battery voltage with either current sign,
and a charging symbol only for `charging: true`. Missing fields display unknown
values. Telemetry updates do not reset the automatic boot countdown.

Touch uses the existing EFI Absolute Pointer protocol, not a second vendor
pointer ABI. Raw coordinates are mapped through protocol min/max to physical GOP
pixels, then rotated together with the display. Errors cancel the current
gesture without a click. The W202DS provider rejects corrupt or multiple-contact
packets and publishes the protocol only after its retained-firmware probe
passes.

Before starting a child, the reference host finishes presentation and drops its
VM and output buffers. After LoadImage succeeds, it calls `set_quiesced(1)`;
failure prevents StartImage. A returning child causes `set_quiesced(0)`. The
board also stops at ExitBootServices, without allocating or calling Boot
Services from that callback. The protocol is not callable after that boundary.
Quiesce and resume are idempotent. Hardware state and memory are owned by the
firmware driver, never by a child image. An absent optional protocol is allowed;
a malformed protocol is an error, not silent fallback.

This protocol does not implement storage, Android image loading, Linux DT
fixups, PSCI reset, persistent settings or a cross-OS resident JavaScript VM. A
configured EFI filename is not evidence that an OS loader is available on the
board.
