/* SPDX-License-Identifier: BSD-2-Clause-Patent */
/* Lab-only ramfb GOP mode adapter. Some AAVMF builds expose only 640/800/1024
 * modes. Configure a new reserved-RAM surface using QEMU's fw_cfg interface,
 * keeping the original GOP protocol address valid for firmware consumers.
 * The parent application stays resident until its children return/EBS. */
#include <Uefi.h>
#include <Library/BaseLib.h>
#include <Library/BaseMemoryLib.h>
#include <Library/FrameBufferBltLib.h>
#include <Library/MemoryAllocationLib.h>
#include <Library/QemuFwCfgLib.h>
#include <Library/UefiBootServicesTableLib.h>
#include <Protocol/GraphicsOutput.h>
RETURN_STATUS EFIAPI QemuFwCfgInitialize(VOID);
#pragma pack(1)
typedef struct {UINT64 Address;UINT32 Format,Flags,Width,Height,Stride;} RAMFB;
#pragma pack()
static EFI_GRAPHICS_OUTPUT_PROTOCOL *Gop,Original;
static EFI_GRAPHICS_OUTPUT_PROTOCOL_MODE Mode;
static EFI_GRAPHICS_OUTPUT_MODE_INFORMATION Info;
static FRAME_BUFFER_CONFIGURE *Blt;
static FIRMWARE_CONFIG_ITEM Selector;
static UINTN BltBytes,Pages;
static BOOLEAN Replaced,Changed;
static UINT32 Previous;
static EFI_STATUS EFIAPI Query(EFI_GRAPHICS_OUTPUT_PROTOCOL *This,UINT32 Number,
    UINTN *Size,EFI_GRAPHICS_OUTPUT_MODE_INFORMATION **Out) {
  (void)This;
  if(!Size || !Out || Number) return EFI_INVALID_PARAMETER;
  *Out=AllocateCopyPool(sizeof(Info),&Info);
  if(!*Out) return EFI_OUT_OF_RESOURCES;
  *Size=sizeof(Info);return EFI_SUCCESS;
}
static EFI_STATUS EFIAPI Set(EFI_GRAPHICS_OUTPUT_PROTOCOL *This,UINT32 Number) {
  RAMFB Config;
  (void)This;
  if(Number) return EFI_UNSUPPORTED;
  Config=(RAMFB){SwapBytes64(Mode.FrameBufferBase),SwapBytes32(0x34325258),0,
    SwapBytes32(Info.HorizontalResolution),SwapBytes32(Info.VerticalResolution),
    SwapBytes32(Info.PixelsPerScanLine*4)};
  ZeroMem((VOID *)(UINTN)Mode.FrameBufferBase,Mode.FrameBufferSize);
  QemuFwCfgSelectItem(Selector);QemuFwCfgWriteBytes(sizeof(Config),&Config);
  return EFI_SUCCESS;
}
static EFI_STATUS EFIAPI Draw(EFI_GRAPHICS_OUTPUT_PROTOCOL *This,
    EFI_GRAPHICS_OUTPUT_BLT_PIXEL *Buffer,EFI_GRAPHICS_OUTPUT_BLT_OPERATION Operation,
    UINTN Sx,UINTN Sy,UINTN Dx,UINTN Dy,UINTN W,UINTN H,UINTN Delta) {
  (void)This;
  return FrameBufferBlt(Blt,Buffer,Operation,Sx,Sy,Dx,Dy,W,H,Delta);
}
EFI_STATUS Pj4edk2QemuDisplayInit(VOID) {
  EFI_STATUS Status;RETURN_STATUS Result;UINTN Bytes,Size;
  FIRMWARE_CONFIG_ITEM Item;UINT32 Width=1920,Height=1200,N;
  CHAR8 Text[32],*End;
  Status=gBS->LocateProtocol(&gEfiGraphicsOutputProtocolGuid,NULL,(VOID **)&Gop);
  if(EFI_ERROR(Status) || !Gop->Mode || !Gop->Mode->Info) return EFI_UNSUPPORTED;
  Original=*Gop;Previous=Gop->Mode->Mode;
  QemuFwCfgInitialize();
  if(QemuFwCfgIsAvailable() &&
      !EFI_ERROR(QemuFwCfgFindFile("opt/pj4edk2/display",&Item,&Bytes))) {
    if(!Bytes || Bytes>=sizeof(Text)) return EFI_INVALID_PARAMETER;
    ZeroMem(Text,sizeof(Text));QemuFwCfgSelectItem(Item);QemuFwCfgReadBytes(Bytes,Text);
    if(RETURN_ERROR(AsciiStrDecimalToUintnS(Text,&End,&Size)) || *End!='x' || Size>4096) return EFI_INVALID_PARAMETER;
    Width=Size;
    if(RETURN_ERROR(AsciiStrDecimalToUintnS(End+1,&End,&Size)) || *End || Size>2400) return EFI_INVALID_PARAMETER;
    Height=Size;
  }
  if(Width<320 || Height<200) return EFI_INVALID_PARAMETER;
  /* Prefer ordinary GOP modes; the fallback applies only to QEMU ramfb. */
  for(N=0;N<Gop->Mode->MaxMode;N++) {
    EFI_GRAPHICS_OUTPUT_MODE_INFORMATION *Candidate=NULL;
    if(!EFI_ERROR(Gop->QueryMode(Gop,N,&Bytes,&Candidate))) {
      BOOLEAN Match=Candidate->HorizontalResolution==Width && Candidate->VerticalResolution==Height;
      FreePool(Candidate);
      if(Match) {
        if(N==Previous) return EFI_SUCCESS;
        Status=Gop->SetMode(Gop,N);Changed=!EFI_ERROR(Status);return Status;
      }
    }
  }
  if(!QemuFwCfgIsAvailable() || EFI_ERROR(QemuFwCfgFindFile("etc/ramfb",&Selector,&Bytes)) ||
      Bytes!=sizeof(RAMFB)) return EFI_UNSUPPORTED;
  Info=(EFI_GRAPHICS_OUTPUT_MODE_INFORMATION){0,Width,Height,
    PixelBlueGreenRedReserved8BitPerColor,{0,0,0,0},Width};
  Pages=EFI_SIZE_TO_PAGES((UINTN)Width*Height*4);
  Mode=(EFI_GRAPHICS_OUTPUT_PROTOCOL_MODE){1,0,&Info,sizeof(Info),
    (UINTN)AllocateReservedPages(Pages),EFI_PAGES_TO_SIZE(Pages)};
  if(!Mode.FrameBufferBase) return EFI_OUT_OF_RESOURCES;
  Result=FrameBufferBltConfigure((VOID *)(UINTN)Mode.FrameBufferBase,&Info,NULL,&BltBytes);
  if(Result!=RETURN_BUFFER_TOO_SMALL) goto Fail;
  Blt=AllocatePool(BltBytes);
  if(!Blt) {Result=RETURN_OUT_OF_RESOURCES;goto Fail;}
  Result=FrameBufferBltConfigure((VOID *)(UINTN)Mode.FrameBufferBase,&Info,Blt,&BltBytes);
  if(RETURN_ERROR(Result)) goto Fail;
  *Gop=(EFI_GRAPHICS_OUTPUT_PROTOCOL){Query,Set,Draw,&Mode};Replaced=TRUE;
  return Set(Gop,0);
 Fail:
  if(Blt) FreePool(Blt);
  FreePages((VOID *)(UINTN)Mode.FrameBufferBase,Pages);return Result;
}
VOID Pj4edk2QemuDisplayRestore(VOID) {
  if(Replaced) {
    *Gop=Original;
    /* Restore the original firmware's scanout before freeing ours. */
    if(EFI_ERROR(Gop->SetMode(Gop,Previous))) CpuDeadLoop();
    FreePool(Blt);FreePages((VOID *)(UINTN)Mode.FrameBufferBase,Pages);
    Replaced=FALSE;
  } else if(Changed && EFI_ERROR(Gop->SetMode(Gop,Previous))) CpuDeadLoop();
}
