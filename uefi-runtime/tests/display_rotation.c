/* Real bridge: optional accelerator errors, retry and child lifetime. */
#include "../edk2/Display.c"
#define CHECK(x) do {if (!(x)) return __LINE__;} while(0)
EFI_GUID gEfiGraphicsOutputProtocolGuid={0};
VOID EFIAPI FreePool(VOID *P) {}
static EFI_STATUS result,stopping;
static unsigned rotations,stops;
static BOOLEAN present=TRUE;
static EFI_STATUS EFIAPI rotate(PJ4EDK2_ROTATE_PROTOCOL *P,CONST PJ4EDK2_ROTATE_REQUEST *R) {
  ++rotations;
  if(R->width!=2160 || R->height!=1080 || R->source_stride!=8640 ||
     R->x!=3 || R->y!=4 || R->damage_width!=1000 || R->damage_height!=800)
    return EFI_INVALID_PARAMETER;
  return result;
}
static EFI_STATUS EFIAPI stop(PJ4EDK2_ROTATE_PROTOCOL *P) {++stops;return stopping;}
static PJ4EDK2_ROTATE_PROTOCOL provider={PJ4EDK2_ROTATE_REVISION,sizeof(provider),rotate,stop};
static EFI_STATUS EFIAPI locate(EFI_GUID *G,VOID *R,VOID **P) {
  *P=&provider;return present?EFI_SUCCESS:EFI_NOT_FOUND;
}
static EFI_BOOT_SERVICES services={.LocateProtocol=locate};
EFI_BOOT_SERVICES *gBS=&services;
static UINTN invoke(VOID) {return pjb_display_rotate((VOID *)1,9331200,(VOID *)2,9331200,4320,2160,1080,3,4,1000,800);}
int main(void) {
  present=FALSE;CHECK(invoke()==EFI_UNSUPPORTED && !rotations);
  present=TRUE;CHECK(invoke()==EFI_UNSUPPORTED && !rotations);
  CHECK(pjb_display_stop()==EFI_SUCCESS);
  provider.revision=0;CHECK(invoke()==EFI_UNSUPPORTED && !rotations);
  CHECK(pjb_display_stop()==EFI_SUCCESS);
  provider.revision=PJ4EDK2_ROTATE_REVISION;
  result=EFI_NOT_READY;CHECK(invoke()==EFI_NOT_READY && rotations==1 && !RotationDisabled);
  result=EFI_SUCCESS;CHECK(invoke()==EFI_SUCCESS && rotations==2 && DisplayRotator==&provider);
  result=EFI_DEVICE_ERROR;CHECK(invoke()==EFI_DEVICE_ERROR && rotations==3 && RotationDisabled);
  CHECK(invoke()==EFI_UNSUPPORTED && rotations==3);
  // No orientation/mode lease exists, but DMA resources must still stop.
  CHECK(!DisplayModeLeased);
  stopping=EFI_TIMEOUT;CHECK(pjb_display_stop()==EFI_TIMEOUT && stops==1 && DisplayRotator==&provider);
  stopping=EFI_SUCCESS;CHECK(pjb_display_stop()==EFI_SUCCESS && stops==2 && !DisplayRotator && !RotationDisabled);
  result=EFI_SUCCESS;CHECK(invoke()==EFI_SUCCESS && rotations==4);
  CHECK(pjb_display_stop()==EFI_SUCCESS && stops==3);
  return 0;
}
