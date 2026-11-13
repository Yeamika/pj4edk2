/* SPDX-License-Identifier: (BSD-2-Clause OR GPL-2.0-only) */
#ifndef PJ4EDK2_LINUX_DISPLAY_H
#define PJ4EDK2_LINUX_DISPLAY_H
#include "pj4edk2_boot_display.h"
/* Explicit platform-validated surface. This helper does not discover GOP,
 * reserve/map memory, authorize MMIO, or assume a particular boot-table ABI. */
struct pj4edk2_surface {
    unsigned long long address;
    unsigned int capacity, bytes, width, height, stride, format, rotation, mount;
};
struct pj4edk2_region { unsigned int width, height, x, y, w, h; };
static inline int pj4edk2_linux_display_cells(const struct pj4edk2_surface *s,
        const struct pj4edk2_region *r, unsigned int *out)
{
    unsigned int v[20], i;
    if (!s || !r || !out) return 0;
    v[0]=s->format==1 ? 1 : 2; v[1]=2;
    v[2]=s->address >> 32; v[3]=s->address;
    v[4]=s->capacity; v[5]=s->bytes; v[6]=s->width; v[7]=s->height;
    v[8]=s->stride; v[9]=s->format; v[10]=r->width; v[11]=r->height;
    v[12]=r->x; v[13]=r->y; v[14]=r->w; v[15]=r->h;
    v[16]=s->rotation; v[17]=s->mount; v[18]=s->format==1 ? 2 : 4; v[19]=1;
    if (!pj4edk2_display_validate_cells(v,20)) return 0;
    for(i=0;i<20;i++) out[i]=v[i];
    return 1;
}
#endif
