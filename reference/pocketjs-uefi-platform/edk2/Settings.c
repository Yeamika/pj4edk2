/* Private UEFI host adapter. Persistent format and policy live in settings.rs. */
#include <Uefi.h>
#include <Guid/FileInfo.h>
#include <Guid/Gpt.h>
#include <Protocol/PartitionInfo.h>
#include <Protocol/SimpleFileSystem.h>
#include <Library/BaseMemoryLib.h>
#include <Library/MemoryAllocationLib.h>
#include <Library/UefiBootServicesTableLib.h>

#define SETTINGS_BYTES 512
static const CHAR16 *Names[] = { L"settings0.bin", L"settings1.bin" };

static EFI_STATUS OpenEsp(EFI_FILE_PROTOCOL **Root)
{
  EFI_HANDLE *Handles, Selected = NULL;
  EFI_SIMPLE_FILE_SYSTEM_PROTOCOL *Fs;
  EFI_PARTITION_INFO_PROTOCOL *Part;
  EFI_STATUS Status;
  UINTN Count, Index, Matches = 0;
  Status = gBS->LocateHandleBuffer(ByProtocol, &gEfiSimpleFileSystemProtocolGuid,
                                 NULL, &Count, &Handles);
  if (EFI_ERROR(Status)) return Status;
  for (Index = 0; Index < Count; ++Index) {
    Status = gBS->HandleProtocol(Handles[Index], &gEfiPartitionInfoProtocolGuid, (VOID **)&Part);
    if (!EFI_ERROR(Status) && Part->Type == PARTITION_TYPE_GPT &&
        CompareGuid(&Part->Info.Gpt.PartitionTypeGUID, &gEfiPartTypeSystemPartGuid)) {
      Selected = Handles[Index]; ++Matches;
    }
  }
  FreePool(Handles);
  // A generic host must not choose an arbitrary disk when there are two ESPs.
  if (Matches != 1) return Matches ? EFI_NO_MAPPING : EFI_NOT_FOUND;
  Status = gBS->HandleProtocol(Selected, &gEfiSimpleFileSystemProtocolGuid, (VOID **)&Fs);
  return EFI_ERROR(Status) ? Status : Fs->OpenVolume(Fs, Root);
}

static EFI_STATUS FileSize(EFI_FILE_PROTOCOL *File, UINT64 *Size)
{
  EFI_FILE_INFO *Info;
  EFI_STATUS Status;
  UINTN Bytes = 0;
  Status = File->GetInfo(File, &gEfiFileInfoGuid, &Bytes, NULL);
  if (Status != EFI_BUFFER_TOO_SMALL) return EFI_ERROR(Status) ? Status : EFI_VOLUME_CORRUPTED;
  if (Bytes < SIZE_OF_EFI_FILE_INFO || Bytes > 4096) return EFI_VOLUME_CORRUPTED;
  Info = AllocatePool(Bytes);
  if (!Info) return EFI_OUT_OF_RESOURCES;
  Status = File->GetInfo(File, &gEfiFileInfoGuid, &Bytes, Info);
  if (!EFI_ERROR(Status)) {
    if (Info->Attribute & EFI_FILE_DIRECTORY) Status = EFI_VOLUME_CORRUPTED;
    else *Size = Info->FileSize;
  }
  FreePool(Info);
  return Status;
}

static EFI_STATUS CloseFile(EFI_FILE_PROTOCOL *File, EFI_STATUS Status)
{
  EFI_STATUS Closed = File->Close(File);
  return EFI_ERROR(Status) ? Status : Closed;
}

static EFI_STATUS Access(UINTN Slot, VOID *Bytes, UINTN Length, BOOLEAN Write)
{
  EFI_FILE_PROTOCOL *Root = NULL, *Directory = NULL, *File = NULL;
  EFI_STATUS Status;
  UINT64 Size = 0, Mode;
  UINTN Done;
  if (Slot > 1 || Bytes == NULL || Length != SETTINGS_BYTES) return EFI_INVALID_PARAMETER;
  Status = OpenEsp(&Root);
  if (EFI_ERROR(Status)) return Status;
  Mode = EFI_FILE_MODE_READ | (Write ? EFI_FILE_MODE_WRITE | EFI_FILE_MODE_CREATE : 0);
  Status = Root->Open(Root, &Directory, L"\\EFI\\pj4edk2", Mode, EFI_FILE_DIRECTORY);
  if (EFI_ERROR(Status)) goto Done;
  Status = Directory->Open(Directory, &File, (CHAR16 *)Names[Slot], Mode, 0);
  if (EFI_ERROR(Status)) goto Done;
  Status = FileSize(File, &Size);
  if (EFI_ERROR(Status)) goto Done;
  if (Size > SETTINGS_BYTES || (!Write && Size != SETTINGS_BYTES)) { Status = EFI_VOLUME_CORRUPTED; goto Done; }
  Done = Length;
  Status = Write ? File->Write(File, &Done, Bytes) : File->Read(File, &Done, Bytes);
  if (!EFI_ERROR(Status) && Done != Length) Status = EFI_DEVICE_ERROR;
  if (!EFI_ERROR(Status) && Write) Status = File->Flush(File);
Done:
  if (File) Status = CloseFile(File, Status);
  if (Directory) {
    if (!EFI_ERROR(Status) && Write) Status = Directory->Flush(Directory);
    Status = CloseFile(Directory, Status);
  }
  if (!EFI_ERROR(Status) && Write) Status = Root->Flush(Root);
  return CloseFile(Root, Status);
}

UINTN pjb_settings_read(UINTN Slot, VOID *Bytes, UINTN Length) { return Access(Slot, Bytes, Length, FALSE); }
UINTN pjb_settings_write(UINTN Slot, const VOID *Bytes, UINTN Length) { return Access(Slot, (VOID *)Bytes, Length, TRUE); }
