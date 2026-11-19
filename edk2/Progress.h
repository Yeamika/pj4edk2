/* SPDX-License-Identifier: BSD-2-Clause-Patent */
#ifndef PJ4EDK2_EFI_PROGRESS_H
#define PJ4EDK2_EFI_PROGRESS_H

#include <Uefi.h>
#include <Library/BaseMemoryLib.h>
#include <Library/MemoryAllocationLib.h>
#include <Library/UefiBootServicesTableLib.h>
#include <Protocol/GraphicsOutput.h>
#include <Protocol/HiiFont.h>

/* Optional progress inside the caller's validated GOP rectangle. HII draws
 * the system font at its native size into one row; GOP presents that row.
 * No mode changes, direct framebuffer access, panel commands or fixed
 * addresses. Finish all calls before the final GetMemoryMap/ExitBootServices.
 */
typedef struct {
  EFI_GRAPHICS_OUTPUT_PROTOCOL *Gop;
  EFI_HII_FONT_PROTOCOL *Font;
  EFI_FONT_DISPLAY_INFO *Info;
  EFI_IMAGE_OUTPUT *Line;
  UINTN X, Y, Rows, Row, Columns, Lines;
} PJ4EDK2_EFI_PROGRESS;

STATIC VOID Pj4edk2ProgressClose(PJ4EDK2_EFI_PROGRESS *P) {
  if (P->Line) {
    if (P->Line->Image.Bitmap) FreePool(P->Line->Image.Bitmap);
    FreePool(P->Line);
  }
  if (P->Info) FreePool(P->Info);
  ZeroMem(P,sizeof(*P));
}

STATIC EFI_STATUS Pj4edk2ProgressInit(PJ4EDK2_EFI_PROGRESS *P,
    EFI_GRAPHICS_OUTPUT_PROTOCOL *Gop, UINTN X, UINTN Y, UINTN Width, UINTN Height) {
  EFI_GUID Guid=EFI_HII_FONT_PROTOCOL_GUID;
  EFI_FONT_HANDLE Handle=NULL;
  EFI_IMAGE_OUTPUT *Glyph=NULL;
  EFI_GRAPHICS_OUTPUT_BLT_PIXEL Black={0};
  EFI_STATUS Status;
  UINTN CellWidth,CellHeight;

  ZeroMem(P,sizeof(*P));
  if (!Gop || !Gop->Blt || !Gop->Mode || !Gop->Mode->Info ||
      X>Gop->Mode->Info->HorizontalResolution ||
      Y>Gop->Mode->Info->VerticalResolution || Width<=16 || Height<=16 ||
      Width>Gop->Mode->Info->HorizontalResolution-X ||
      Height>Gop->Mode->Info->VerticalResolution-Y || Width>MAX_UINT16)
    return EFI_INVALID_PARAMETER;
  Status=gBS->LocateProtocol(&Guid,NULL,(VOID **)&P->Font);
  if (EFI_ERROR(Status)) goto Fail;
  Status=P->Font->GetFontInfo(P->Font,&Handle,NULL,&P->Info,NULL);
  if (EFI_ERROR(Status)) goto Fail;
  P->Info->ForegroundColor.Blue=0xde;
  P->Info->ForegroundColor.Green=0xde;
  P->Info->ForegroundColor.Red=0xde;
  P->Info->BackgroundColor=Black;
  P->Info->FontInfoMask &= ~(EFI_FONT_INFO_SYS_FORE_COLOR|EFI_FONT_INFO_SYS_BACK_COLOR);
  /* Query the system font without the custom display colors. EDK II can
   * return EFI_WARN_UNKNOWN_GLYPH with no image for an unmatched font;
   * a warning alone does not guarantee that Glyph was allocated. */
  Status=P->Font->GetGlyph(P->Font,L'M',NULL,&Glyph,NULL);
  if (EFI_ERROR(Status)) goto Fail;
  if (!Glyph || !Glyph->Image.Bitmap) { Status=EFI_UNSUPPORTED; goto Fail; }
  CellWidth=Glyph->Width; CellHeight=Glyph->Height+1;
  FreePool(Glyph->Image.Bitmap); FreePool(Glyph); Glyph=NULL;
  if (!CellWidth || CellHeight<2 || CellHeight>Height-16) {
    Status=EFI_UNSUPPORTED; goto Fail;
  }
  P->Columns=(Width-16)/CellWidth;
  if (!P->Columns) { Status=EFI_UNSUPPORTED; goto Fail; }
  P->Line=AllocateZeroPool(sizeof(*P->Line));
  if (!P->Line) { Status=EFI_OUT_OF_RESOURCES; goto Fail; }
  P->Line->Width=(UINT16)(Width-16); P->Line->Height=(UINT16)CellHeight;
  P->Line->Image.Bitmap=AllocateZeroPool((Width-16)*CellHeight*sizeof(Black));
  if (!P->Line->Image.Bitmap) { Status=EFI_OUT_OF_RESOURCES; goto Fail; }
  P->Gop=Gop; P->X=X+8; P->Y=Y+8; P->Rows=(Height-16)/CellHeight;
  Status=Gop->Blt(Gop,&Black,EfiBltVideoFill,0,0,P->X,P->Y,Width-16,Height-16,0);
  if (!EFI_ERROR(Status)) return EFI_SUCCESS;
Fail:
  if (Glyph) {
    if (Glyph->Image.Bitmap) FreePool(Glyph->Image.Bitmap);
    FreePool(Glyph);
  }
  Pj4edk2ProgressClose(P);
  return Status;
}

STATIC EFI_STATUS Pj4edk2ProgressLine(PJ4EDK2_EFI_PROGRESS *P, CONST CHAR16 *Text) {
  CHAR16 Bounded[160];
  UINTN I=0;
  EFI_STATUS Status;

  if (!P->Gop || !Text) return EFI_NOT_READY;
  while (I<P->Columns && I<ARRAY_SIZE(Bounded)-1 && Text[I]) {
    Bounded[I]=(Text[I]>=32 && Text[I]<127)?Text[I]:L'?'; I++;
  }
  Bounded[I]=0;
  ZeroMem(P->Line->Image.Bitmap,
      (UINTN)P->Line->Width*P->Line->Height*sizeof(EFI_GRAPHICS_OUTPUT_BLT_PIXEL));
  Status=P->Font->StringToImage(P->Font,
      EFI_HII_OUT_FLAG_CLIP|EFI_HII_OUT_FLAG_CLIP_CLEAN_X|EFI_HII_IGNORE_LINE_BREAK,
      Bounded,P->Info,&P->Line,0,0,NULL,NULL,NULL);
  if (EFI_ERROR(Status)) return Status;
  Status=P->Gop->Blt(P->Gop,P->Line->Image.Bitmap,EfiBltBufferToVideo,
      0,0,P->X,P->Y+P->Row*P->Line->Height,P->Line->Width,P->Line->Height,0);
  if (!EFI_ERROR(Status)) { P->Row=(P->Row+1)%P->Rows; P->Lines++; }
  return Status;
}
#endif
