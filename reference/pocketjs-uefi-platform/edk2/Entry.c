/* PocketJS needs its own stack: the Polaris firmware stack is only 256 KiB. */
#include <Uefi.h>
#include <Protocol/SerialIo.h>
#include <Library/BaseLib.h>
#include <Library/UefiBootServicesTableLib.h>
#include <Library/ShellCEntryLib.h>

/* Recent application log, retained across child EFI returns during this boot.
 * Bounded and independent of optional serial output. Single UI-thread owner. */
static unsigned char BootLog[8192];
static UINTN LogHead,LogCount;
static void LogByte(unsigned char Byte) {
  BootLog[LogHead]=Byte; LogHead=(LogHead+1)%sizeof(BootLog);
  if (LogCount<sizeof(BootLog)) ++LogCount;
}
UINTN pjb_log_read(unsigned char *Output, UINTN Capacity) {
  UINTN I,Count=LogCount<Capacity?LogCount:Capacity;
  if (Output==NULL) return 0;
  for (I=0;I<Count;++I) Output[I]=BootLog[(LogHead+sizeof(BootLog)-Count+I)%sizeof(BootLog)];
  return Count;
}

/* Optional standard UEFI diagnostics; no board addresses in the host. */
void pjb_log(const unsigned char *Text, UINTN Length)
{
  static EFI_GUID Guid = EFI_SERIAL_IO_PROTOCOL_GUID;
  EFI_SERIAL_IO_PROTOCOL *Serial = NULL;
  UINTN Count,I;
  if (Text == NULL || Length > 4096) return;
  for (I=0;I<Length;++I) LogByte(Text[I]);
  LogByte('\n');
  if (gBS == NULL) return;
  if (EFI_ERROR(gBS->LocateProtocol(&Guid, NULL, (VOID **)&Serial)) || Serial == NULL) return;
  Count = Length;
  Serial->Write(Serial, &Count, (VOID *)Text);
  Count = 2;
  Serial->Write(Serial, &Count, "\r\n");
}

extern int pjb_main(EFI_HANDLE image, EFI_SYSTEM_TABLE *table);

typedef struct {
  BASE_LIBRARY_JUMP_BUFFER Return;
  volatile EFI_STATUS Result;
} PJB_CONTEXT;

static VOID EFIAPI PjbOnStack(VOID *Context, VOID *Unused)
{
  PJB_CONTEXT *Run = Context;
  (void)Unused;
  CHAR16 *Arguments[] = { L"PocketJsBoot", NULL };
  Run->Result = (EFI_STATUS)ShellAppMain(1, Arguments);
  LongJump(&Run->Return, 1);
}

int main(int argc, char **argv)
{
  (void)argc;
  (void)argv;
  pjb_log((const unsigned char *)"PJB_MAIN", 8);
  return pjb_main(gImageHandle, gST);
}

// LibC's stock ShellCEntryLib refuses to run without ShellParameters. BDS
// launches have no shell: call LibC's real startup directly with our own argv.
EFI_STATUS EFIAPI PjbUefiMain(EFI_HANDLE Image, EFI_SYSTEM_TABLE *Table)
{
  const UINTN Pages = EFI_SIZE_TO_PAGES(2 * 1024 * 1024);
  EFI_PHYSICAL_ADDRESS Stack = 0;
  PJB_CONTEXT Run;
  pjb_log((const unsigned char *)"PJB_ENTRY", 9);
  (void)Image;
  (void)Table;
  if (EFI_ERROR(gBS->AllocatePages(AllocateAnyPages, EfiBootServicesData, Pages, &Stack)))
    return EFI_OUT_OF_RESOURCES;
  Run.Result = EFI_ABORTED;
  if (SetJump(&Run.Return) == 0)
    SwitchStack(PjbOnStack, &Run, NULL, (VOID *)(UINTN)(Stack + EFI_PAGES_TO_SIZE(Pages)));
  gBS->FreePages(Stack, Pages);
  return Run.Result;
}
