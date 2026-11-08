#ifndef POCKETJS_BOOT_CONTEXT_H
#define POCKETJS_BOOT_CONTEXT_H

/* Generated from Rust by tools/generate_header.rs + cbindgen 0.29.2. Do not edit. */

#ifdef PJB_EDK2
#include <Base.h>
typedef UINT8 uint8_t;
typedef UINT16 uint16_t;
typedef UINT32 uint32_t;
typedef UINT64 uint64_t;
typedef INT32 int32_t;
typedef UINTN uintptr_t;
#ifndef offsetof
#define offsetof(type, field) OFFSET_OF(type, field)
#endif
#else
#include <stdint.h>
#include <stddef.h>
#endif
#if defined(__x86_64__) || defined(_M_X64)
#define PJB_TARGET_X64
#if defined(_MSC_VER)
#define PJB_EFIAPI
#else
#define PJB_EFIAPI __attribute__((ms_abi))
#endif
#elif defined(__aarch64__) || defined(_M_ARM64)
#define PJB_TARGET_AARCH64
#define PJB_EFIAPI
#else
#error "Only UEFI X64 and AArch64 are supported"
#endif


#if defined(PJB_TARGET_X64)
#define PJB_MACHINE 34404
#endif

#if defined(PJB_TARGET_AARCH64)
#define PJB_MACHINE 43620
#endif

#define PJB_CONTEXT_SIGNATURE 3544430025670609488

#define PJB_ABI_MAJOR 1

#define PJB_ABI_MINOR 0

#define PJB_CAP_LIVE_INSTANCE 1

#define PJB_CAP_HEAP_RANGES 2

#define PJB_CAP_SINGLE_CONSUMER 4

#define PJB_REQUIRED_CAPABILITIES ((PJB_CAP_LIVE_INSTANCE | PJB_CAP_HEAP_RANGES) | PJB_CAP_SINGLE_CONSUMER)

#define PJB_HEAP_MIXED 0

#define PJB_HEAP_QUICKJS 1

#define PJB_HEAP_UI 2

#define PJB_HEAP_ASSETS 3

#define PJB_OP_ACQUIRE 1

#define PJB_OP_PUMP 2

#define PJB_OP_RELEASE 3

#define PJB_OP_QUIESCE 4

#define PJB_PLATFORM_REVISION 1

#define PJB_POWER_PERCENT 1

#define PJB_POWER_EXTERNAL 2

#define PJB_POWER_VOLTAGE 4

#define PJB_POWER_CURRENT 8

#define PJB_POWER_TEMPERATURE 16

#define PJB_POWER_CHARGING 32

#define PJB_POWER_RAW_FGU 64

/**
 * Qualifies POWER_PERCENT: based on retained state and may be stale.
 */
#define PJB_POWER_PERCENT_ESTIMATED 128

/**
 * Independent optional service; does not change PlatformProtocol revision 1.
 */
#define PJB_CHARGE_CONTROL_REVISION 1

#define PJB_BOOT_DISPLAY_REVISION 1

typedef uintptr_t PJB_STATUS;

typedef struct {
  uint64_t base_address;
  uint64_t byte_length;
  uint32_t kind;
  uint32_t reserved;
} PJB_HEAP_RANGE;

typedef struct {
  uint32_t struct_size;
  uint32_t operation;
  uint64_t generation;
  void *consumer_image_handle;
  uint64_t lease;
} PJB_REQUEST;

typedef struct {
  uint32_t struct_size;
  uint32_t reserved;
  uint64_t lease;
} PJB_RESPONSE;

/**
 * The callback stays in the original resident EFI image. Callers must
 * satisfy that owner's pointer, lease and Boot Services lifetime contract.
 */
typedef PJB_STATUS (PJB_EFIAPI *PJB_INVOKE)(void *runtime_instance,
                                 const PJB_REQUEST *request,
                                 PJB_RESPONSE *response);

typedef struct {
  uint64_t signature;
  uint32_t struct_size;
  uint16_t abi_major;
  uint16_t abi_minor;
  uint32_t machine_type;
  uint32_t pointer_bits;
  uint64_t capabilities;
  uint64_t generation;
  void *owner_image_handle;
  void *runtime_instance;
  const PJB_HEAP_RANGE *heap_ranges;
  uint32_t heap_range_count;
  uint32_t heap_range_entry_size;
  PJB_INVOKE invoke;
  uint64_t reserved[4];
} PJB_CONTEXT_TABLE;

typedef struct {
  uint32_t data1;
  uint16_t data2;
  uint16_t data3;
  uint8_t data4[8];
} PJB_GUID;

/**
 * Zero valid bits means unknown, never zero-percent or disconnected. Units:
 * millivolts, signed milliamps (positive into battery), deci-degrees Celsius.
 * saved_soc_raw is diagnostic retained data. POWER_PERCENT_ESTIMATED marks
 * a percentage based on retained or modelled state, not an exact live reading.
 */
typedef struct {
  uint32_t size;
  uint32_t valid;
  uint64_t sample_ms;
  uint32_t percent;
  uint32_t external_power;
  uint32_t voltage_mv;
  int32_t current_ma;
  int32_t temperature_decic;
  uint32_t charging;
  uint32_t voltage_raw;
  uint32_t saved_soc_raw;
} PJB_POWER_SAMPLE;

typedef PJB_STATUS (PJB_EFIAPI *PJB_READ_POWER)(void *protocol, PJB_POWER_SAMPLE *sample);

typedef PJB_STATUS (PJB_EFIAPI *PJB_SET_QUIESCED)(void *protocol, uint32_t quiesced);

/**
 * Published with InstallProtocolInterface(PLATFORM_GUID). A successful
 * set_quiesced(1) stops polling and releases the hardware bus. set_quiesced(0)
 * resumes only before ExitBootServices, e.g. when StartImage returns. Both are
 * idempotent; callers must stop invoking this protocol after ExitBootServices.
 */
typedef struct {
  uint32_t revision;
  uint32_t size;
  PJB_READ_POWER read_power;
  PJB_SET_QUIESCED set_quiesced;
} PJB_PLATFORM_PROTOCOL;

typedef PJB_STATUS (PJB_EFIAPI *PJB_READ_CHARGE_ENABLED)(void *protocol, uint32_t *enabled);

typedef PJB_STATUS (PJB_EFIAPI *PJB_SET_CHARGE_ENABLED)(void *protocol, uint32_t enabled);

/**
 * Boot Services only, single caller at TPL_APPLICATION. Values are 0/1.
 * read_enabled reads hardware enable state, not actual battery charge phase.
 * set_enabled succeeds only after readback; the board may reject enable or
 * stop later on faults. Callers refresh state; errors mean unknown state.
 * No automatic persistence. Quiesce/ExitBootServices revoke the enable request.
 */
typedef struct {
  uint32_t revision;
  uint32_t size;
  PJB_READ_CHARGE_ENABLED read_enabled;
  PJB_SET_CHARGE_ENABLED set_enabled;
  /**
   * Optional tail (check size). Requested state may remain true while
   * unplugged; explicit stop and real faults revoke it. No persistence.
   */
  PJB_READ_CHARGE_ENABLED read_requested;
} PJB_CHARGE_CONTROL_PROTOCOL;

/**
 * The boot-output rectangle on the primary GOP, in logical landscape pixels.
 * Rotation is clockwise from logical coordinates to GOP pixels (0 or 90).
 * The child must copy this record before ExitBootServices; it is not runtime
 * memory, a framebuffer address, nor permission to access arbitrary scanout.
 */
typedef struct {
  uint32_t revision;
  uint32_t size;
  uint32_t display_width;
  uint32_t display_height;
  uint32_t x;
  uint32_t y;
  uint32_t width;
  uint32_t height;
  uint32_t rotation;
} PJB_BOOT_DISPLAY;

#define PJB_SUCCESS 0

#define PJB_INVALID_PARAMETER 9223372036854775810ull

#define PJB_UNSUPPORTED 9223372036854775811ull

#ifdef __cplusplus
extern "C" {
#endif // __cplusplus

/**
 * Validate the public ABI header and dispatch into the original resident service.
 * Runtime state, leases and operation semantics remain with that service.
 *
 * # Safety
 * Before Boot Services exit, table must reference a readable, aligned header
 * through struct_size, and the full table when that size is sufficient.
 * Request/response follow the same size contract; response must be writable.
 * All pointers, heap descriptors and callback code must remain live for the
 * call. Buffers must not alias data mutated by the callback. Null is rejected.
 */
PJB_STATUS pjb_call(const PJB_CONTEXT_TABLE *table,
                    const PJB_REQUEST *request,
                    PJB_RESPONSE *response);

#ifdef __cplusplus
}  // extern "C"
#endif  // __cplusplus


#define PJB_CONTEXT_GUID ((PJB_GUID){0x6988824e, 0x3b93, 0x4bd4, {0xbf, 0x07, 0x65, 0x51, 0x7b, 0x0f, 0x90, 0x6d}})
#define PJB_PLATFORM_GUID ((PJB_GUID){0x38ff2cf1, 0x825c, 0x46d5, {0x8f, 0x19, 0xf4, 0x31, 0x2e, 0x3b, 0x45, 0x70}})
#define PJB_CHARGE_CONTROL_GUID ((PJB_GUID){0x69f253de, 0x281e, 0x4aca, {0xa7, 0x88, 0x8c, 0x27, 0xce, 0xc4, 0x17, 0x10}})

#if defined(__cplusplus)
#define PJB_ASSERT static_assert
#else
#define PJB_ASSERT _Static_assert
#endif
#define PJB_BOOT_DISPLAY_GUID ((PJB_GUID){0xd1a604f6, 0xa2bf, 0x4b83, {0x93, 0x62, 0x15, 0x40, 0x82, 0x1e, 0x9f, 0xa1}})
PJB_ASSERT(sizeof(PJB_BOOT_DISPLAY) == 36, "Rust/C layout: PJB_BOOT_DISPLAY");
PJB_ASSERT(sizeof(void *) == 8, "Rust/C layout: void *");
PJB_ASSERT(sizeof(PJB_GUID) == 16, "Rust/C layout: PJB_GUID");
PJB_ASSERT(sizeof(PJB_HEAP_RANGE) == 24, "Rust/C layout: PJB_HEAP_RANGE");
PJB_ASSERT(sizeof(PJB_REQUEST) == 32, "Rust/C layout: PJB_REQUEST");
PJB_ASSERT(sizeof(PJB_RESPONSE) == 16, "Rust/C layout: PJB_RESPONSE");
PJB_ASSERT(sizeof(PJB_CONTEXT_TABLE) == 112, "Rust/C layout: PJB_CONTEXT_TABLE");
PJB_ASSERT(sizeof(PJB_PLATFORM_PROTOCOL) == 24, "Rust/C layout: PJB_PLATFORM_PROTOCOL");
PJB_ASSERT(sizeof(PJB_CHARGE_CONTROL_PROTOCOL) == 32, "Rust/C layout: PJB_CHARGE_CONTROL_PROTOCOL");
PJB_ASSERT(sizeof(PJB_POWER_SAMPLE) == 48, "Rust/C layout: PJB_POWER_SAMPLE");
PJB_ASSERT(offsetof(PJB_CONTEXT_TABLE, runtime_instance) == 48, "Rust/C field: runtime_instance");
PJB_ASSERT(offsetof(PJB_CONTEXT_TABLE, invoke) == 72, "Rust/C field: invoke");
#undef PJB_ASSERT

#endif  /* POCKETJS_BOOT_CONTEXT_H */
