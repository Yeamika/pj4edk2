/* SPDX-License-Identifier: MIT
 * Optional Boot Services protocol. A platform owns its USB, partitions and
 * Fastboot implementation; PJ4EDK2 schedules it without board dependencies.
 */
#ifndef PJ4EDK2_FASTBOOT_H
#define PJ4EDK2_FASTBOOT_H
#include <Uefi.h>
#define PJ4EDK2_FASTBOOT_GUID {0x5eb2355c,0x8108,0x48de,{0x89,0x68,0x9e,0xd9,0x55,0x63,0x8d,0x18}}
#define PJ4EDK2_FASTBOOT_REVISION 1
#define PJ4EDK2_FASTBOOT_BUSY 1u
#define PJ4EDK2_FASTBOOT_CONTINUE 2u
#define PJ4EDK2_FASTBOOT_REBOOT 4u
#define PJ4EDK2_FASTBOOT_BOOTLOADER 8u
typedef struct PJ4EDK2_FASTBOOT_PROTOCOL PJ4EDK2_FASTBOOT_PROTOCOL;
struct PJ4EDK2_FASTBOOT_PROTOCOL {
    UINT32 revision, size;
    /* Start/Poll must not wait for host enumeration or a USB transfer. */
    EFI_STATUS (EFIAPI *start)(PJ4EDK2_FASTBOOT_PROTOCOL *This);
    EFI_STATUS (EFIAPI *poll)(PJ4EDK2_FASTBOOT_PROTOCOL *This, UINT32 *Flags);
    /* Cancels requests before any EFI child or OS receives USB ownership. */
    EFI_STATUS (EFIAPI *stop)(PJ4EDK2_FASTBOOT_PROTOCOL *This);
};
#endif
