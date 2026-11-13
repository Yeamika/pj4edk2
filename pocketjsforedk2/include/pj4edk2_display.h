/* SPDX-License-Identifier: MIT
 * Optional GOP mode orientation metadata. Swapping resolution alone does not
 * establish a mapping from native touch/panel coordinates to framebuffer RAM.
 */
#ifndef PJ4EDK2_DISPLAY_H
#define PJ4EDK2_DISPLAY_H
#include <Uefi.h>
#include <Protocol/GraphicsOutput.h>
#define PJ4EDK2_DISPLAY_GUID {0x9ac2ec81,0x3a76,0x4cec,{0x98,0xa2,0x4e,0x7d,0x51,0x25,0x18,0x63}}
#define PJ4EDK2_DISPLAY_REVISION 1
typedef struct PJ4EDK2_DISPLAY_PROTOCOL PJ4EDK2_DISPLAY_PROTOCOL;
struct PJ4EDK2_DISPLAY_PROTOCOL {
  UINT32 revision,size;
  UINT32 handoff_mode;
  EFI_GRAPHICS_OUTPUT_PROTOCOL *gop;
  /* Clockwise framebuffer-to-native-panel rotation, before panel mounting.
   * Return MAX_UINT32 for an unsupported mode. Boot Services lifetime only. */
  UINT32 (EFIAPI *rotation)(PJ4EDK2_DISPLAY_PROTOCOL *This,UINT32 Mode);
};
#endif
