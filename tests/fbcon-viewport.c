/* SPDX-License-Identifier: (BSD-2-Clause OR GPL-2.0-only) */
#include <assert.h>
#include <limits.h>
#include <stdio.h>
#include <string.h>
#include "pj4edk2_fbcon_viewport.h"

static void check_rotation(unsigned int rotation, unsigned int format)
{
    unsigned int v[20] = {
        2, 2, 0, 0x80000000, 100000, 100000, 160, 100, 672, 2,
        160, 100, 17, 19, 110, 61, 0, 0, 4, 1,
    };
    struct pj4edk2_fbcon_viewport p;
    unsigned int x, y, logical_w, logical_h, left, top, right, bottom;
    unsigned long long offset;
    unsigned char pixels[100000];

    v[16] = rotation;
    if (format == 1) {
        v[0] = v[9] = 1;
        v[18] = 2;
        v[8] = 352;
    } else {
        v[9] = format;
    }
    assert(pj4edk2_fbcon_viewport(v, &p));
    logical_w = rotation == 90 || rotation == 270 ? v[7] : v[6];
    logical_h = rotation == 90 || rotation == 270 ? v[6] : v[7];
    left = ((unsigned long long)v[12] * logical_w + v[10]-1) / v[10] + 8;
    top = ((unsigned long long)v[13] * logical_h + v[11]-1) / v[11] + 8;
    right = (unsigned long long)(v[12]+v[14]) * logical_w / v[10] - 8;
    bottom = (unsigned long long)(v[13]+v[15]) * logical_h / v[11] - 8;
    assert(p.rotation == rotation / 90);
    memset(pixels, 0xa5, sizeof(pixels));
    for (y = p.y; y < p.y + p.height; ++y)
        memset(pixels + y * p.stride + p.x * v[18], 0x22, p.width * v[18]);
    /* Check every pixel against an independent inverse rotation, including
     * UI padding, outside pixels and stride padding. */
    for (y = 0; y < v[7]; ++y) {
        for (x = 0; x < v[6]; ++x) {
            unsigned int lx = x, ly = y, b;
            int inside;
            switch (rotation) {
            case 90: lx = y; ly = v[6]-1-x; break;
            case 180: lx = v[6]-1-x; ly = v[7]-1-y; break;
            case 270: lx = v[7]-1-y; ly = x; break;
            }
            inside = lx >= left && lx < right && ly >= top && ly < bottom;
            offset = (unsigned long long)y * p.stride + x * v[18];
            for (b = 0; b < v[18]; ++b)
                assert(pixels[offset+b] == (inside ? 0x22 : 0xa5));
        }
        for (x = v[6] * v[18]; x < v[8]; ++x)
            assert(pixels[y * v[8] + x] == 0xa5);
    }
    offset = p.offset + p.size;
    assert(offset == (unsigned long long)(p.y+p.height-1)*p.stride +
                     (p.x+p.width)*v[18]);
    assert(offset <= v[5]);
}

int main(void)
{
    unsigned int rotation, format;
    unsigned int v[20] = {
        2, 2, 0, 0xb0000000, 0x9e4000, 9216000, 1920, 1200, 7680, 2,
        1920, 1200, 120, 84, 1640, 960, 0, 90, 4, 1,
    };
    struct pj4edk2_fbcon_viewport p = {0}, previous;

    for (rotation = 0; rotation < 360; rotation += 90)
        for (format = 1; format <= 3; format++)
            check_rotation(rotation, format);
    assert(pj4edk2_fbcon_viewport(v, &p));
    assert(p.x == 128 && p.y == 92 && p.width == 1624 && p.height == 944);
    previous = p;
    v[12] = UINT_MAX;
    assert(!pj4edk2_fbcon_viewport(v, &p));
    assert(!memcmp(&p, &previous, sizeof(p)));
    v[12] = 120;
    v[14] = 16;
    assert(!pj4edk2_fbcon_viewport(v, &p));
    assert(!pj4edk2_fbcon_viewport(NULL, &p));
    assert(!pj4edk2_fbcon_viewport(v, NULL));
    puts("PASS: fbcon viewport, three formats, four rotations, stride/UI guards and invalid input");
    return 0;
}
