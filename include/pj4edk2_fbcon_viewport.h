/* SPDX-License-Identifier: (BSD-2-Clause OR GPL-2.0-only) */
#ifndef PJ4EDK2_FBCON_VIEWPORT_H
#define PJ4EDK2_FBCON_VIEWPORT_H

#include "pj4edk2_boot_display.h"

/* Physical subrectangle in the inherited buffer. fbcon rotates its content;
 * the platform retains responsibility for any scanout/DPU rotation. */
struct pj4edk2_fbcon_viewport {
    unsigned int x, y, width, height, stride, bytes_per_pixel, rotation;
    unsigned long long offset, size;
};

static inline int pj4edk2_fbcon_viewport(const unsigned int *v,
                                       struct pj4edk2_fbcon_viewport *out)
{
    struct pj4edk2_fbcon_viewport p;
    unsigned int w, h, right, bottom, left, top;
    unsigned long long x, y;

    if (!out || !pj4edk2_display_validate_cells(v, PJ4EDK2_DISPLAY_CELLS))
        return 0;
    w = (v[16] == 90 || v[16] == 270) ? v[7] : v[6];
    h = (v[16] == 90 || v[16] == 270) ? v[6] : v[7];
    /* Round inward, then keep the existing eight-pixel UI padding. */
    x = ((unsigned long long)v[12] * w + v[10] - 1) / v[10];
    y = ((unsigned long long)v[13] * h + v[11] - 1) / v[11];
    right = (unsigned long long)(v[12] + v[14]) * w / v[10];
    bottom = (unsigned long long)(v[13] + v[15]) * h / v[11];
    if (right <= x || bottom <= y || right - x <= 16 || bottom - y <= 16)
        return 0;
    left = x + 8;
    top = y + 8;
    right -= 8;
    bottom -= 8;
    p.width = right - left;
    p.height = bottom - top;
    p.x = left;
    p.y = top;
    switch (v[16]) {
    case 90:
        p.x = v[6] - bottom;
        p.y = left;
        p.width = bottom - top;
        p.height = right - left;
        break;
    case 180:
        p.x = v[6] - right;
        p.y = v[7] - bottom;
        break;
    case 270:
        p.x = top;
        p.y = v[7] - right;
        p.width = bottom - top;
        p.height = right - left;
        break;
    }
    p.stride = v[8];
    p.bytes_per_pixel = v[18];
    p.rotation = v[16] / 90;
    p.offset = (unsigned long long)p.y * p.stride +
               (unsigned long long)p.x * p.bytes_per_pixel;
    p.size = (unsigned long long)(p.height - 1) * p.stride +
             (unsigned long long)p.width * p.bytes_per_pixel;
    if (p.offset > v[5] || p.size > v[5] - p.offset)
        return 0;
    *out = p;
    return 1;
}

#endif
