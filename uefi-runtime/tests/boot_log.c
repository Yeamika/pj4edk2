/* Test the actual session log with no SerialIo protocol or Boot Services. */
#define main RuntimeMain
#include "../edk2/Entry.c"
#undef main
#define CHECK(x) do { if (!(x)) return __LINE__; } while (0)
EFI_BOOT_SERVICES *gBS;
int main(void) {
  unsigned char block[4096],got[8192];
  UINTN i;
  CHECK(pjb_log_read(got,sizeof(got))==0);
  for(i=0;i<sizeof(block);++i)block[i]='A';
  pjb_log(block,sizeof(block));
  for(i=0;i<sizeof(block);++i)block[i]='B';
  pjb_log(block,sizeof(block));
  CHECK(pjb_log_read(got,sizeof(got))==8192);
  CHECK(got[0]=='A' && got[4093]=='A' && got[4094]=='\n' && got[4095]=='B' && got[8191]=='\n');
  CHECK(pjb_log_read(got,8)==8 && got[0]=='B' && got[7]=='\n');
  CHECK(pjb_log_read(NULL,8)==0);
  pjb_log(NULL,12);pjb_log(block,4097);
  CHECK(pjb_log_read(got,8)==8 && got[0]=='B' && got[7]=='\n');
  return 0;
}
