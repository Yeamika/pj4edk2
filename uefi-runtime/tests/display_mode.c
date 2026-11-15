/* Test native GOP acquisition, same-layout handoff and failure retry. */
#include "../edk2/Display.c"
#define CHECK(x) do {if (!(x)) return __LINE__;} while (0)
EFI_GUID gEfiGraphicsOutputProtocolGuid={0};
static EFI_GRAPHICS_OUTPUT_MODE_INFORMATION info={0,1920,1200,PixelBlueGreenRedReserved8BitPerColor,{0},1920};
static EFI_GRAPHICS_OUTPUT_PROTOCOL_MODE mode={1,0,&info,sizeof(info),0,0};
static EFI_GRAPHICS_OUTPUT_PROTOCOL gop={.Mode=&mode};
static EFI_STATUS located,handoff_status;
static unsigned handoffs;
static UINT32 EFIAPI rotation(PJ4EDK2_DISPLAY_PROTOCOL *This,UINT32 Mode) {return 0;}
static EFI_STATUS EFIAPI handoff(PJ4EDK2_DISPLAY_PROTOCOL *This) {++handoffs;return handoff_status;}
static PJ4EDK2_DISPLAY_PROTOCOL orientation={PJ4EDK2_DISPLAY_REVISION,sizeof(orientation),0,&gop,rotation,handoff};
static BOOLEAN present=TRUE;
VOID EFIAPI FreePool(VOID *P) {}
static EFI_STATUS EFIAPI locate(EFI_GUID *G,VOID *R,VOID **Out) {
 EFI_GUID Guid=PJ4EDK2_DISPLAY_GUID;
 if(G->Data1==Guid.Data1){*Out=&orientation;return present?located:EFI_NOT_FOUND;}
 *Out=&gop;return located;
}
static EFI_BOOT_SERVICES services={.LocateProtocol=locate};
EFI_BOOT_SERVICES *gBS=&services;
int main(void) {
 located=EFI_NOT_FOUND;CHECK(pjb_display_start()==EFI_NOT_FOUND);
 located=EFI_SUCCESS;present=FALSE;
 CHECK(pjb_display_start()==EFI_SUCCESS && !DisplayModeLeased);
 present=TRUE;orientation.revision=1;
 CHECK(pjb_display_start()==EFI_SUCCESS && !DisplayModeLeased);
 orientation.revision=PJ4EDK2_DISPLAY_REVISION;
 CHECK(pjb_display_start()==EFI_SUCCESS && DisplayModeLeased && pjb_display_rotation()==0);
 CHECK(pjb_display_start()==EFI_ALREADY_STARTED);
 handoff_status=EFI_TIMEOUT;
 CHECK(pjb_display_stop()==EFI_TIMEOUT && DisplayModeLeased && handoffs==1 && mode.Mode==0);
 handoff_status=EFI_SUCCESS;
 CHECK(pjb_display_stop()==EFI_SUCCESS && !DisplayModeLeased && handoffs==2 && mode.Mode==0);
 CHECK(pjb_display_rotation()==0 && pjb_display_stop()==EFI_SUCCESS && handoffs==2);
 return 0;
}
