/* Read-only discovery: one canonical loader per vendor on the unique GPT ESP. */
#include "Esp.h"
#include <Guid/FileInfo.h>
#include <Library/BaseLib.h>
#include <Library/BaseMemoryLib.h>

#define PATH_CHARS 241
#define MAX_DIRECTORIES 256
#define MAX_ENTRIES 32

static BOOLEAN Same(CONST CHAR16 *A, CONST CHAR16 *B)
{
  for (; *A && *B; ++A, ++B) {
    CHAR16 X=*A, Y=*B;
    if (X >= L'a' && X <= L'z') X-=L'a'-L'A';
    if (Y >= L'a' && Y <= L'z') Y-=L'a'-L'A';
    if (X != Y) return FALSE;
  }
  return *A == *B;
}

static BOOLEAN FileExists(EFI_FILE_PROTOCOL *Root, CONST CHAR16 *Path)
{
  EFI_FILE_PROTOCOL *File;
  union { UINT64 Align; UINT8 Bytes[1024]; } Buffer;
  EFI_FILE_INFO *Info=(VOID *)Buffer.Bytes;
  UINTN Size=sizeof(Buffer);
  EFI_STATUS Status=Root->Open(Root, &File, (CHAR16 *)Path, EFI_FILE_MODE_READ, 0);
  if (EFI_ERROR(Status)) return FALSE;
  Status=File->GetInfo(File, &gEfiFileInfoGuid, &Size, Info);
  BOOLEAN Found=!EFI_ERROR(Status) && Size>=SIZE_OF_EFI_FILE_INFO &&
                Size<=sizeof(Buffer) && Info->Size>=SIZE_OF_EFI_FILE_INFO &&
                !(Info->Attribute & EFI_FILE_DIRECTORY) && Info->FileSize>0;
  if (EFI_ERROR(File->Close(File))) Found=FALSE;
  return Found;
}

UINTN pjb_esp_entry_exists(CONST CHAR16 *Path)
{
  EFI_FILE_PROTOCOL *Root;
  EFI_STATUS Status;
  BOOLEAN Found;
  if (Path==NULL || Path[0]!=L'\\') return EFI_INVALID_PARAMETER;
  Status=pjb_open_esp(&Root, NULL);
  if (EFI_ERROR(Status)) return Status;
  Found=FileExists(Root, Path);
  Status=Root->Close(Root);
  return EFI_ERROR(Status) ? Status : (Found ? EFI_SUCCESS : EFI_NOT_FOUND);
}

UINTN pjb_esp_entries(CHAR16 *Paths, UINTN Capacity, UINTN *Used)
{
  EFI_FILE_PROTOCOL *Root, *Directory;
  EFI_STATUS Status;
  UINTN Size, Count=0, Total=0, N, I;
  union { UINT64 Align; UINT8 Bytes[1024]; } Buffer;
  EFI_FILE_INFO *Info=(VOID *)Buffer.Bytes;
  CONST CHAR16 *Default=EFI_REMOVABLE_MEDIA_FILE_NAME, *Loader=Default;
  CHAR16 Path[PATH_CHARS];
  if (Paths==NULL || Used==NULL) return EFI_INVALID_PARAMETER;
  *Used=0;
  // Use the filename for the compiled architecture; BOOT itself is not scanned.
  for (; *Default; ++Default) if (*Default==L'\\') Loader=Default+1;
  Status=pjb_open_esp(&Root, NULL);
  if (EFI_ERROR(Status)) return Status;
  Status=Root->Open(Root, &Directory, L"\\EFI", EFI_FILE_MODE_READ, 0);
  if (EFI_ERROR(Status)) { Root->Close(Root); return Status; }
  while (Count<MAX_DIRECTORIES && Total<MAX_ENTRIES) {
    Size=sizeof(Buffer);
    Status=Directory->Read(Directory, &Size, Info);
    if (EFI_ERROR(Status) || Size==0) break;
    ++Count;
    if (Size>sizeof(Buffer) || Size<SIZE_OF_EFI_FILE_INFO || Info->Size>Size ||
        Info->Size<SIZE_OF_EFI_FILE_INFO) { Status=EFI_VOLUME_CORRUPTED; break; }
    if (!(Info->Attribute & EFI_FILE_DIRECTORY)) continue;
    N=0;
    while (SIZE_OF_EFI_FILE_INFO+(N+1)*sizeof(CHAR16)<=Info->Size && Info->FileName[N]) ++N;
    if (SIZE_OF_EFI_FILE_INFO+(N+1)*sizeof(CHAR16)>Info->Size) { Status=EFI_VOLUME_CORRUPTED; break; }
    if (N==0 || N>96 || Info->FileName[0]==L'.' || Same(Info->FileName,L"BOOT") ||
        Same(Info->FileName,L"pj4edk2") || Same(Info->FileName,L"Tools")) continue;
    for (I=0; I<N; ++I) if (Info->FileName[I]<32 || Info->FileName[I]==L'\\' || Info->FileName[I]==L'/') break;
    if (I!=N) continue;
    StrCpyS(Path,PATH_CHARS,L"\\EFI\\");
    StrCatS(Path,PATH_CHARS,Info->FileName);
    StrCatS(Path,PATH_CHARS,L"\\");
    StrCatS(Path,PATH_CHARS,Loader);
    if (!FileExists(Root,Path)) continue;
    N=StrLen(Path)+1;
    if (*Used+N>Capacity) { Status=EFI_BUFFER_TOO_SMALL; break; }
    CopyMem(Paths+*Used,Path,N*sizeof(CHAR16));
    *Used+=N; ++Total;
  }
  EFI_STATUS Closed=Directory->Close(Directory);
  if (!EFI_ERROR(Status) && EFI_ERROR(Closed)) Status=Closed;
  Closed=Root->Close(Root);
  if (!EFI_ERROR(Status) && EFI_ERROR(Closed)) Status=Closed;
  if (EFI_ERROR(Status)) *Used=0;
  return Status;
}
