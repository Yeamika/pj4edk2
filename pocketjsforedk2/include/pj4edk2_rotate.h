/* SPDX-License-Identifier: MIT
 * Optional synchronous clockwise BGRX rotation. Boot Services lifetime only.
 * The provider owns its DMA buffers. On error the caller's destination is
 * untouched and software fallback is safe; stop must succeed before a child
 * can reclaim the provider's memory or replace its IOMMU configuration.
 */
#ifndef PJ4EDK2_ROTATE_H
#define PJ4EDK2_ROTATE_H
#include <Uefi.h>
#define PJ4EDK2_ROTATE_GUID {0x70fe085c,0x6d6a,0x47f8,{0xa1,0x96,0xc8,0x69,0x67,0x53,0x8c,0x8e}}
#define PJ4EDK2_ROTATE_REVISION 1
typedef struct {
  CONST VOID *source;
  UINTN source_bytes,source_stride;
  VOID *destination;
  UINTN destination_bytes,destination_stride;
  UINT32 width,height,x,y,damage_width,damage_height;
} PJ4EDK2_ROTATE_REQUEST;
typedef struct PJ4EDK2_ROTATE_PROTOCOL PJ4EDK2_ROTATE_PROTOCOL;
struct PJ4EDK2_ROTATE_PROTOCOL {
  UINT32 revision,size;
  EFI_STATUS (EFIAPI *rotate)(PJ4EDK2_ROTATE_PROTOCOL *This,CONST PJ4EDK2_ROTATE_REQUEST *Request);
  EFI_STATUS (EFIAPI *stop)(PJ4EDK2_ROTATE_PROTOCOL *This);
};
#endif
