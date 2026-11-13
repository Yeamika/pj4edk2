/* Exercise production directory scanning with standard UEFI protocol mocks. */
#define main settings_adapter_tests
#include "settings_file.c"
#undef main
#include "../edk2/Discovery.c"
UINTN EFIAPI StrLen(CONST CHAR16 *s) { UINTN n=0;while(s[n])++n;return n; }
RETURN_STATUS EFIAPI StrCpyS(CHAR16 *d,UINTN cap,CONST CHAR16 *s) {
  UINTN n=StrLen(s)+1;if(n>cap)return RETURN_BUFFER_TOO_SMALL;
  copy(d,s,n*sizeof(CHAR16));return RETURN_SUCCESS;
}
RETURN_STATUS EFIAPI StrCatS(CHAR16 *d,UINTN cap,CONST CHAR16 *s) {
  UINTN n=StrLen(d);return StrCpyS(d+n,cap-n,s);
}
VOID *EFIAPI CopyMem(VOID *d,CONST VOID *s,UINTN n) {copy(d,s,n);return d;}
static UINTN cursor;
static BOOLEAN corrupt;
static EFI_STATUS EFIAPI scan_read(EFI_FILE_PROTOCOL *f,UINTN *n,VOID *out) {
  CONST CHAR16 *names[]={L"BOOT",L"pj4edk2",L"Tools",L"Android",L"Debian"};
  if(cursor==5){*n=0;return EFI_SUCCESS;}
  EFI_FILE_INFO *i=out;
  i->Size=SIZE_OF_EFI_FILE_INFO+(StrLen(names[cursor])+1)*sizeof(CHAR16);
  i->Attribute=EFI_FILE_DIRECTORY;
  StrCpyS(i->FileName,32,names[cursor++]);
  *n=corrupt ? 1 : i->Size;
  return EFI_SUCCESS;
}
static EFI_STATUS EFIAPI scan_open(EFI_FILE_PROTOCOL *self,EFI_FILE_PROTOCOL **out,CHAR16 *path,UINT64 mode,UINT64 attr) {
  *out=Same(path,L"\\EFI") ? &directory : &file;return EFI_SUCCESS;
}
static EFI_STATUS EFIAPI scan_info(EFI_FILE_PROTOCOL *f,EFI_GUID *g,UINTN *n,VOID *out) {
  EFI_FILE_INFO *i=out;i->Size=SIZE_OF_EFI_FILE_INFO+sizeof(CHAR16);
  i->Attribute=0;i->FileSize=32;*n=i->Size;return EFI_SUCCESS;
}
int main(void) {
  services.LocateHandleBuffer=locate;services.HandleProtocol=protocol;
  part.Type=PARTITION_TYPE_GPT;part.Info.Gpt.PartitionTypeGUID=gEfiPartTypeSystemPartGuid;
  fs.OpenVolume=volume;
  root.Open=scan_open;root.Close=directory.Close=file.Close=close_file;
  directory.Read=scan_read;file.GetInfo=scan_info;
  CHAR16 paths[1024];UINTN used=0;
  CHECK(pjb_esp_entries(paths,1024,&used)==EFI_SUCCESS && used>0);
  CHECK(Same(paths,L"\\EFI\\Android\\BOOTX64.EFI"));
  CHECK(Same(paths+StrLen(paths)+1,L"\\EFI\\Debian\\BOOTX64.EFI"));
  CHECK(pjb_esp_entry_exists(paths)==EFI_SUCCESS);
  count=2;CHECK(pjb_esp_entries(paths,1024,&used)==EFI_NO_MAPPING && used==0);
  count=0;CHECK(pjb_esp_entries(paths,1024,&used)==EFI_NOT_FOUND);count=1;
  cursor=0;CHECK(pjb_esp_entries(paths,1,&used)==EFI_BUFFER_TOO_SMALL && used==0);
  cursor=0;corrupt=TRUE;CHECK(pjb_esp_entries(paths,1024,&used)==EFI_VOLUME_CORRUPTED && used==0);
  return 0;
}
