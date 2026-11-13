/* SPDX-License-Identifier: BSD-2-Clause-Patent */
/* Generic EFI-stub bridge. LoadOptions: absolute child EFI path, space, its
 * command line. Requires pj4edk2's optional BootDisplay table and linear GOP.
 * Only EFI-reserved RAM is accepted: no arbitrary MMIO or BLT-only scanout. */
#include <Uefi.h>
#include <Guid/Fdt.h>
#include <Library/BaseLib.h>
#include <Library/BaseMemoryLib.h>
#include <Library/DevicePathLib.h>
#include <Library/MemoryAllocationLib.h>
#include <Library/UefiBootServicesTableLib.h>
#include <Library/UefiLib.h>
#include <Protocol/LoadedImage.h>
#include <Protocol/GraphicsOutput.h>
#include <libfdt.h>
#define PJB_EDK2
#include "pocketjs_boot_context.h"
#include "pj4edk2_linux_display.h"
#include "pj4edk2_linux_fdt.h"
/* Prove full EFI reserved-memory coverage, including multiple descriptors. */
STATIC BOOLEAN ReservedRam(UINT64 Base,UINT64 Bytes) {
  EFI_MEMORY_DESCRIPTOR *Map=NULL,*D; UINTN Size=0,Key,Step,Offset;
  UINT32 Version; EFI_STATUS Status; UINT64 Cursor=Base,End=Base+Bytes;
  Status=gBS->GetMemoryMap(&Size,Map,&Key,&Step,&Version);
  if(Status!=EFI_BUFFER_TOO_SMALL) return FALSE;
  Size+=Step*8; Map=AllocatePool(Size); if(!Map) return FALSE;
  Status=gBS->GetMemoryMap(&Size,Map,&Key,&Step,&Version);
  if(EFI_ERROR(Status)) {FreePool(Map);return FALSE;}
  while(Cursor<End) {
    BOOLEAN Found=FALSE;
    for(Offset=0;Offset<Size;Offset+=Step) {
      UINT64 Limit;
      D=(VOID *)((UINT8 *)Map+Offset); Limit=D->PhysicalStart+EFI_PAGES_TO_SIZE(D->NumberOfPages);
      if(D->Type==EfiReservedMemoryType && D->PhysicalStart<=Cursor && Limit>Cursor &&
          (D->Attribute & EFI_MEMORY_WB)) {Cursor=MIN(Limit,End);Found=TRUE;break;}
    }
    if(!Found) {FreePool(Map);return FALSE;}
  }
  FreePool(Map);return TRUE;
}
EFI_STATUS EFIAPI Pj4edk2BridgeMain(EFI_HANDLE Handle,EFI_SYSTEM_TABLE *System) {
  PJB_GUID WireGuid=PJB_BOOT_DISPLAY_GUID; EFI_GUID DisplayGuid;
  EFI_LOADED_IMAGE_PROTOCOL *Self,*Child;
  EFI_GRAPHICS_OUTPUT_PROTOCOL *Gop;
  EFI_DEVICE_PATH_PROTOCOL *Path=NULL;
  PJB_BOOT_DISPLAY *Display=NULL;
  VOID *Original=NULL,*Dtb=NULL;
  EFI_HANDLE Image=NULL; EFI_STATUS Status;
  CHAR16 *Options=NULL,*Command; UINTN Index,Length;
  UINT32 Cells[20],Phandle; INT32 Reserved,Node,Chosen;
  struct pj4edk2_surface Surface; struct pj4edk2_region Region;
  fdt64_t Reg[2]; BOOLEAN Installed=FALSE;
  CopyMem(&DisplayGuid,&WireGuid,sizeof(DisplayGuid));
  for(Index=0;Index<System->NumberOfTableEntries;Index++) {
    EFI_CONFIGURATION_TABLE *T=&System->ConfigurationTable[Index];
    if(CompareGuid(&T->VendorGuid,&DisplayGuid)) Display=T->VendorTable;
    if(CompareGuid(&T->VendorGuid,&gFdtTableGuid)) Original=T->VendorTable;
  }
  if(!Display || Display->revision!=1 || Display->size<sizeof(*Display) ||
      !Original || fdt_check_header(Original)) return EFI_UNSUPPORTED;
  Status=gBS->LocateProtocol(&gEfiGraphicsOutputProtocolGuid,NULL,(VOID **)&Gop);
  if(EFI_ERROR(Status) || !Gop->Mode || !Gop->Mode->Info ||
      !Gop->Mode->FrameBufferBase || Gop->Mode->FrameBufferSize>MAX_UINT32 ||
      Gop->Mode->Info->PixelsPerScanLine>MAX_UINT32/4) return EFI_UNSUPPORTED;
  Surface=(struct pj4edk2_surface){Gop->Mode->FrameBufferBase,
    Gop->Mode->FrameBufferSize,Gop->Mode->FrameBufferSize,
    Gop->Mode->Info->HorizontalResolution,Gop->Mode->Info->VerticalResolution,
    Gop->Mode->Info->PixelsPerScanLine*4,0,Display->rotation,0};
  if(Gop->Mode->Info->PixelFormat==PixelBlueGreenRedReserved8BitPerColor) Surface.format=2;
  else if(Gop->Mode->Info->PixelFormat==PixelRedGreenBlueReserved8BitPerColor) Surface.format=3;
  else return EFI_UNSUPPORTED;
  Region=(struct pj4edk2_region){Display->display_width,Display->display_height,
    Display->x,Display->y,Display->width,Display->height};
  if(!pj4edk2_linux_display_cells(&Surface,&Region,Cells) ||
      !ReservedRam(Surface.address,Surface.capacity)) return EFI_NO_MAPPING;
  Status=gBS->HandleProtocol(Handle,&gEfiLoadedImageProtocolGuid,(VOID **)&Self);
  if(EFI_ERROR(Status)) return Status;
  if(!Self->LoadOptions || Self->LoadOptionsSize<4 || Self->LoadOptionsSize>65536 ||
      Self->LoadOptionsSize%2) return EFI_INVALID_PARAMETER;
  Length=Self->LoadOptionsSize/2;
  Options=AllocateZeroPool((Length+1)*sizeof(CHAR16));
  if(!Options) return EFI_OUT_OF_RESOURCES;
  CopyMem(Options,Self->LoadOptions,Self->LoadOptionsSize);
  Command=Options; while(*Command && *Command!=L' ') Command++;
  if(*Command) *Command++=0;
  if(Options[0]!=L'\\') {Status=EFI_INVALID_PARAMETER;goto Done;}
  Path=FileDevicePath(Self->DeviceHandle,Options);
  if(!Path) {Status=EFI_OUT_OF_RESOURCES;goto Done;}
  Status=gBS->LoadImage(FALSE,Handle,Path,NULL,0,&Image);
  if(EFI_ERROR(Status)) {if(Image)gBS->UnloadImage(Image);goto Done;}
  Status=gBS->HandleProtocol(Image,&gEfiLoadedImageProtocolGuid,(VOID **)&Child);
  if(EFI_ERROR(Status)) goto Unload;
  Child->LoadOptions=Command; Child->LoadOptionsSize=StrSize(Command);
  Dtb=AllocatePages(EFI_SIZE_TO_PAGES(fdt_totalsize(Original)+16384));
  if(!Dtb) {Status=EFI_OUT_OF_RESOURCES;goto Unload;}
  Status=EFI_VOLUME_CORRUPTED;
  if(fdt_open_into(Original,Dtb,fdt_totalsize(Original)+16384)) goto Unload;
  Reserved=fdt_path_offset(Dtb,"/reserved-memory");
  if(Reserved==-FDT_ERR_NOTFOUND) {
    Reserved=fdt_add_subnode(Dtb,0,"reserved-memory");
    if(Reserved<0 || fdt_setprop_u32(Dtb,Reserved,"#address-cells",2) ||
        fdt_setprop_u32(Dtb,Reserved,"#size-cells",2) ||
        fdt_setprop(Dtb,Reserved,"ranges",NULL,0)) goto Unload;
  } else if(Reserved<0 || fdt_address_cells(Dtb,Reserved)!=2 ||
      fdt_size_cells(Dtb,Reserved)!=2) goto Unload;
  Node=fdt_add_subnode(Dtb,Reserved,"pj4edk2-scanout");
  Phandle=1;
  { INT32 N=-1,Depth=0; UINT32 P;
    while((N=fdt_next_node(Dtb,N,&Depth))>=0) {
      P=fdt_get_phandle(Dtb,N);
      if(P>=Phandle) {if(P>=0xfffffffeU) goto Unload;Phandle=P+1;}
    }
    if(N!=-FDT_ERR_NOTFOUND) goto Unload;
  }
  Reg[0]=cpu_to_fdt64(Surface.address);Reg[1]=cpu_to_fdt64(Surface.capacity);
  if(Node<0 || fdt_setprop(Dtb,Node,"reg",Reg,sizeof(Reg)) ||
     fdt_setprop_u32(Dtb,Node,"phandle",Phandle) ||
     fdt_setprop(Dtb,Node,"pocketjs,coherent-scanout",NULL,0)) goto Unload;
  Chosen=fdt_path_offset(Dtb,"/chosen");
  if(Chosen<0 || fdt_setprop_u32(Dtb,Chosen,"pocketjs,boot-display-memory",Phandle) ||
      pj4edk2_linux_fdt_display(Dtb,Cells) ||
      fdt_add_mem_rsv(Dtb,Surface.address,Surface.capacity)) goto Unload;
  Status=gBS->InstallConfigurationTable(&gFdtTableGuid,Dtb);
  if(EFI_ERROR(Status)) goto Unload;
  Installed=TRUE;
  /* No Print()/GOP mode changes: the PJ4EDK2 frame stays visible. */
  Status=gBS->StartImage(Image,NULL,NULL);
  Image=NULL; /* Exit() may have unloaded a returning child. */
 Unload:
  if(Image) gBS->UnloadImage(Image);
 Done:
  if(Installed) {
    EFI_STATUS Restore=gBS->InstallConfigurationTable(&gFdtTableGuid,Original);
    if(EFI_ERROR(Restore)) CpuDeadLoop();
  }
  if(Dtb) FreePages(Dtb,EFI_SIZE_TO_PAGES(fdt_totalsize(Original)+16384));
  if(Path) FreePool(Path);
  if(Options) FreePool(Options);
  return Status;
}
