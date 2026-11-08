/* Generic Boot Services hardware bridge; board drivers own all MMIO. */
#include <Uefi.h>
#include <Library/BaseMemoryLib.h>
#include <Library/UefiBootServicesTableLib.h>
#define PJB_EDK2
#include "pocketjs_boot_context.h"

static EFI_STATUS Platform(PJB_PLATFORM_PROTOCOL **Protocol)
{
  PJB_GUID WireGuid=PJB_PLATFORM_GUID;
  EFI_GUID Guid;
  EFI_STATUS Status;
  CopyMem(&Guid,&WireGuid,sizeof(Guid));
  Status=gBS->LocateProtocol(&Guid,NULL,(VOID **)Protocol);
  if (EFI_ERROR(Status)) return Status;
  if (*Protocol==NULL || (*Protocol)->revision!=PJB_PLATFORM_REVISION ||
      (*Protocol)->size<sizeof(**Protocol) || (*Protocol)->read_power==NULL ||
      (*Protocol)->set_quiesced==NULL) return EFI_INCOMPATIBLE_VERSION;
  return EFI_SUCCESS;
}
UINTN pjb_read_power(PJB_POWER_SAMPLE *Sample)
{
  PJB_PLATFORM_PROTOCOL *Protocol;
  EFI_STATUS Status;
  if (Sample==NULL) return EFI_INVALID_PARAMETER;
  ZeroMem(Sample,sizeof(*Sample)); Sample->size=sizeof(*Sample);
  Status=Platform(&Protocol);
  if (!EFI_ERROR(Status)) Status=Protocol->read_power(Protocol,Sample);
  if (EFI_ERROR(Status)) {
    ZeroMem(Sample,sizeof(*Sample)); Sample->size=sizeof(*Sample);
  }
  return Status;
}
UINTN pjb_quiesce_hardware(UINT32 Quiesced)
{
  PJB_PLATFORM_PROTOCOL *Protocol;
  EFI_STATUS Status=Platform(&Protocol);
  /* Other boards may use only standard UEFI protocols. */
  if (Status==EFI_NOT_FOUND) return EFI_SUCCESS;
  if (EFI_ERROR(Status)) return Status;
  return Protocol->set_quiesced(Protocol,Quiesced);
}

static EFI_STATUS Charge(PJB_CHARGE_CONTROL_PROTOCOL **Protocol)
{
  PJB_GUID WireGuid=PJB_CHARGE_CONTROL_GUID;
  EFI_GUID Guid; EFI_STATUS Status;
  CopyMem(&Guid,&WireGuid,sizeof(Guid));
  Status=gBS->LocateProtocol(&Guid,NULL,(VOID **)Protocol);
  if (EFI_ERROR(Status)) return Status;
  if (*Protocol==NULL || (*Protocol)->revision!=PJB_CHARGE_CONTROL_REVISION ||
      (*Protocol)->size<OFFSET_OF(PJB_CHARGE_CONTROL_PROTOCOL,read_requested) || (*Protocol)->read_enabled==NULL ||
      (*Protocol)->set_enabled==NULL) return EFI_INCOMPATIBLE_VERSION;
  return EFI_SUCCESS;
}
UINTN pjb_charge_read(UINT32 *Available, UINT32 *Enabled, UINT32 *Requested)
{
  PJB_CHARGE_CONTROL_PROTOCOL *Protocol; EFI_STATUS Status;
  if (Available==NULL || Enabled==NULL || Requested==NULL) return EFI_INVALID_PARAMETER;
  *Available=0; *Enabled=2; *Requested=2;
  Status=Charge(&Protocol);
  if (EFI_ERROR(Status)) return Status;
  *Available=1;
  if (Protocol->size>=sizeof(*Protocol) && Protocol->read_requested!=NULL) {
    EFI_STATUS RequestStatus=Protocol->read_requested(Protocol,Requested);
    if (EFI_ERROR(RequestStatus) || *Requested>1) *Requested=2;
  }
  Status=Protocol->read_enabled(Protocol,Enabled);
  if (EFI_ERROR(Status) || *Enabled>1) {
    *Enabled=2;
    return EFI_ERROR(Status)?Status:EFI_DEVICE_ERROR;
  }
  return EFI_SUCCESS;
}
UINTN pjb_charge_set(UINT32 Enabled)
{
  PJB_CHARGE_CONTROL_PROTOCOL *Protocol; EFI_STATUS Status;
  if (Enabled>1) return EFI_INVALID_PARAMETER;
  Status=Charge(&Protocol);
  return EFI_ERROR(Status)?Status:Protocol->set_enabled(Protocol,Enabled);
}

UINTN pjb_shutdown(EFI_SYSTEM_TABLE *System)
{
  EFI_STATUS Status,Resume;
  if (System==NULL || System->RuntimeServices==NULL || System->RuntimeServices->ResetSystem==NULL)
    return EFI_UNSUPPORTED;
  Status=pjb_quiesce_hardware(1);
  if (EFI_ERROR(Status)) return Status;
  System->RuntimeServices->ResetSystem(EfiResetShutdown,EFI_SUCCESS,0,NULL);
  /* Successful shutdown cannot return. Restore the UI's hardware on failure. */
  Resume=pjb_quiesce_hardware(0);
  return EFI_ERROR(Resume)?Resume:EFI_DEVICE_ERROR;
}
