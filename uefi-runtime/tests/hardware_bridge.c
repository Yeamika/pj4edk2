/* Execute the actual C bridge and child launch boundary with fake EFI services. */
#include "../edk2/Hardware.c"
#include "../edk2/Chainload.c"
#include "../edk2/Fastboot.c"
#include "../edk2/Display.c"
#define CHECK(x) do { if (!(x)) return __LINE__; } while(0)
static EFI_STATUS locate_status,read_status,stop_status,resume_status,child_status;
static unsigned stopped,resumed,started,unloaded;
static PJB_PLATFORM_PROTOCOL protocol;
static PJB_CHARGE_CONTROL_PROTOCOL charge;
static UINT32 charge_value;
static EFI_STATUS charge_status;
static EFI_LOADED_IMAGE_PROTOCOL loaded;
static PJ4EDK2_FASTBOOT_PROTOCOL fastboot;
static EFI_STATUS fastboot_stop_status;
static UINT32 fastboot_flags;
static unsigned fastboot_starts,fastboot_stops;
static EFI_STATUS display_restore_status;
static unsigned display_restores;
static EFI_STATUS EFIAPI RestoreDisplay(EFI_GRAPHICS_OUTPUT_PROTOCOL *This,UINT32 Mode) {
  ++display_restores;return display_restore_status;
}
static EFI_GRAPHICS_OUTPUT_PROTOCOL display_gop={.SetMode=RestoreDisplay};
static EFI_STATUS EFIAPI FastbootStart(PJ4EDK2_FASTBOOT_PROTOCOL *This) { ++fastboot_starts;return EFI_SUCCESS; }
static EFI_STATUS EFIAPI FastbootStop(PJ4EDK2_FASTBOOT_PROTOCOL *This) { ++fastboot_stops;return fastboot_stop_status; }
static EFI_STATUS EFIAPI FastbootPoll(PJ4EDK2_FASTBOOT_PROTOCOL *This,UINT32 *Flags) { *Flags=fastboot_flags;return EFI_SUCCESS; }
EFI_GUID gEfiLoadedImageProtocolGuid={0};
EFI_HANDLE gImageHandle;
VOID *EFIAPI CopyMem(VOID *d,CONST VOID *s,UINTN n) { UINT8 *a=d;const UINT8 *b=s;while(n--)*a++=*b++;return d; }
VOID *EFIAPI ZeroMem(VOID *d,UINTN n) { UINT8 *a=d;while(n--)*a++=0;return d; }
UINTN EFIAPI StrSize(CONST CHAR16 *s) { UINTN n=0;while(s[n])++n;return (n+1)*2; }
static EFI_STATUS EFIAPI Locate(EFI_GUID *g,VOID *r,VOID **out) {
  EFI_GUID guid=PJ4EDK2_FASTBOOT_GUID;
  if(g->Data1==guid.Data1) *out=&fastboot;
  else *out=(g->Data1==PJB_CHARGE_CONTROL_GUID.data1)?(VOID *)&charge:(VOID *)&protocol;
  return locate_status;
}
static EFI_STATUS EFIAPI Handle(EFI_HANDLE h,EFI_GUID *g,VOID **out) { *out=&loaded;return EFI_SUCCESS; }
static EFI_STATUS EFIAPI Unload(EFI_HANDLE h) { ++unloaded;return EFI_SUCCESS; }
static EFI_STATUS EFIAPI Start(EFI_HANDLE h,UINTN *n,CHAR16 **s) { ++started;return child_status; }
static EFI_CONFIGURATION_TABLE table;
static EFI_SYSTEM_TABLE system_table={.ConfigurationTable=&table};
EFI_SYSTEM_TABLE *gST=&system_table;
static unsigned published;
static EFI_STATUS install_status;
BOOLEAN EFIAPI CompareGuid(CONST GUID *a,CONST GUID *b) { return a->Data1==b->Data1; }
VOID EFIAPI CpuDeadLoop(VOID) { __builtin_trap(); }
static EFI_STATUS EFIAPI Install(EFI_GUID *g,VOID *value) {
  if (install_status) return install_status;
  table.VendorGuid=*g;table.VendorTable=value;system_table.NumberOfTableEntries=value?1:0;
  ++published;return EFI_SUCCESS;
}
static EFI_BOOT_SERVICES bs={.LocateProtocol=Locate,.HandleProtocol=Handle,.UnloadImage=Unload,.StartImage=Start,.InstallConfigurationTable=Install};
EFI_BOOT_SERVICES *gBS=&bs;
static PJB_STATUS PJB_EFIAPI Read(VOID *this,PJB_POWER_SAMPLE *s) {
  if(this!=&protocol)return EFI_INVALID_PARAMETER;
  s->valid=PJB_POWER_PERCENT;s->percent=83;return read_status;
}
static PJB_STATUS PJB_EFIAPI Quiesce(VOID *this,UINT32 q) {
  if(this!=&protocol)return EFI_INVALID_PARAMETER;
  if(q){++stopped;return stop_status;}
  ++resumed;return resume_status;
}
static PJB_STATUS PJB_EFIAPI ReadCharge(VOID *this,UINT32 *e) { *e=charge_value;return charge_status; }
static PJB_STATUS PJB_EFIAPI SetCharge(VOID *this,UINT32 e) { if (!charge_status) charge_value=e;return charge_status; }
static unsigned reset_calls;
static VOID EFIAPI ResetSystemTest(EFI_RESET_TYPE type,EFI_STATUS status,UINTN size,VOID *data) {
  if (type==EfiResetShutdown) ++reset_calls;
}
int main(void) {
  UINT32 available=9,enabled=9,requested=9;
  charge.revision=PJB_CHARGE_CONTROL_REVISION;charge.size=sizeof(charge);
  charge.read_enabled=ReadCharge;charge.set_enabled=SetCharge;
  CHECK(pjb_charge_read(&available,&enabled,&requested)==EFI_SUCCESS && available==1 && enabled==0);
  CHECK(pjb_charge_set(1)==EFI_SUCCESS && charge_value==1);
  CHECK(pjb_charge_read(&available,&enabled,&requested)==EFI_SUCCESS && enabled==1);
  CHECK(pjb_charge_set(2)==EFI_INVALID_PARAMETER && charge_value==1);
  charge_status=EFI_TIMEOUT;
  CHECK(pjb_charge_set(0)==EFI_TIMEOUT && charge_value==1);
  CHECK(pjb_charge_read(&available,&enabled,&requested)==EFI_TIMEOUT && available==1 && enabled==2);
  charge_status=EFI_SUCCESS;charge_value=7;
  CHECK(pjb_charge_read(&available,&enabled,&requested)==EFI_DEVICE_ERROR && enabled==2);
  charge.size=0;
  CHECK(pjb_charge_read(&available,&enabled,&requested)==EFI_INCOMPATIBLE_VERSION && available==0);
  charge.size=sizeof(charge);locate_status=EFI_NOT_FOUND;
  CHECK(pjb_charge_set(1)==EFI_NOT_FOUND);
  CHECK(pjb_charge_read(&available,&enabled,&requested)==EFI_NOT_FOUND && available==0 && enabled==2);
  locate_status=EFI_SUCCESS;

  PJB_POWER_SAMPLE sample={0};
  CONST CHAR16 options[]={0};
  protocol.revision=PJB_PLATFORM_REVISION;protocol.size=sizeof(protocol);
  protocol.read_power=Read;protocol.set_quiesced=Quiesce;
  CHECK(pjb_read_power(&sample)==EFI_SUCCESS && sample.percent==83);
  read_status=EFI_TIMEOUT;
  CHECK(pjb_read_power(&sample)==EFI_TIMEOUT && sample.valid==0 && sample.percent==0);
  locate_status=EFI_NOT_FOUND;
  CHECK(pjb_read_power(&sample)==EFI_NOT_FOUND && sample.valid==0);
  CHECK(pjb_quiesce_hardware(1)==EFI_SUCCESS && stopped==0);
  locate_status=EFI_SUCCESS;protocol.revision=99;
  CHECK(pjb_quiesce_hardware(1)==EFI_INCOMPATIBLE_VERSION);
  protocol.revision=PJB_PLATFORM_REVISION;
  stop_status=EFI_DEVICE_ERROR;
  CHECK(StartChild((EFI_HANDLE)1,options,NULL)==EFI_DEVICE_ERROR && unloaded==1 && started==0 && resumed==0);
  stop_status=EFI_SUCCESS;child_status=EFI_ABORTED;
  CHECK(StartChild((EFI_HANDLE)1,options,NULL)==EFI_ABORTED && started==1 && resumed==1 && unloaded==1);
  child_status=EFI_SUCCESS;resume_status=EFI_TIMEOUT;
  CHECK(StartChild((EFI_HANDLE)1,options,NULL)==EFI_TIMEOUT && resumed==2 && unloaded==1);
  PJB_BOOT_DISPLAY display={.revision=1,.size=sizeof(display),.display_width=1920};
  resume_status=EFI_SUCCESS;
  CHECK(StartChild((EFI_HANDLE)1,options,&display)==EFI_SUCCESS && published==2 && table.VendorTable==NULL);
  PJB_BOOT_DISPLAY previous={0}; table.VendorTable=&previous;system_table.NumberOfTableEntries=1;
  CHECK(StartChild((EFI_HANDLE)1,options,&display)==EFI_SUCCESS && table.VendorTable==&previous);
  stop_status=EFI_TIMEOUT;
  CHECK(StartChild((EFI_HANDLE)1,options,&display)==EFI_TIMEOUT && table.VendorTable==&previous);
  stop_status=EFI_SUCCESS; install_status=EFI_OUT_OF_RESOURCES;
  unsigned before=started;
  CHECK(StartChild((EFI_HANDLE)1,options,&display)==EFI_OUT_OF_RESOURCES && started==before);
  install_status=EFI_SUCCESS;
  CHECK(StartChild((EFI_HANDLE)1,options,NULL)==EFI_SUCCESS && table.VendorTable==&previous);
  EFI_RUNTIME_SERVICES rt={.ResetSystem=ResetSystemTest};
  EFI_SYSTEM_TABLE system={.RuntimeServices=&rt};
  stopped=resumed=0;stop_status=EFI_DEVICE_ERROR;resume_status=EFI_SUCCESS;
  CHECK(pjb_shutdown(&system)==EFI_DEVICE_ERROR && reset_calls==0 && stopped==1 && resumed==0);
  stop_status=EFI_SUCCESS;
  CHECK(pjb_shutdown(&system)==EFI_DEVICE_ERROR && reset_calls==1 && resumed==1);
  CHECK(pjb_shutdown(NULL)==EFI_UNSUPPORTED);

  /* A failed display restore must block child execution before hardware
   * quiesce; retain the mode lease so the caller can retry safely. */
  unsigned display_children=started,display_quiesced=stopped;
  DisplayGop=&display_gop;DisplayModeLeased=TRUE;
  display_restore_status=EFI_TIMEOUT;
  CHECK(StartChild((EFI_HANDLE)1,options,NULL)==EFI_TIMEOUT &&
        started==display_children && stopped==display_quiesced && DisplayModeLeased);
  display_restore_status=EFI_SUCCESS;
  CHECK(StartChild((EFI_HANDLE)1,options,NULL)==EFI_SUCCESS &&
        started==display_children+1 && !DisplayModeLeased && display_restores==2);

  /* A live transfer or failed USB release must stop both child launch and
   * power actions before the board is quiesced or ResetSystem is called. */
  fastboot=(PJ4EDK2_FASTBOOT_PROTOCOL){1,sizeof(fastboot),FastbootStart,FastbootPoll,FastbootStop};
  CHECK(pjb_fastboot_start()==EFI_SUCCESS && fastboot_starts==1);
  fastboot_flags=PJ4EDK2_FASTBOOT_BUSY;
  UINT32 flags;
  CHECK(pjb_fastboot_poll(&flags)==EFI_SUCCESS);
  unsigned children=started,quiesced=stopped,resets=reset_calls;
  CHECK(StartChild((EFI_HANDLE)1,options,NULL)==EFI_NOT_READY && started==children && stopped==quiesced);
  CHECK(pjb_shutdown(&system)==EFI_NOT_READY && reset_calls==resets && stopped==quiesced && !fastboot_stops);
  fastboot_flags=0;CHECK(pjb_fastboot_poll(&flags)==EFI_SUCCESS);
  fastboot_stop_status=EFI_DEVICE_ERROR;
  CHECK(StartChild((EFI_HANDLE)1,options,NULL)==EFI_DEVICE_ERROR && started==children && stopped==quiesced);
  CHECK(pjb_shutdown(&system)==EFI_DEVICE_ERROR && reset_calls==resets && stopped==quiesced && Active);
  fastboot_stop_status=EFI_SUCCESS;
  CHECK(StartChild((EFI_HANDLE)1,options,NULL)==EFI_SUCCESS && started==children+1 && !Active);
  CHECK(pjb_fastboot_start()==EFI_SUCCESS && fastboot_starts==2);
  CHECK(pjb_shutdown(&system)==EFI_DEVICE_ERROR && reset_calls==resets+1 && Active && fastboot_starts==3);
  CHECK(pjb_fastboot_stop()==EFI_SUCCESS);
  return 0;
}
