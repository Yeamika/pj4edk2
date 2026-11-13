/* SPDX-License-Identifier: BSD-2-Clause */
/* ARM64 standalone framebuffer demo: no Linux, initramfs, libc or GOP BLT.
 * After ExitBootServices succeeds, only direct pixel writes and CPU instructions
 * execute. The inherited physical framebuffer mode remains unchanged. */
#include <Uefi.h>
#include <Library/BaseLib.h>
#include <Library/PrintLib.h>
#include <Library/UefiBootServicesTableLib.h>
#include <Protocol/GraphicsOutput.h>
#include <Protocol/SerialIo.h>
static EFI_STATUS LeaveFirmware(EFI_HANDLE Image) {
  EFI_MEMORY_DESCRIPTOR *Map=NULL;UINTN Capacity=0,Bytes,Key,Step;UINT32 Version;
  EFI_STATUS Status;UINTN Attempt;
  for(Attempt=0;Attempt<8;Attempt++) {
    Bytes=Capacity;
    Status=gBS->GetMemoryMap(&Bytes,Map,&Key,&Step,&Version);
    if(Status==EFI_BUFFER_TOO_SMALL) {
      if(Map) gBS->FreePool(Map);
      Capacity=Bytes+Step*8;
      Status=gBS->AllocatePool(EfiLoaderData,Capacity,(VOID **)&Map);
      if(EFI_ERROR(Status)) return Status;
      continue;
    }
    if(EFI_ERROR(Status)) break;
    Status=gBS->ExitBootServices(Image,Key);
    if(!EFI_ERROR(Status)) return EFI_SUCCESS;
    if(Status!=EFI_INVALID_PARAMETER) break;
  }
  if(Map) gBS->FreePool(Map);
  return EFI_ERROR(Status)?Status:EFI_ABORTED;
}
EFI_STATUS EFIAPI BaremetalMain(EFI_HANDLE Image,EFI_SYSTEM_TABLE *System) {
  static CONST UINT32 Colors[]={0xffffff,0xffff00,0x00ffff,0x00ff00,
                                0xff00ff,0xff0000,0x0000ff,0x000000};
  EFI_GRAPHICS_OUTPUT_PROTOCOL *Gop;
  EFI_GRAPHICS_OUTPUT_MODE_INFORMATION Info;
  EFI_PHYSICAL_ADDRESS Base;UINTN Size,X,Y;UINT32 Bpp;
  EFI_SERIAL_IO_PROTOCOL *Serial=NULL;EFI_STATUS Status;
  CHAR8 Message[128];UINTN Length;
  (void)System;
  Status=gBS->LocateProtocol(&gEfiGraphicsOutputProtocolGuid,NULL,(VOID **)&Gop);
  if(EFI_ERROR(Status) || !Gop->Mode || !Gop->Mode->Info) return EFI_UNSUPPORTED;
  Info=*Gop->Mode->Info;Base=Gop->Mode->FrameBufferBase;Size=Gop->Mode->FrameBufferSize;
  Bpp=4;
  if(Info.PixelFormat==PixelBitMask && Info.PixelInformation.RedMask==0xf800 &&
     Info.PixelInformation.GreenMask==0x7e0 && Info.PixelInformation.BlueMask==0x1f) Bpp=2;
  else if(Info.PixelFormat!=PixelRedGreenBlueReserved8BitPerColor &&
          Info.PixelFormat!=PixelBlueGreenRedReserved8BitPerColor) return EFI_UNSUPPORTED;
  if(!Base || !Info.HorizontalResolution || !Info.VerticalResolution ||
     Info.PixelsPerScanLine<Info.HorizontalResolution ||
     (UINT64)Info.PixelsPerScanLine*Info.VerticalResolution*Bpp>Size ||
     Base>MAX_UINT64-Size) return EFI_BAD_BUFFER_SIZE;
  if(!EFI_ERROR(gBS->LocateProtocol(&gEfiSerialIoProtocolGuid,NULL,(VOID **)&Serial))) {
    Length=AsciiSPrint(Message,sizeof(Message),"PJ4EDK2_BAREMETAL starting %ux%u; drawing after ExitBootServices\r\n",
                      Info.HorizontalResolution,Info.VerticalResolution);
    Serial->Write(Serial,&Length,Message);
  }
  gBS->SetWatchdogTimer(0,0,0,NULL);
  Status=LeaveFirmware(Image);if(EFI_ERROR(Status)) return Status;
  DisableInterrupts();
  for(Y=0;Y<Info.VerticalResolution;Y++) {
    volatile UINT8 *Row=(volatile UINT8 *)(UINTN)Base+(UINT64)Y*Info.PixelsPerScanLine*Bpp;
    for(X=0;X<Info.HorizontalResolution;X++) {
      UINT32 Color=Colors[X*8/Info.HorizontalResolution];
      if(Bpp==2) {
        UINT16 Value=((Color>>8)&0xf800)|((Color>>5)&0x7e0)|((Color>>3)&0x1f);
        Row[X*2]=Value;Row[X*2+1]=Value>>8;
      } else {
        Row[X*4]=(Info.PixelFormat==PixelRedGreenBlueReserved8BitPerColor)?Color>>16:Color;
        Row[X*4+1]=Color>>8;
        Row[X*4+2]=(Info.PixelFormat==PixelRedGreenBlueReserved8BitPerColor)?Color:Color>>16;
        Row[X*4+3]=0;
      }
    }
  }
  MemoryFence();
  CpuDeadLoop();
  return EFI_SUCCESS; /* Unreachable, keeps compiler entrypoint checks simple. */
}
