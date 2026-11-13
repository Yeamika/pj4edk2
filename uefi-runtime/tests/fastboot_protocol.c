/* Compile the real platform-independent bridge against a fake EFI backend. */
#include "../edk2/Fastboot.c"
#define CHECK(x) do { if(!(x)) return __LINE__; } while(0)
static EFI_BOOT_SERVICES Services;
EFI_BOOT_SERVICES *gBS=&Services;
static EFI_STATUS located=EFI_SUCCESS, started=EFI_SUCCESS, stopped=EFI_SUCCESS, polled=EFI_SUCCESS;
static UINT32 flags;
static unsigned starts,stops,polls;
static EFI_STATUS EFIAPI start(PJ4EDK2_FASTBOOT_PROTOCOL *This) { ++starts;return started; }
static EFI_STATUS EFIAPI stop(PJ4EDK2_FASTBOOT_PROTOCOL *This) { ++stops;return stopped; }
static EFI_STATUS EFIAPI poll(PJ4EDK2_FASTBOOT_PROTOCOL *This,UINT32 *Flags) { ++polls;*Flags=flags;return polled; }
static PJ4EDK2_FASTBOOT_PROTOCOL Protocol={1,sizeof(Protocol),start,poll,stop};
static EFI_STATUS EFIAPI locate(EFI_GUID *Guid,VOID *Registration,VOID **Out) {
    *Out=&Protocol;return located;
}
int main(void) {
    UINT32 value=99;Services.LocateProtocol=locate;
    located=EFI_NOT_FOUND;CHECK(pjb_fastboot_start()==EFI_NOT_FOUND);
    CHECK(pjb_fastboot_poll(&value)==EFI_NOT_FOUND && value==0);
    CHECK(pjb_fastboot_before_boot()==EFI_SUCCESS && !starts && !polls);
    located=EFI_SUCCESS;Protocol.revision=0;
    CHECK(pjb_fastboot_start()==EFI_INCOMPATIBLE_VERSION && !starts);
    Protocol.revision=1;Protocol.size=4;
    CHECK(pjb_fastboot_start()==EFI_INCOMPATIBLE_VERSION && !starts);
    Protocol.size=sizeof(Protocol);started=EFI_DEVICE_ERROR;
    CHECK(pjb_fastboot_start()==EFI_DEVICE_ERROR && starts==1);
    started=EFI_SUCCESS;CHECK(pjb_fastboot_start()==EFI_SUCCESS && starts==2);
    CHECK(pjb_fastboot_start()==EFI_SUCCESS && starts==2);
    flags=PJ4EDK2_FASTBOOT_BUSY;CHECK(pjb_fastboot_poll(&value)==EFI_SUCCESS && value==flags);
    CHECK(pjb_fastboot_before_boot()==EFI_NOT_READY && !stops);
    flags=0;CHECK(pjb_fastboot_poll(&value)==EFI_SUCCESS);
    stopped=EFI_DEVICE_ERROR;CHECK(pjb_fastboot_before_boot()==EFI_DEVICE_ERROR && Active);
    stopped=EFI_SUCCESS;CHECK(pjb_fastboot_before_boot()==EFI_SUCCESS && !Active);
    CHECK(pjb_fastboot_start()==EFI_SUCCESS);
    polled=EFI_DEVICE_ERROR;CHECK(pjb_fastboot_poll(&value)==EFI_DEVICE_ERROR && value==0 && !Active);
    CHECK(pjb_fastboot_stop()==EFI_SUCCESS);
    return 0;
}
