/* SPDX-License-Identifier: BSD-2-Clause */
/* Host-side FDT producer using the same public helpers as an EFI adapter.
 * The RAM descriptor is a protocol fixture, NOT QEMU's live GPU framebuffer. */
#include <stdio.h>
#include <stdlib.h>
#include <libfdt.h>
#include "pj4edk2_boot_display.h"
#include "pj4edk2_linux_chosen.h"
int main(int argc, char **argv)
{
    unsigned char *fdt = calloc(1, 2*1024*1024), wire[PJ4EDK2_DISPLAY_BYTES];
    unsigned int cells[20] = {1,2,0,0x50000000,128000,128000,320,200,640,1,
        320,200,16,16,288,168,0,0,2,1};
    unsigned long long end;
    FILE *file;
    size_t bytes;
    int chosen;
    if (argc != 3 || !fdt) return 2;
    file=fopen(argv[1],"rb"); if(!file)return 2;
    bytes=fread(fdt,1,2*1024*1024,file);fclose(file);
    if(bytes<40 || fdt_check_header(fdt) || fdt_totalsize(fdt)>bytes ||
       fdt_open_into(fdt,fdt,2*1024*1024))return 3;
    chosen=fdt_path_offset(fdt,"/chosen");
    if(chosen<0)chosen=fdt_add_subnode(fdt,0,"chosen");
    if(chosen<0 || !pj4edk2_display_encode_be(cells,20,wire,sizeof(wire)) ||
       !pj4edk2_linux_initrd_range(0x48000000,4096,&end) || end!=0x48001000 ||
       pj4edk2_linux_initrd_range(~0ULL,2,&end) ||
       fdt_setprop(fdt,chosen,PJ4EDK2_DISPLAY_PROPERTY,wire,sizeof(wire)) ||
       fdt_add_mem_rsv(fdt,0x50000000,128000) || fdt_pack(fdt))return 4;
    file=fopen(argv[2],"wb");if(!file)return 5;
    bytes=fdt_totalsize(fdt);
    if(fwrite(fdt,1,bytes,file)!=bytes || fclose(file))return 5;
    free(fdt);puts("PJ4EDK2_EFI PASS encoded RGB565 test descriptor and initrd bounds");return 0;
}
