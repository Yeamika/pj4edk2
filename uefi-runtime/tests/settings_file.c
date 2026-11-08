/* Run the actual UEFI file adapter against failing standard protocol callbacks. */
#include "../edk2/Settings.c"
#define CHECK(x) do { if (!(x)) return __LINE__; } while (0)
EFI_GUID gEfiSimpleFileSystemProtocolGuid={1};
EFI_GUID gEfiPartitionInfoProtocolGuid={2};
EFI_GUID gEfiFileInfoGuid={3};
EFI_GUID gEfiPartTypeSystemPartGuid=EFI_PART_TYPE_EFI_SYSTEM_PART_GUID;
static EFI_BOOT_SERVICES services;
EFI_BOOT_SERVICES *gBS=&services;
static EFI_SIMPLE_FILE_SYSTEM_PROTOCOL fs;
static EFI_PARTITION_INFO_PROTOCOL part;
static EFI_FILE_PROTOCOL root, directory, file;
static EFI_HANDLE handles[2]={(VOID *)1,(VOID *)2};
static UINT8 bytes[512], info_pool[4096];
static UINTN size, count=1, closed, writes, flushes, fail;
static BOOLEAN exists, writable=TRUE;
VOID *EFIAPI AllocatePool(UINTN n) { return n<=sizeof(info_pool)?info_pool:NULL; }
VOID EFIAPI FreePool(VOID *p) {}
static void copy(VOID *d, CONST VOID *s, UINTN n) { for (UINTN i=0;i<n;++i) ((UINT8 *)d)[i]=((CONST UINT8 *)s)[i]; }
BOOLEAN EFIAPI CompareGuid(CONST GUID *a, CONST GUID *b) {
  for (UINTN i=0;i<sizeof(*a);++i) if (((CONST UINT8 *)a)[i]!=((CONST UINT8 *)b)[i]) return FALSE;
  return TRUE;
}
static EFI_STATUS EFIAPI locate(EFI_LOCATE_SEARCH_TYPE t, EFI_GUID *g, VOID *key, UINTN *n, EFI_HANDLE **h) {
  *n=count;*h=handles;return EFI_SUCCESS;
}
static EFI_STATUS EFIAPI protocol(EFI_HANDLE h, EFI_GUID *g, VOID **out) {
  *out=CompareGuid(g,&gEfiPartitionInfoProtocolGuid)?(VOID *)&part:(VOID *)&fs;return EFI_SUCCESS;
}
static EFI_STATUS EFIAPI volume(EFI_SIMPLE_FILE_SYSTEM_PROTOCOL *f, EFI_FILE_PROTOCOL **out) { *out=&root;return EFI_SUCCESS; }
static EFI_STATUS EFIAPI open_file(EFI_FILE_PROTOCOL *self, EFI_FILE_PROTOCOL **out, CHAR16 *path, UINT64 mode, UINT64 attr) {
  if (self==&root) { *out=&directory; return EFI_SUCCESS; }
  if (!exists && !(mode&EFI_FILE_MODE_CREATE)) return EFI_NOT_FOUND;
  if (mode&EFI_FILE_MODE_WRITE && !writable) return EFI_WRITE_PROTECTED;
  exists=TRUE;*out=&file;return EFI_SUCCESS;
}
static EFI_STATUS EFIAPI close_file(EFI_FILE_PROTOCOL *f) { ++closed;return fail==4?EFI_DEVICE_ERROR:EFI_SUCCESS; }
static EFI_STATUS EFIAPI info(EFI_FILE_PROTOCOL *f, EFI_GUID *g, UINTN *n, VOID *out) {
  if (*n<sizeof(EFI_FILE_INFO)) { *n=sizeof(EFI_FILE_INFO);return EFI_BUFFER_TOO_SMALL; }
  EFI_FILE_INFO *i=out;i->Attribute=0;i->FileSize=size;return EFI_SUCCESS;
}
static EFI_STATUS EFIAPI read_file(EFI_FILE_PROTOCOL *f, UINTN *n, VOID *out) {
  if (fail==5) --*n;
  copy(out,bytes,*n);return EFI_SUCCESS;
}
static EFI_STATUS EFIAPI write_file(EFI_FILE_PROTOCOL *f, UINTN *n, VOID *in) {
  ++writes;
  if (fail==1) return EFI_WRITE_PROTECTED;
  if (fail==2) --*n;
  size=*n;copy(bytes,in,*n);return EFI_SUCCESS;
}
static EFI_STATUS EFIAPI flush(EFI_FILE_PROTOCOL *f) { ++flushes;return fail==3?EFI_DEVICE_ERROR:EFI_SUCCESS; }
int main(void) {
  UINT8 input[512], output[512];
  services.LocateHandleBuffer=locate;services.HandleProtocol=protocol;
  part.Type=PARTITION_TYPE_GPT;part.Info.Gpt.PartitionTypeGUID=gEfiPartTypeSystemPartGuid;
  fs.OpenVolume=volume;
  root.Open=directory.Open=open_file;root.Close=directory.Close=file.Close=close_file;
  root.Flush=directory.Flush=file.Flush=flush;file.GetInfo=info;file.Read=read_file;file.Write=write_file;
  for (UINTN i=0;i<512;++i) input[i]=(UINT8)(i*7);
  CHECK(pjb_settings_read(0,output,512)==EFI_NOT_FOUND && closed==2);
  CHECK(pjb_settings_write(1,input,512)==EFI_SUCCESS && size==512 && writes==1 && flushes==3);
  CHECK(pjb_settings_read(1,output,512)==EFI_SUCCESS);
  for (UINTN i=0;i<512;++i) CHECK(input[i]==output[i]);
  count=2;CHECK(pjb_settings_write(1,input,512)==EFI_NO_MAPPING && writes==1);
  count=0;CHECK(pjb_settings_read(0,output,512)==EFI_NOT_FOUND);count=1;
  CHECK(pjb_settings_write(2,input,512)==EFI_INVALID_PARAMETER);
  CHECK(pjb_settings_write(0,input,511)==EFI_INVALID_PARAMETER);
  size=99;CHECK(pjb_settings_read(0,output,512)==EFI_VOLUME_CORRUPTED);
  CHECK(pjb_settings_write(0,input,512)==EFI_SUCCESS && size==512);
  size=513;CHECK(pjb_settings_write(0,input,512)==EFI_VOLUME_CORRUPTED);size=512;
  writable=FALSE;CHECK(pjb_settings_write(0,input,512)==EFI_WRITE_PROTECTED);writable=TRUE;
  for (fail=1;fail<=4;++fail) { size=512; CHECK(EFI_ERROR(pjb_settings_write(0,input,512))); }
  fail=5;size=512;CHECK(pjb_settings_read(0,output,512)==EFI_DEVICE_ERROR);
  return 0;
}
