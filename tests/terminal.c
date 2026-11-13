/* SPDX-License-Identifier: GPL-2.0-only */
#include <assert.h>
#include <stdlib.h>
#include <stdio.h>
#include <string.h>
#include "pj4edk2_terminal.h"
/* Guarded pixel tests exercise the production raster, all rotations/formats,
 * scaled viewport, stride padding, scrolling and damage-cache equivalence. */
static void check(unsigned int format,unsigned int rotation,unsigned int cw,unsigned int ch)
{
    unsigned int bpp=format==1?2:4, i,x,y,px,py,off;
    unsigned int w=320,h=240,stride=w*bpp+32,bytes=stride*h;
    unsigned int v[20]={format==1?1:2,2,0,0x40000000,bytes,bytes,w,h,stride,
        format,640,480,40,32,540,400,rotation,0,bpp,1};
    unsigned char font[4096],*ram=malloc(bytes+128),*before=malloc(bytes);
    struct pj4edk_terminal t={0};struct pj4edk_glyphs glyphs;
    struct pj4edk_shadow shadow={0};struct pj4edk_damage damage;
    memset(font,0xff,sizeof(font));memset(ram,0xa5,bytes+128);
    assert(!pj4edk_terminal_init_sized(&t,v,(void *)(ram+64),font,cw,ch));
    pj4edk_prepare_font(&glyphs,font);
    for(i=0;i<2000;i++)pj4edk_character(&t,i%23==0?'\n':i%19==0?'\t':'X');
    pj4edk_draw_damage(&t,&shadow,&damage);assert(damage.cells==t.cols*t.rows);
    for(i=0;i<64;i++) assert(ram[i]==0xa5 && ram[64+bytes+i]==0xa5);
    /* Every byte outside the exact text rectangle and every stride-padding
     * byte must retain its previous value, even on rotated scanout. */
    for(y=0;y<h;y++)for(x=0;x<stride;x++) {
        int touched=0;
        if(x<w*bpp) {
            px=x/bpp;py=y;
            switch(rotation) {
            case 90: off=px;px=py;py=w-1-off;break;
            case 180:px=w-1-px;py=h-1-py;break;
            case 270:off=px;px=h-1-py;py=off;break;
            }
            touched=px>=t.left && px<t.left+t.cols*t.cell_w &&
                py>=t.top && py<t.top+t.rows*t.cell_h;
        }
        assert(touched || ram[64+y*stride+x]==0xa5);
    }
    memcpy(before,ram+64,bytes);pj4edk_draw_damage(&t,&shadow,&damage);
    assert(!damage.cells && !memcmp(before,ram+64,bytes));
    free(before);free(ram);
}
static void optimized_rgb565(void)
{
    unsigned int v[20]={1,2,0,0xb0000000,0x9e4000,4608000,1200,1920,2400,
        1,1920,1200,120,84,1640,960,90,180,2,1};
    unsigned char font[4096],*a=malloc(4608000),*b=malloc(4608000);
    struct pj4edk_terminal t={0},u={0};struct pj4edk_glyphs glyphs;
    struct pj4edk_shadow sa={0},sb={0};struct pj4edk_damage damage;
    unsigned int i;
    assert(a && b);
    memset(a,0xa5,4608000);memcpy(b,a,4608000);
    for(i=0;i<sizeof(font);i++)font[i]=(i*37)^(i>>3);
    assert(!pj4edk_terminal_init(&t,v,(void *)a,font));
    assert(!pj4edk_terminal_init(&u,v,(void *)b,font));
    pj4edk_prepare_font(&glyphs,font);u.glyphs=&glyphs;
    for(i=0;i<10000;i++) {
        unsigned char c=i%47==0?'\n':32+i%95;
        pj4edk_character(&t,c);pj4edk_character(&u,c);
    }
    pj4edk_draw_damage(&t,&sa,&damage);pj4edk_draw_damage(&u,&sb,&damage);
    assert(!memcmp(a,b,4608000));
    free(a);free(b);
}
int main(void)
{
    unsigned int f,r;for(f=1;f<=3;f++)for(r=0;r<360;r+=90){check(f,r,12,24);check(f,r,8,16);}
    optimized_rgb565();
    puts("PASS: terminal formats, rotation, guards, padding, scrolling, damage");
}
