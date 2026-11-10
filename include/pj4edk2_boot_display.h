/* SPDX-License-Identifier: (BSD-2-Clause OR GPL-2.0-only) */
/* Version-1 PocketJS DT boot-display wire contract. No platform addresses,
 * kernel/UEFI APIs, allocation, MMIO or rendering policy live here. */
#ifndef PJ4EDK2_BOOT_DISPLAY_PROTOCOL_H
#define PJ4EDK2_BOOT_DISPLAY_PROTOCOL_H

#define PJ4EDK2_DISPLAY_PROPERTY "pocketjs,boot-display"
#define PJ4EDK2_DISPLAY_CELLS 20u
#define PJ4EDK2_DISPLAY_BYTES (PJ4EDK2_DISPLAY_CELLS * 4u)
typedef char pj4edk2_display_u32_width[(sizeof(unsigned int) == 4) ? 1 : -1];
typedef char pj4edk2_display_u64_width[(sizeof(unsigned long long) == 8) ? 1 : -1];

static inline int pj4edk2_display_rectangle_valid(unsigned int canvas_w,
        unsigned int canvas_h, unsigned int x, unsigned int y,
        unsigned int width, unsigned int height)
{
    return canvas_w && canvas_h && width && height && x <= canvas_w &&
        y <= canvas_h && width <= canvas_w - x && height <= canvas_h - y;
}

static inline int pj4edk2_display_rotation_valid(unsigned int rotation)
{
    return rotation == 0 || rotation == 90 || rotation == 180 || rotation == 270;
}

/* Validate decoded host-endian cells, independently of a board's accepted
 * framebuffer address, supported resolution and memory reservation policy. */
static inline int pj4edk2_display_validate_cells(const unsigned int *v,
                                             unsigned int count)
{
    unsigned long long address, scan_bytes, row_bytes;

    if (!v || count != PJ4EDK2_DISPLAY_CELLS || v[0] != 1 || v[1] != 2 ||
        v[19] != 1 || v[9] != 1 || v[18] != 2 || !v[4] || !v[5] ||
        !v[6] || !v[7] || !v[8] || v[5] > v[4] ||
        !pj4edk2_display_rotation_valid(v[16]) || !pj4edk2_display_rotation_valid(v[17]) ||
        !pj4edk2_display_rectangle_valid(v[10], v[11], v[12], v[13], v[14], v[15]))
        return 0;
    address = ((unsigned long long)v[2] << 32) | v[3];
    row_bytes = (unsigned long long)v[6] * v[18];
    scan_bytes = (unsigned long long)v[8] * v[7];
    return row_bytes <= v[8] && scan_bytes <= v[5] &&
        address <= ~0ULL - v[4];
}

/* Byte access allows an unaligned DT property. Invalid input leaves out
 * untouched; no host endianness or FDT structure layout is assumed. */
static inline int pj4edk2_display_decode_be(const void *wire, unsigned int bytes,
                                        unsigned int *out)
{
    const unsigned char *p = (const unsigned char *)wire;
    unsigned int v[PJ4EDK2_DISPLAY_CELLS], i;

    if (!p || !out || bytes != PJ4EDK2_DISPLAY_BYTES)
        return 0;
    for (i = 0; i < PJ4EDK2_DISPLAY_CELLS; i++)
        v[i] = ((unsigned int)p[i * 4] << 24) |
               ((unsigned int)p[i * 4 + 1] << 16) |
               ((unsigned int)p[i * 4 + 2] << 8) | p[i * 4 + 3];
    if (!pj4edk2_display_validate_cells(v, PJ4EDK2_DISPLAY_CELLS))
        return 0;
    for (i = 0; i < PJ4EDK2_DISPLAY_CELLS; i++)
        out[i] = v[i];
    return 1;
}

static inline int pj4edk2_display_encode_be(const unsigned int *v,
                                        unsigned int count, void *wire,
                                        unsigned int bytes)
{
    unsigned char *p = (unsigned char *)wire;
    unsigned int i;

    if (!p || bytes != PJ4EDK2_DISPLAY_BYTES || !pj4edk2_display_validate_cells(v, count))
        return 0;
    for (i = 0; i < PJ4EDK2_DISPLAY_CELLS; i++) {
        p[i * 4] = (unsigned char)(v[i] >> 24);
        p[i * 4 + 1] = (unsigned char)(v[i] >> 16);
        p[i * 4 + 2] = (unsigned char)(v[i] >> 8);
        p[i * 4 + 3] = (unsigned char)v[i];
    }
    return 1;
}
#endif
