/* SPDX-License-Identifier: BSD-2-Clause */
#ifndef PJ4EDK2_LINUX_FDT_H
#define PJ4EDK2_LINUX_FDT_H
/* Include libfdt.h before this header, using the caller's firmware/host port. */
#include "pj4edk2_boot_display.h"
#include "pj4edk2_linux_chosen.h"
static inline int pj4edk2_linux_fdt_display(void *fdt, const unsigned int *cells)
{
    unsigned char wire[PJ4EDK2_DISPLAY_BYTES];
    int chosen=fdt_path_offset(fdt,"/chosen");
    if (!pj4edk2_display_encode_be(cells,20,wire,sizeof(wire))) return -FDT_ERR_BADVALUE;
    if (chosen == -FDT_ERR_NOTFOUND) chosen=fdt_add_subnode(fdt,0,"chosen");
    if (chosen<0) return chosen;
    return fdt_setprop(fdt,chosen,PJ4EDK2_DISPLAY_PROPERTY,wire,sizeof(wire));
}
static inline int pj4edk2_linux_fdt_initrd(void *fdt, unsigned long long start,
                                        unsigned long long bytes)
{
    unsigned long long end; int chosen,ret;
    if (!pj4edk2_linux_initrd_range(start,bytes,&end)) return -FDT_ERR_BADVALUE;
    chosen=fdt_path_offset(fdt,"/chosen");
    if (chosen<0) return chosen;
    ret=fdt_setprop_u64(fdt,chosen,"linux,initrd-start",start);
    if (ret) return ret;
    return fdt_setprop_u64(fdt,chosen,"linux,initrd-end",end);
}
#endif
