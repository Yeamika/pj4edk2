/* SPDX-License-Identifier: MIT */
#include <Uefi.h>
#include <Library/UefiBootServicesTableLib.h>
#include "pj4edk2_fastboot.h"

static PJ4EDK2_FASTBOOT_PROTOCOL *Backend;
static BOOLEAN Busy, Active;
UINTN pjb_fastboot_start(VOID) {
    EFI_GUID Guid=PJ4EDK2_FASTBOOT_GUID;
    EFI_STATUS Status;
    if(Active) return EFI_SUCCESS;
    Backend=NULL;Busy=FALSE;
    Status=gBS->LocateProtocol(&Guid,NULL,(VOID **)&Backend);
    if(EFI_ERROR(Status)) { Backend=NULL;return Status; }
    if(Backend==NULL || Backend->revision!=PJ4EDK2_FASTBOOT_REVISION ||
       Backend->size<sizeof(*Backend) || !Backend->start || !Backend->poll || !Backend->stop) {
        Backend=NULL;return EFI_INCOMPATIBLE_VERSION;
    }
    Status=Backend->start(Backend);
    Active=!EFI_ERROR(Status);
    if(!Active) Backend=NULL;
    return Status;
}
UINTN pjb_fastboot_stop(VOID) {
    EFI_STATUS Status=EFI_SUCCESS;
    if(Active && Backend!=NULL) Status=Backend->stop(Backend);
    /* Never forget a backend which failed to release USB ownership. */
    if(!EFI_ERROR(Status)) { Active=FALSE;Busy=FALSE;Backend=NULL; }
    return Status;
}
UINTN pjb_fastboot_poll(UINT32 *Flags) {
    EFI_STATUS Status;
    if(Flags==NULL) return EFI_INVALID_PARAMETER;
    *Flags=0;
    if(!Active || !Backend) return EFI_NOT_FOUND;
    Status=Backend->poll(Backend,Flags);
    if(EFI_ERROR(Status)) { pjb_fastboot_stop();*Flags=0;return Status; }
    Busy=(*Flags&PJ4EDK2_FASTBOOT_BUSY)!=0;
    return EFI_SUCCESS;
}
UINTN pjb_fastboot_before_boot(VOID) {
    if(Busy) return EFI_NOT_READY;
    return pjb_fastboot_stop();
}
