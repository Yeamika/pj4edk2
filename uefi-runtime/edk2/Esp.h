/* Private host helpers; no board-specific storage addresses. */
#ifndef PJB_ESP_H
#define PJB_ESP_H
#include <Uefi.h>
#include <Protocol/SimpleFileSystem.h>
EFI_STATUS pjb_open_esp(EFI_FILE_PROTOCOL **Root, EFI_HANDLE *Handle);
#endif
