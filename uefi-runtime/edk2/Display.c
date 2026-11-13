/* SPDX-License-Identifier: MIT
 * Standard GOP mode lease. Restore the inherited mode before EFI children.
 * No board registers, framebuffer addresses or pixel-format conversion here.
 */
#include <Uefi.h>
#include <Protocol/GraphicsOutput.h>
#include <Library/UefiBootServicesTableLib.h>
#include <Library/MemoryAllocationLib.h>
#include "pj4edk2_display.h"

static EFI_GRAPHICS_OUTPUT_PROTOCOL *DisplayGop;
static UINT32 DisplayOriginalMode;
static BOOLEAN DisplayModeLeased;
static UINT32 DisplayPanelRotation;

UINT32 pjb_display_rotation(VOID) {return DisplayPanelRotation;}

UINTN pjb_display_stop(VOID) {
    EFI_STATUS Status;
    if (!DisplayModeLeased) return EFI_SUCCESS;
    Status=DisplayGop->SetMode(DisplayGop,DisplayOriginalMode);
    if (!EFI_ERROR(Status)) { DisplayModeLeased=FALSE;DisplayGop=NULL;DisplayPanelRotation=0; }
    return Status;
}

UINTN pjb_display_start(VOID) {
    EFI_GRAPHICS_OUTPUT_PROTOCOL *Gop;
    EFI_GRAPHICS_OUTPUT_MODE_INFORMATION *Info;
    EFI_STATUS Status;
    UINT32 Mode, Width, Height;
    UINTN Size;
    EFI_GUID OrientationGuid=PJ4EDK2_DISPLAY_GUID;
    PJ4EDK2_DISPLAY_PROTOCOL *Orientation=NULL;
    if (DisplayModeLeased) return EFI_ALREADY_STARTED;
    Status=gBS->LocateProtocol(&gEfiGraphicsOutputProtocolGuid,NULL,(VOID **)&Gop);
    if (EFI_ERROR(Status)) return Status;
    if (!Gop || !Gop->Mode || !Gop->Mode->Info) return EFI_UNSUPPORTED;
    Width=Gop->Mode->Info->HorizontalResolution;
    Height=Gop->Mode->Info->VerticalResolution;
    Status=gBS->LocateProtocol(&OrientationGuid,NULL,(VOID **)&Orientation);
    if (EFI_ERROR(Status) || !Orientation || Orientation->revision!=PJ4EDK2_DISPLAY_REVISION ||
        Orientation->size<sizeof(*Orientation) || Orientation->gop!=Gop || !Orientation->rotation ||
        Orientation->handoff_mode>=Gop->Mode->MaxMode)
        Orientation=NULL;
    if (Gop->Mode->Info->PixelFormat==PixelBlueGreenRedReserved8BitPerColor) {
        if (Orientation) {
            UINT32 Rotation=Orientation->rotation(Orientation,Gop->Mode->Mode);
            if (Rotation!=0 && Rotation!=90) return EFI_UNSUPPORTED;
            DisplayPanelRotation=Rotation;
            // GraphicsConsole may have chosen the landscape mode before us.
            // The platform's compatibility mode, not the entry mode, is the
            // required EFI handoff target in that case.
            if (Gop->Mode->Mode!=Orientation->handoff_mode) {
                DisplayGop=Gop;DisplayOriginalMode=Orientation->handoff_mode;DisplayModeLeased=TRUE;
            }
        }
        return EFI_SUCCESS;
    }
    for (Mode=0;Mode<Gop->Mode->MaxMode;++Mode) {
        Info=NULL;Size=0;
        Status=Gop->QueryMode(Gop,Mode,&Size,&Info);
        if (EFI_ERROR(Status)) continue;
        BOOLEAN Swapped=Info && Size>=sizeof(*Info) && Width<Height &&
            Info->HorizontalResolution==Height && Info->VerticalResolution==Width &&
            Orientation && Orientation->rotation(Orientation,Mode)==90;
        BOOLEAN Match=Info && Size>=sizeof(*Info) &&
            Info->PixelFormat==PixelBlueGreenRedReserved8BitPerColor &&
            ((Info->HorizontalResolution==Width && Info->VerticalResolution==Height) || Swapped);
        if (Info) FreePool(Info);
        if (!Match) continue;
        DisplayGop=Gop;DisplayOriginalMode=Orientation?Orientation->handoff_mode:Gop->Mode->Mode;DisplayModeLeased=TRUE;
        DisplayPanelRotation=Swapped?90:0;
        Status=Gop->SetMode(Gop,Mode);
        if (!EFI_ERROR(Status)) return EFI_SUCCESS;
        /* Even a failing SetMode can have touched hardware. Keep the lease
         * until rollback succeeds; a child must never inherit this failure. */
        EFI_STATUS Restored=pjb_display_stop();
        return EFI_ERROR(Restored)?Restored:Status;
    }
    return EFI_SUCCESS; /* Older firmware: retain the GOP BLT fallback. */
}
