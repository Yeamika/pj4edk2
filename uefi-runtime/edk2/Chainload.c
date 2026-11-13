/* Load a real EFI image from a filesystem or a firmware-volume file GUID. */
#include <Uefi.h>
#include <PiDxe.h>
#include <Library/BaseLib.h>
#include <Library/BaseMemoryLib.h>
#include <Library/DevicePathLib.h>
#include <Library/MemoryAllocationLib.h>
#include <Library/UefiBootServicesTableLib.h>
#include <Library/UefiLib.h>
#include <Protocol/FirmwareVolume2.h>
#include <Protocol/LoadedImage.h>
#include <Protocol/SimpleFileSystem.h>
#include "Esp.h"
#define PJB_EDK2
#include "pocketjs_boot_context.h"
UINTN pjb_quiesce_hardware(UINT32 Quiesced);
UINTN pjb_fastboot_before_boot(VOID);
UINTN pjb_display_stop(VOID);

static EFI_STATUS StartChild(EFI_HANDLE Image, const CHAR16 *Options, const PJB_BOOT_DISPLAY *Display)
{
  EFI_STATUS Status;
  PJB_GUID WireGuid=PJB_BOOT_DISPLAY_GUID;
  EFI_GUID Guid;
  VOID *Previous=NULL;
  UINTN Index;
  PJB_BOOT_DISPLAY Copy;
  BOOLEAN Published=FALSE;
  EFI_LOADED_IMAGE_PROTOCOL *Loaded;
  Status = gBS->HandleProtocol(Image, &gEfiLoadedImageProtocolGuid, (VOID **)&Loaded);
  if (EFI_ERROR(Status)) {
    gBS->UnloadImage(Image);
    return Status;
  }
  Loaded->LoadOptions = (VOID *)Options;
  Loaded->LoadOptionsSize = (UINT32)StrSize(Options);
  CopyMem(&Guid,&WireGuid,sizeof(Guid));
  for (Index=0;Index<gST->NumberOfTableEntries;++Index)
    if (CompareGuid(&Guid,&gST->ConfigurationTable[Index].VendorGuid))
      Previous=gST->ConfigurationTable[Index].VendorTable;
  if (Display!=NULL) {
    Copy=*Display;
    Status=gBS->InstallConfigurationTable(&Guid,&Copy);
    if (EFI_ERROR(Status)) { gBS->UnloadImage(Image); return Status; }
    Published=TRUE;
  } else if (Previous!=NULL) {
    Status=gBS->InstallConfigurationTable(&Guid,NULL);
    if (EFI_ERROR(Status)) { gBS->UnloadImage(Image); return Status; }
    Published=TRUE;
  }
  // The caller retains Options until the child exits; application Exit() also
  // unloads the child. No child handle or image memory is used after StartImage.
  Status=pjb_fastboot_before_boot();
  if (EFI_ERROR(Status)) { gBS->UnloadImage(Image); goto Restore; }
  Status=pjb_display_stop();
  if (EFI_ERROR(Status)) { gBS->UnloadImage(Image); goto Restore; }
  Status=pjb_quiesce_hardware(1);
  if (EFI_ERROR(Status)) {
    gBS->UnloadImage(Image);
    goto Restore;
  }
  Status=gBS->StartImage(Image, NULL, NULL);
  /* A returning child has not exited Boot Services. Its handle may already
   * be unloaded by Exit(); touch only our still-resident platform protocol. */
  {
    EFI_STATUS Resume=pjb_quiesce_hardware(0);
    if (!EFI_ERROR(Status) && EFI_ERROR(Resume)) Status=Resume;
  }
Restore:
  /* Child consumers copy before ExitBootServices. The stack record never
   * survives a returning child; restore any pre-existing configuration. */
  if (Published) {
    EFI_STATUS Restored=gBS->InstallConfigurationTable(&Guid,Previous);
    if (EFI_ERROR(Restored)) CpuDeadLoop(); /* Never leave a dangling table. */
  }
  return Status;
}

UINTN pjb_start_entry(const CHAR16 *Path, const CHAR16 *Options, const PJB_BOOT_DISPLAY *Display)
{
  EFI_STATUS Status, Last = EFI_NOT_FOUND;
  EFI_GUID Guid;
  EFI_HANDLE *Handles = NULL, Image;
  UINTN Count, Index;
  EFI_DEVICE_PATH_PROTOCOL *DevicePath;
  MEDIA_FW_VOL_FILEPATH_DEVICE_PATH FileNode;
  BOOLEAN Firmware;

  if (Path == NULL || Options == NULL)
    return EFI_INVALID_PARAMETER;
  Firmware = StrnCmp(Path, L"fv:", 3) == 0;
  if (Firmware) {
    Status = StrToGuid(Path + 3, &Guid);
    if (EFI_ERROR(Status))
      return Status;
    EfiInitializeFwVolDevicepathNode(&FileNode, &Guid);
  } else if (Path[0] != L'\\') {
    return EFI_INVALID_PARAMETER;
  }
  if (!Firmware) {
    EFI_FILE_PROTOCOL *Root;
    EFI_HANDLE Selected;
    Status=pjb_open_esp(&Root,&Selected);
    if (EFI_ERROR(Status)) return Status;
    Status=Root->Close(Root);
    if (EFI_ERROR(Status)) return Status;
    Handles=AllocatePool(sizeof(*Handles));
    if (Handles==NULL) return EFI_OUT_OF_RESOURCES;
    Handles[0]=Selected; Count=1;
  } else {
    Status=gBS->LocateHandleBuffer(ByProtocol,&gEfiFirmwareVolume2ProtocolGuid,NULL,&Count,&Handles);
    if (EFI_ERROR(Status)) return Status;
  }
  for (Index = 0; Index < Count; ++Index) {
    DevicePath = Firmware ? AppendDevicePathNode(DevicePathFromHandle(Handles[Index]),
        (EFI_DEVICE_PATH_PROTOCOL *)&FileNode) : FileDevicePath(Handles[Index], Path);
    if (DevicePath == NULL) {
      Last = EFI_OUT_OF_RESOURCES;
      break;
    }
    Status = gBS->LoadImage(FALSE, gImageHandle, DevicePath, NULL, 0, &Image);
    FreePool(DevicePath);
    if (!EFI_ERROR(Status)) {
      FreePool(Handles);
      return StartChild(Image, Options, Display);
    }
    if (Status == EFI_SECURITY_VIOLATION) {
      // LoadImage may return a handle even when policy blocks execution.
      gBS->UnloadImage(Image);
      Last = Status;
      break;
    }
    if (Status != EFI_NOT_FOUND)
      Last = Status;
  }
  FreePool(Handles);
  return Last;
}
