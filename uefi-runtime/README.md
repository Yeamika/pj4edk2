# PocketJS UEFI runtime

This directory implements the ARM64 EFI host for the shared interfaces and
default pages. Board drivers, boot targets and firmware packaging belong to
`edk2-ums9620` and `w202ds-boot`. No board checkout or page snapshot is kept
here.

## Build independent components

`tools/build-components.sh` builds `PocketJS.a` containing the Rust engine
adapter/pages, official C guest, QuickJS and EFI host
entry/chainload/time/settings glue. It also exports `abi-support/`: the pinned
EDK II/edk2-libc libraries and generated constructors required to link the
application. Board JSON is compiled separately by the SDK producer.

The repository root `build.sh` supplies these inputs. For direct low-level use:

```sh
export PJB_EDK2_ROOT=/path/to/pinned/edk2
export PJB_EDK2_LIBC=/path/to/pinned/edk2-libc
export EDK_TOOLS_PATH=/path/to/built/BaseTools
export PJB_BUILD_DIR=/path/to/private/work
export PJB_ARCHIVE=/path/to/output/PocketJS.a
bash tools/build-components.sh
```

The PATH must provide GCC/binutils for AArch64, native C tools, make and Python
3\. Runtime archives contain ordinary ARM64 ELF objects, reserve x18, abort on
panic and disable LTO/sanitizers. The lower-level `build-staticlib.sh` emits
only the Rust portion; it is not a complete runtime delivery. `prepare-edk2.py`
generates an isolated work package; it does not modify upstream source.

`w202ds-boot/tools/edk2_sdk.py export` combines platform FFS modules, the ABI
support libraries and board configuration into a pinned SDK. `edk2_sdk.py link`
then links the independent runtime archive and assembles the firmware volumes.
No compile, flash or device command occurs in that final link step.

## Persistent settings

`src/settings.rs` consumes the shared `BootSettings` type; the interface project
and page code own no filesystem implementation. `edk2/Settings.c` uses standard
SimpleFileSystem and PartitionInfo protocols to select exactly one GPT ESP. Zero
ESPs or multiple ESPs fail without choosing an arbitrary disk.

The host creates `\EFI\pj4edk2\settings0.bin` and `settings1.bin` as needed.
Each is a fixed 512-byte record with `PJSSET01`, a monotonic 64-bit sequence, a
bounded JSON payload and CRC32 over the whole record. The payload contains
`timeout_seconds` (0–120) and `default_entry_id`. Startup uses the newest valid
record and validates the entry against the current configuration. Missing or
corrupt records fall back to embedded defaults; a valid older slot survives a
corrupt latest slot. This format is application state, not UEFI variable
storage.

Save writes the inactive slot, flushes and closes the file and directories, then
reopens it for byte-exact verification. Only then does the host acknowledge
SettingsSaved or DefaultEntryRead. Short I/O, write protection, flush failures,
close failures and mismatched readback report failure. Rewriting a truncated
inactive record is supported. Debug enablement remains an explicit session
choice. The two records detect torn content; FAT metadata is not a transactional
filesystem, and sudden power-loss testing remains TODO.

Tests: `bash ../test.sh --rust-only` from this directory (set `CARGO_TARGET_DIR`
outside sources) and
`PJB_EDK2_ROOT=/path/to/edk2 python3 -B tests/test_settings_file.py`. The
explicit `PJB_CARGO_FEATURES=settings-self-test` build uses the real page
requests, persistence adapter and acknowledgements. With a fresh store, its
first RAM boot saves 19 seconds and the second configured entry; the second boot
checks reload and restores the embedded defaults. It refuses other existing
sequences. Normal builds leave this feature unset. Board logs and hardware
acceptance belong to the BSP; self-test binaries are never normal resident
firmware.

## Runtime limits

The runtime handles its own Recovery page: diagnostics read current display,
touch, power, charging and settings-file status; Boot log reads a bounded 8 KiB
application-session ring independently of optional SerialIo output. This is the
current application's recent log, not a previous-boot or complete BSP log.
Restore startup settings saves the embedded default entry with timeout zero
through the existing inactive-bank flush/readback path. It reports success only
after verification. No OS recovery loader or partition repair is invoked. The
shared `RecoveryResult` event drives paged results and updates page settings
only after a successful restore. Hardware acceptance belongs to the BSP.

The host uses GOP BLT, console keyboard and optional absolute-pointer protocols.
Firmware owns hardware initialization. The optional pocketjsforedk2 platform
protocol supplies power telemetry once per second and quiesce/resume around
StartImage. Unknown or failed reads clear displayed telemetry. The host persists
boot timeout and default entry on a standard GPT ESP. The application runs until
an explicit exit or boot request; the platform owns any recovery watchdog
deadline. Board configuration provides the boot targets and timeout; a
configured path does not prove an OS loader is installed. See the shared
[hardware contract](../pocketjsforedk2/docs/hardware.md).

When a child EFI application returns, the host recreates the menu after
releasing the previous VM and display buffers. Launch errors are displayed on
the menu; a successful diagnostic return is not reported as an error. Automatic
startup is disabled for that returned menu without changing the saved ESP
settings. The page regression checks error display, cancellation of repeated
automatic launches, settings input and an explicit retry. Device validation is
pending.

The software renderer retains its logical framebuffer and reports a stable
buffer identity for partial refresh. Portrait GOP output rotates damaged
rectangles into a compact buffer capped at 2 MiB. Larger updates are split into
strips; small damage is still submitted in one BLT. This avoids keeping a second
full 1920×1200×4 buffer alongside the logical framebuffer, which failed to
allocate when storage was connected in a 48 MiB UEFI environment. The event loop
waits only the remainder of its 16 ms budget. Diagnostic frame counts measure
software iterations, not panel scanout frequency; timing logs separate script,
advance and presentation work.

Historical QEMU/device evidence is not included in this source repository; it
does not validate newly built firmware. For QEMU, `tools/qemu-smoke.py` accepts
an explicit EFI input and portable QEMU tools directory. Physical hardware tests
belong to the board workspace.

For an explicit CPU model and display size:

```sh
python3 tools/qemu-smoke.py --efi /path/to/PocketJsBoot.efi \
  --tools /path/to/qemu/sysroot --run /path/to/fresh/run \
  --cpu cortex-a55 --cpus 1 --size 1200x1920
```

`tools/qemu-profile.py` runs selection changes, settings and the boot animation,
saving serial logs, screenshots, hashes and `profile.json`. It pins the guest to
one available host CPU. `--cpu-budget 50` applies a CPU-time feedback limit with
a 5 ms control interval to test a smaller CPU budget; the report records actual
host CPU consumption. Use a fresh `--run` directory for each case. The test
reaches an EFI boot request without installing an OS loader.

`tools/qemu-boot-profile.py` measures startup separately. It starts QEMU paused
at virtual reset, enables the requested CPU budget, then times `cont` through
the first `PJB_FRAME_OK` and captures that first screen. For example:

```sh
python3 tools/qemu-boot-profile.py --efi /path/to/PocketJsBoot.efi \
  --tools /path/to/qemu/sysroot --run /path/to/fresh/boot-run \
  --cpu cortex-a55 --cpu-budget 50 --size 1200x1920
```

The boot limiter caps accumulated idle credit at 5 ms, so firmware waits cannot
let later UI initialization run at full CPU speed. The report saves host wall
times to application entry, protocol readiness and first UI submission, the
application's own startup timer, CPU consumption and a PNG. Serial markers are
polled every 5 ms and include observation bounds. The total includes AAVMF's
initialization and waits; it is a virtual boot measurement, not a physical
board's power-on or backlight timing. The interaction profiler above enables its
limit after protocol readiness and does not measure a complete boot. The
underlying smoke helper's `--paused` option returns when the paused QMP monitor
is available.

`tools/qemu-frame-profile.py` measures continuous selection and settings
transitions using a probe entirely inside the emulator tooling:

```sh
python3 tools/qemu-frame-profile.py --efi /path/to/PocketJsBoot.efi \
  --tools /path/to/qemu/sysroot --run /path/to/fresh/frame-run \
  --cpu cortex-a55 --cpu-budget 50 --size 1200x1920 --frames 101
```

The EFI is copied unchanged and its hash is checked. `--trace-display` on the
smoke helper enables QEMU's `virtio_gpu_cmd_res_flush` trace. The profiler
receives trace lines over a FIFO and timestamps them on another host CPU; it
saves the raw trace, timestamps, inputs, screenshots and frame statistics. It
rejects measured windows containing multiple flushes in the same FIFO read,
because those timestamps would be ambiguous. Times include probe and
input-driver overhead and measure virtual display submission rather than
physical scanout. No runtime or board firmware probe is built or installed.

Each workload sends one key after each observed display update, creating
continuous navigation or alternating settings transitions. The default 101
submissions produce 100 consecutive frame intervals per workload. Mean FPS is
interval count divided by elapsed time; P95 uses nearest rank; 5% low is the
inverse of the mean of the slowest 5% of intervals. Initial
input-to-first-update latency is saved separately. These are defined workloads,
not an FPS measurement of an idle menu or an OS boot animation.

`tools/qemu-stack-profile.py` generates guest-call-stack flame graphs using the
emulator's optional local Unix GDB socket. It accepts explicit `--elf`,
`--gdb-root`, `--python-libs` (pygdbmi) and `--flamegraph` (the standard
`flamegraph.pl`) inputs, in addition to `--efi`, `--tools` and a fresh `--run`.
It locates the loaded image by matching its entry instructions, loads the
existing ELF symbols at that base, and samples the same continuous workloads.
Unwinding stops at `PjbOnStack`, where the entry trampoline switches stacks; raw
frames beyond that boundary are retained only as diagnostic evidence. JSON
records whether each unwind reached that boundary. SVGs support zoom and search
and include samples whose outer stack could not be recovered. The default is 400
samples per workload at randomised intervals around 25 ms.

This probe uses GDB interrupts and therefore pauses the guest. Its timings must
not be combined with the independent frame profiler's FPS measurements. The
smoke helper's `--gdb` flag exposes only a local Unix socket, and is off by
default. The original EFI hash is checked; no probe is compiled into it.

QEMU CPU models select instruction features; changing A55 to A53 does not
reproduce their physical throughput, caches or clocks. CPU budgets are synthetic
stress conditions, rather than physical CPU-frequency equivalents. Firmware
slow-frame logs are sampled at most once per second; their median is not a
median over every rendered frame, and loop counts are not display FPS.

## Optional charging control

`Hardware.c` locates and validates the independent public ChargeControlProtocol.
The runtime executes SetChargingEnabled, reads hardware state again, then queues
ChargeConfigured for the next UI tick. Periodic ChargeState refresh
distinguishes unsupported hardware from a supported device whose state is
unknown. No platform registers or charging policy live in this host, and
charging is not persisted.

The opt-in `charge-self-test` feature drives the actual button through page key
events and the normal host handler, checks acknowledgements, and requests stop
on completion or failure. It is excluded from normal firmware. W202DS physical
RAM tests passed enable/stop with signed battery current; see that BSP's
delivery for measurements. `tests/test_hardware_bridge.py` covers
absent/malformed protocols, read errors, invalid values and child-launch failure
paths.

## Linux boot output

The host validates the page's `display_regions` against the actual GOP and
passes `boot-output` through the public optional BootDisplay configuration table
during child execution. Existing EFI LoadOptions remain intact. Return and
quiesce-failure paths restore the previous table before the temporary record
expires. A board's OS loader owns framebuffer validation and the OS-specific
handoff; this runtime contains no Linux DT or board MMIO logic.

`cargo test --manifest-path Cargo.toml --locked --test boot_display --test rotation --test settings`
checks geometry and existing consumers. The C hardware bridge test checks child
launch, table publication/restoration and failure cleanup.
