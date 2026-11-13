/* Test the real mode lease, including a partially failed SetMode rollback. */
#include "../edk2/Display.c"
#define CHECK(x) do {if (!(x)) return __LINE__;} while (0)
EFI_GUID gEfiGraphicsOutputProtocolGuid={0};
static EFI_GRAPHICS_OUTPUT_MODE_INFORMATION info={0,1200,1920,PixelBitMask,{0},1200};
static EFI_GRAPHICS_OUTPUT_MODE_INFORMATION queried;
static EFI_GRAPHICS_OUTPUT_PROTOCOL_MODE mode={2,0,&info,sizeof(info),0,0};
static EFI_STATUS located, select_status, restore_status;
static unsigned sets, frees;
static EFI_STATUS EFIAPI query(EFI_GRAPHICS_OUTPUT_PROTOCOL *G,UINT32 M,UINTN *S,EFI_GRAPHICS_OUTPUT_MODE_INFORMATION **I) {
  queried=info;queried.PixelFormat=M?PixelBlueGreenRedReserved8BitPerColor:PixelBitMask;
  if (M) {queried.HorizontalResolution=1920;queried.VerticalResolution=1200;queried.PixelsPerScanLine=1920;}
  *I=&queried;*S=sizeof(queried);return EFI_SUCCESS;
}
static EFI_STATUS EFIAPI set(EFI_GRAPHICS_OUTPUT_PROTOCOL *G,UINT32 M) {
  ++sets;mode.Mode=M;return M?select_status:restore_status;
}
static EFI_GRAPHICS_OUTPUT_PROTOCOL gop={query,set,NULL,&mode};
static UINT32 EFIAPI rotation(PJ4EDK2_DISPLAY_PROTOCOL *This,UINT32 Mode) {return Mode?90:0;}
static PJ4EDK2_DISPLAY_PROTOCOL orientation={1,sizeof(orientation),0,&gop,rotation};
static BOOLEAN orientation_present=TRUE;
VOID EFIAPI FreePool(VOID *P) {++frees;}
static EFI_STATUS EFIAPI locate(EFI_GUID *G,VOID *R,VOID **Out) {
  EFI_GUID Guid=PJ4EDK2_DISPLAY_GUID;
  if (G->Data1==Guid.Data1) {*Out=&orientation;return orientation_present?located:EFI_NOT_FOUND;}
  *Out=&gop;return located;
}
static EFI_BOOT_SERVICES services={.LocateProtocol=locate};
EFI_BOOT_SERVICES *gBS=&services;
int main(void) {
  located=EFI_NOT_FOUND;CHECK(pjb_display_start()==EFI_NOT_FOUND && !sets);
  located=EFI_SUCCESS;mode.MaxMode=1;
  CHECK(pjb_display_start()==EFI_SUCCESS && !sets && !DisplayModeLeased);
  mode.MaxMode=2;
  orientation_present=FALSE;
  CHECK(pjb_display_start()==EFI_SUCCESS && !sets && !DisplayModeLeased);
  orientation_present=TRUE;
  CHECK(pjb_display_start()==EFI_SUCCESS && mode.Mode==1 && DisplayModeLeased);
  CHECK(pjb_display_rotation()==90);
  CHECK(pjb_display_start()==EFI_ALREADY_STARTED);
  restore_status=EFI_TIMEOUT;
  CHECK(pjb_display_stop()==EFI_TIMEOUT && DisplayModeLeased);
  restore_status=EFI_SUCCESS;
  CHECK(pjb_display_stop()==EFI_SUCCESS && mode.Mode==0 && !DisplayModeLeased);
  CHECK(pjb_display_rotation()==0);
  select_status=EFI_DEVICE_ERROR;
  CHECK(pjb_display_start()==EFI_DEVICE_ERROR && mode.Mode==0 && !DisplayModeLeased);
  restore_status=EFI_TIMEOUT;
  CHECK(pjb_display_start()==EFI_TIMEOUT && DisplayModeLeased);
  restore_status=EFI_SUCCESS;CHECK(pjb_display_stop()==EFI_SUCCESS);
  CHECK(frees>=7);
  /* The console selected native mode before the runtime started. Still
   * restore the explicit compatibility mode and transform native touch. */
  select_status=EFI_SUCCESS;mode.Mode=1;info.PixelFormat=PixelBlueGreenRedReserved8BitPerColor;
  info.HorizontalResolution=1920;info.VerticalResolution=1200;
  CHECK(pjb_display_start()==EFI_SUCCESS && DisplayModeLeased && pjb_display_rotation()==90);
  CHECK(pjb_display_stop()==EFI_SUCCESS && mode.Mode==0 && pjb_display_rotation()==0);
  return 0;
}
