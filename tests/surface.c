/* SPDX-License-Identifier: BSD-2-Clause */
#include <assert.h>
#include <stdio.h>
#include <string.h>
#include "pj4edk2_linux_display.h"
int main(void)
{
    struct pj4edk2_surface s={0x5c000000,1920*1200*4,1920*1200*4,1920,1200,1920*4,2,0,0};
    struct pj4edk2_region r={1920,1200,120,84,1640,960};
    unsigned int cells[20],before[20];unsigned char wire[80];
    assert(pj4edk2_linux_display_cells(&s,&r,cells));
    assert(cells[0]==2 && cells[9]==2 && cells[18]==4);
    assert(pj4edk2_display_encode_be(cells,20,wire,80));
    assert(pj4edk2_display_decode_be(wire,80,before));
    assert(!memcmp(cells,before,sizeof(cells)));
    s.stride=1; assert(!pj4edk2_linux_display_cells(&s,&r,cells));
    assert(!memcmp(cells,before,sizeof(cells)));
    s=(struct pj4edk2_surface){0xb0000000,0x9e4000,4608000,1200,1920,2400,1,90,180};
    assert(pj4edk2_linux_display_cells(&s,&r,cells));
    assert(cells[0]==1 && cells[9]==1 && cells[16]==90 && cells[18]==2);
    r.x=~0u; assert(!pj4edk2_linux_display_cells(&s,&r,cells));
    puts("PASS: shared surface conversion, RGB565 compatibility, 32bpp, failed output unchanged");
}
