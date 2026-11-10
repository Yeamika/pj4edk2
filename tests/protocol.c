#include <assert.h>
#include <limits.h>
#include <stdio.h>
#include <string.h>
#include "pjb_boot_display.h"
int main(void)
{
    unsigned int good[20]={1,2,0,0xb0000000,0x9e4000,4608000,1200,1920,2400,1,
        1920,1200,120,84,1640,960,90,180,2,1};
    unsigned int v[20],out[20],before[20];
    unsigned char wire[82];
    assert(pjb_display_validate_cells(good,20));
    assert(!pjb_display_validate_cells(0,20));
    assert(!pjb_display_validate_cells(good,19));
    assert(!pjb_display_validate_cells(good,21));
    memset(wire,0xa5,sizeof(wire));
    assert(pjb_display_encode_be(good,20,wire+1,80));
    assert(wire[0]==0xa5 && wire[81]==0xa5);
    assert(wire[1]==0 && wire[4]==1 && wire[5]==0 && wire[8]==2);
    assert(pjb_display_decode_be(wire+1,80,out));
    assert(!memcmp(out,good,sizeof(out)));
    memcpy(before,out,sizeof(out));
    assert(!pjb_display_decode_be(wire+1,79,out));
    assert(!pjb_display_decode_be(wire+1,81,out));
    assert(!memcmp(before,out,sizeof(out)));
    wire[4]=9;
    assert(!pjb_display_decode_be(wire+1,80,out));
    assert(!memcmp(before,out,sizeof(out)));
    memcpy(v,good,sizeof(v));v[2]=UINT_MAX;v[3]=UINT_MAX;
    assert(!pjb_display_validate_cells(v,20));
    memcpy(v,good,sizeof(v));v[8]=1;
    assert(!pjb_display_validate_cells(v,20));
    memcpy(v,good,sizeof(v));v[7]=UINT_MAX;v[8]=UINT_MAX;
    assert(!pjb_display_validate_cells(v,20));
    memcpy(v,good,sizeof(v));v[12]=UINT_MAX;v[14]=UINT_MAX;
    assert(!pjb_display_validate_cells(v,20));
    memcpy(v,good,sizeof(v));v[14]=0;
    assert(!pjb_display_validate_cells(v,20));
    /* A public contract accepts valid foreign layouts; board policy is separate. */
    memcpy(v,good,sizeof(v));v[3]=0x10000000;v[4]=v[5]=128000;
    v[6]=320;v[7]=200;v[8]=640;v[10]=320;v[11]=200;
    v[12]=v[13]=0;v[14]=320;v[15]=200;v[16]=v[17]=0;
    assert(pjb_display_validate_cells(v,20));
    puts("PASS: protocol, unaligned big-endian roundtrip, bounds, overflow, foreign layout");
    return 0;
}
