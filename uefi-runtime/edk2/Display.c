/* SPDX-License-Identifier: MIT
 * Standard GOP selection and same-layout handoff to EFI children.
 * No board registers, framebuffer addresses or pixel-format conversion here.
 */
#include <Uefi.h>
#include <Protocol/GraphicsOutput.h>
#include <Library/UefiBootServicesTableLib.h>
#include <Library/MemoryAllocationLib.h>
#include "pj4edk2_display.h"
#include "pj4edk2_rotate.h"

static EFI_GRAPHICS_OUTPUT_PROTOCOL *DisplayGop;
static PJ4EDK2_DISPLAY_PROTOCOL *DisplayOrientation;
static BOOLEAN DisplayModeLeased;
static UINT32 DisplayPanelRotation;
static PJ4EDK2_ROTATE_PROTOCOL *DisplayRotator;
static BOOLEAN RotationDisabled;

UINTN pjb_display_rotate(CONST VOID *Source,UINTN SourceBytes,VOID *Destination,UINTN DestinationBytes,
    UINTN DestinationStride,UINT32 Width,UINT32 Height,UINT32 X,UINT32 Y,UINT32 W,UINT32 H) {
    EFI_GUID Guid=PJ4EDK2_ROTATE_GUID;
    EFI_STATUS Status;
    PJ4EDK2_ROTATE_PROTOCOL *Provider=NULL;
    PJ4EDK2_ROTATE_REQUEST Request;
    if (RotationDisabled) return EFI_UNSUPPORTED;
    if (!DisplayRotator) {
        Status=gBS->LocateProtocol(&Guid,NULL,(VOID **)&Provider);
        if (EFI_ERROR(Status) || !Provider || Provider->revision!=PJ4EDK2_ROTATE_REVISION ||
            Provider->size<sizeof(*Provider) || !Provider->rotate || !Provider->stop) {
            RotationDisabled=TRUE;return EFI_UNSUPPORTED;
        }
        DisplayRotator=Provider;
    }
    Request=(PJ4EDK2_ROTATE_REQUEST){Source,SourceBytes,(UINTN)Width*4,Destination,DestinationBytes,
        DestinationStride,Width,Height,X,Y,W,H};
    Status=DisplayRotator->rotate(DisplayRotator,&Request);
    // NOT_READY can decline small updates without disabling the provider.
    if (EFI_ERROR(Status) && Status!=EFI_NOT_READY) RotationDisabled=TRUE;
    return Status;
}

UINT32 pjb_display_rotation(VOID) {return DisplayPanelRotation;}

UINTN pjb_display_stop(VOID) {
    EFI_STATUS Status;
    if (DisplayRotator) {
        Status=DisplayRotator->stop(DisplayRotator);
        if (EFI_ERROR(Status)) return Status;
        DisplayRotator=NULL;
    }
    RotationDisabled=FALSE;
    if (!DisplayModeLeased) return EFI_SUCCESS;
    Status=DisplayOrientation && DisplayOrientation->prepare_handoff ?
        DisplayOrientation->prepare_handoff(DisplayOrientation) : EFI_SUCCESS;
    if (!EFI_ERROR(Status)) { DisplayModeLeased=FALSE;DisplayGop=NULL;DisplayOrientation=NULL;DisplayPanelRotation=0; }
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
            DisplayOrientation=Orientation;
            DisplayGop=Gop;DisplayModeLeased=TRUE;
        }
        return EFI_SUCCESS;
    }
    DisplayOrientation=Orientation;
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
        DisplayGop=Gop;DisplayModeLeased=TRUE;
        DisplayPanelRotation=Swapped?90:0;
        Status=Gop->SetMode(Gop,Mode);
        if (!EFI_ERROR(Status)) return EFI_SUCCESS;
        /* Even a failing SetMode can have touched hardware. Keep the lease
         * until a safe handoff succeeds; a child must never inherit this failure. */
        EFI_STATUS Prepared=pjb_display_stop();
        return EFI_ERROR(Prepared)?Prepared:Status;
    }
    return EFI_SUCCESS; /* Older firmware: retain the GOP BLT fallback. */
}
