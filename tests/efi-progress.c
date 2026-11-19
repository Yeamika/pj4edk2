/* SPDX-License-Identifier: BSD-2-Clause-Patent */
/* Exercise the actual EDK II adapter against bounded RAM and real EDK types. */
#include <assert.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#undef NULL
#include "../edk2/Progress.h"

static EFI_GRAPHICS_OUTPUT_BLT_PIXEL pixels[240][320];
static UINTN allocations, attempts, fail_at, string_calls, presents;
static BOOLEAN missing_font, fail_render, empty_glyph;
static EFI_HII_FONT_PROTOCOL font;
static EFI_BOOT_SERVICES services;
EFI_BOOT_SERVICES *gBS=&services;

VOID *EFIAPI AllocateZeroPool(UINTN Bytes) {
  VOID *P;
  if (++attempts==fail_at) return NULL;
  P=calloc(1,Bytes); if (P) allocations++;
  return P;
}
VOID EFIAPI FreePool(VOID *P) { assert(P && allocations); allocations--; free(P); }
VOID *EFIAPI ZeroMem(VOID *P,UINTN Bytes) { return memset(P,0,Bytes); }

static EFI_STATUS EFIAPI locate(EFI_GUID *Guid,VOID *Registration,VOID **Out) {
  EFI_GUID Expected=EFI_HII_FONT_PROTOCOL_GUID;
  (void)Registration;
  assert(!memcmp(Guid,&Expected,sizeof(Expected)));
  if (missing_font) return EFI_NOT_FOUND;
  *Out=&font; return EFI_SUCCESS;
}
static EFI_STATUS EFIAPI get_info(CONST EFI_HII_FONT_PROTOCOL *This,
    EFI_FONT_HANDLE *Handle,CONST EFI_FONT_DISPLAY_INFO *In,
    EFI_FONT_DISPLAY_INFO **Out,CONST EFI_STRING String) {
  (void)This;(void)Handle;(void)In;(void)String;
  *Out=AllocateZeroPool(sizeof(**Out));
  if (!*Out) return EFI_OUT_OF_RESOURCES;
  (*Out)->FontInfo.FontSize=19;
  return EFI_SUCCESS;
}
static EFI_STATUS EFIAPI get_glyph(CONST EFI_HII_FONT_PROTOCOL *This,CHAR16 Ch,
    CONST EFI_FONT_DISPLAY_INFO *Info,EFI_IMAGE_OUTPUT **Out,UINTN *Baseline) {
  (void)This;(void)Baseline;
  assert(Ch==L'M' && Info==NULL);
  if (empty_glyph) return EFI_WARN_UNKNOWN_GLYPH;
  *Out=AllocateZeroPool(sizeof(**Out));
  if (!*Out) return EFI_OUT_OF_RESOURCES;
  (*Out)->Width=8;(*Out)->Height=19;
  (*Out)->Image.Bitmap=AllocateZeroPool(8*19*sizeof(pixels[0][0]));
  return (*Out)->Image.Bitmap?EFI_SUCCESS:EFI_OUT_OF_RESOURCES;
}
static EFI_STATUS EFIAPI render(CONST EFI_HII_FONT_PROTOCOL *This,
    EFI_HII_OUT_FLAGS Flags,CONST EFI_STRING Text,CONST EFI_FONT_DISPLAY_INFO *Info,
    EFI_IMAGE_OUTPUT **Image,UINTN X,UINTN Y,EFI_HII_ROW_INFO **Rows,
    UINTN *RowCount,UINTN *Columns) {
  UINTN I;
  (void)This;(void)Rows;(void)RowCount;(void)Columns;
  assert(X==0 && Y==0 && !(Flags&EFI_HII_DIRECT_TO_SCREEN));
  assert(!(Info->FontInfoMask&EFI_FONT_INFO_RESIZE));
  for(I=0;Text[I];I++) assert(Text[I]>=32 && Text[I]<127);
  assert(I<=(*Image)->Width/8 && I<160);
  string_calls++;
  if(fail_render) return EFI_DEVICE_ERROR;
  for(I=0;I<(UINTN)(*Image)->Width*(*Image)->Height;I++)
    (*Image)->Image.Bitmap[I]=Info->ForegroundColor;
  return EFI_SUCCESS;
}
static EFI_STATUS EFIAPI blt(EFI_GRAPHICS_OUTPUT_PROTOCOL *This,
    EFI_GRAPHICS_OUTPUT_BLT_PIXEL *Buffer,EFI_GRAPHICS_OUTPUT_BLT_OPERATION Op,
    UINTN SX,UINTN SY,UINTN DX,UINTN DY,UINTN W,UINTN H,UINTN Delta) {
  UINTN X,Y;
  (void)This;(void)Delta;
  assert(!SX && !SY && DX>=48 && DY>=40 && W<=224 && H<=160);
  assert(DX+W<=272 && DY+H<=200);
  assert(Op==EfiBltVideoFill || Op==EfiBltBufferToVideo);
  for(Y=0;Y<H;Y++)for(X=0;X<W;X++)
    pixels[DY+Y][DX+X]=Buffer[Op==EfiBltVideoFill?0:Y*W+X];
  presents++;return EFI_SUCCESS;
}
int main(void) {
  EFI_GRAPHICS_OUTPUT_MODE_INFORMATION ModeInfo={0};
  EFI_GRAPHICS_OUTPUT_PROTOCOL_MODE Mode={0};
  EFI_GRAPHICS_OUTPUT_PROTOCOL Gop={0};
  PJ4EDK2_EFI_PROGRESS P={0};
  UINTN I,X,Y;
  CHAR16 Long[400];
  services.LocateProtocol=locate;
  font.GetFontInfo=get_info;font.GetGlyph=get_glyph;font.StringToImage=render;
  ModeInfo.HorizontalResolution=320;ModeInfo.VerticalResolution=240;
  Mode.Info=&ModeInfo;Gop.Mode=&Mode;Gop.Blt=blt;
  memset(pixels,0xa5,sizeof(pixels));
  assert(!EFI_ERROR(Pj4edk2ProgressInit(&P,&Gop,40,32,240,176)));
  assert(P.Columns==28 && P.Rows==8 && P.Line->Height==20);
  for(I=0;I<399;I++) Long[I]=I==4?L'\n':L'A';
  Long[399]=0;
  for(I=0;I<100;I++)assert(!EFI_ERROR(Pj4edk2ProgressLine(&P,Long)));
  assert(P.Lines==100 && string_calls==100);
  for(Y=0;Y<240;Y++)for(X=0;X<320;X++)
    if(X<48 || X>=272 || Y<40 || Y>=200)
      assert(((unsigned char *)&pixels[Y][X])[0]==0xa5);
  fail_render=TRUE;
  assert(EFI_ERROR(Pj4edk2ProgressLine(&P,L"failure")) && P.Lines==100);
  fail_render=FALSE;
  Pj4edk2ProgressClose(&P);assert(!allocations);
  assert(Pj4edk2ProgressLine(&P,L"closed")==EFI_NOT_READY);
  missing_font=TRUE;
  assert(EFI_ERROR(Pj4edk2ProgressInit(&P,&Gop,40,32,240,176)) && !allocations);
  missing_font=FALSE;
  empty_glyph=TRUE;
  assert(Pj4edk2ProgressInit(&P,&Gop,40,32,240,176)==EFI_UNSUPPORTED && !allocations);
  empty_glyph=FALSE;
  assert(EFI_ERROR(Pj4edk2ProgressInit(&P,&Gop,319,32,240,176)));
  assert(EFI_ERROR(Pj4edk2ProgressInit(&P,&Gop,40,32,240,16)));
  for(I=1;I<=5;I++) {
    attempts=0;fail_at=I;
    assert(EFI_ERROR(Pj4edk2ProgressInit(&P,&Gop,40,32,240,176)));
    assert(!allocations && !P.Gop);
  }
  puts("PASS: GOP bounds, native font, line limits, wrap, missing HII, empty glyph warning, allocation failures, cleanup");
  return 0;
}
