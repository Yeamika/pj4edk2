/* SPDX-License-Identifier: GPL-2.0-only */
/* Small allocation-free terminal. Coordinates are native landscape pixels;
 * The adapter owns mapping, cache maintenance and scanout submission.
 * Kept independent of printk/MMIO so the exact raster code can be exercised
 * against guarded RAM before a kernel is booted on the board.
 */
#ifndef PJ4EDK_BOOT_DISPLAY_CORE_H
#define PJ4EDK_BOOT_DISPLAY_CORE_H

#include "pj4edk2_boot_display.h"

#ifndef PJ4EDK_COLS
#define PJ4EDK_COLS 160
#endif
#ifndef PJ4EDK_ROWS
#define PJ4EDK_ROWS 50
#endif
#if PJ4EDK_ROWS > 64
#error "terminal dirty mask supports at most 64 rows"
#endif
#define PJ4EDK_CELL_W 12
#define PJ4EDK_CELL_H 24

/* Pre-rotated RGB565 glyphs: 55 KiB built once, then contiguous 48-byte
 * copies per glyph column instead of per-pixel scale/bit/color calculation.
 */
struct pj4edk_glyphs {
	unsigned short pixels[96][PJ4EDK_CELL_W][PJ4EDK_CELL_H];
};

static inline void pj4edk_prepare_font(struct pj4edk_glyphs *glyphs, const unsigned char *font)
{
	unsigned int ch, x, y;

	for (ch = 32; ch < 128; ch++)
		for (x = 0; x < PJ4EDK_CELL_W; x++)
			for (y = 0; y < PJ4EDK_CELL_H; y++) {
				unsigned int bits = font[ch * 16 + y * 16 / PJ4EDK_CELL_H];

				glyphs->pixels[ch - 32][x][PJ4EDK_CELL_H - 1 - y] =
					bits & (0x80 >> (x * 8 / PJ4EDK_CELL_W)) ? 0xdedb : 0;
			}
}

struct pj4edk_terminal {
	unsigned short *pixels;
	const unsigned char *font;
	const struct pj4edk_glyphs *glyphs;
	unsigned int left, top, cols, rows, col, row;
	unsigned int physical_w, physical_h, stride, format, rotation, cell_w, cell_h;
	unsigned long long dirty;
	union {
        unsigned char text[PJ4EDK_ROWS][PJ4EDK_COLS];
        unsigned char cells[PJ4EDK_ROWS][PJ4EDK_COLS];
    };
};

static inline int pj4edk_terminal_layout_sized(struct pj4edk_terminal *t, const unsigned int *v,
                             unsigned short *pixels, const unsigned char *font,
                             unsigned int cell_w, unsigned int cell_h)
{
    unsigned int width, height, right, bottom, i, j;
    unsigned long long left, top;
    if (!cell_w || !cell_h || cell_w > 32 || cell_h > 64 ||
        !pj4edk2_display_validate_cells(v, PJ4EDK2_DISPLAY_CELLS))
        return -1;
    width = (v[16] == 90 || v[16] == 270) ? v[7] : v[6];
    height = (v[16] == 90 || v[16] == 270) ? v[6] : v[7];
    left = ((unsigned long long)v[12] * width + v[10] - 1) / v[10] + 8;
    top = ((unsigned long long)v[13] * height + v[11] - 1) / v[11] + 8;
    right = (unsigned long long)(v[12] + v[14]) * width / v[10];
    bottom = (unsigned long long)(v[13] + v[15]) * height / v[11];
    if (right < 8 || bottom < 8 || right - 8 <= left || bottom - 8 <= top)
        return -1;
    t->left = left; t->top = top;
    t->cols = (right - 8 - t->left) / cell_w;
    t->rows = (bottom - 8 - t->top) / cell_h;
    if (!t->cols || !t->rows) return -1;
    /* Bounded storage: a large viewport leaves unused pixels untouched. */
    if (t->cols > PJ4EDK_COLS) t->cols=PJ4EDK_COLS;
    if (t->rows > PJ4EDK_ROWS) t->rows=PJ4EDK_ROWS;
    t->physical_w=v[6]; t->physical_h=v[7]; t->stride=v[8];
    t->format=v[9]; t->rotation=v[16]; t->cell_w=cell_w; t->cell_h=cell_h;
    t->pixels=pixels; t->font=font; t->glyphs=0;
    t->col=t->row=0; t->dirty=(t->rows == 64 ? ~0ULL : (1ULL << t->rows)-1);
    for (i=0;i<t->rows;i++) for(j=0;j<t->cols;j++) t->text[i][j]=' ';
    return 0;
}

static inline int pj4edk_terminal_layout(struct pj4edk_terminal *t, const unsigned int *v,
                                        unsigned short *pixels,const unsigned char *font)
{
    return pj4edk_terminal_layout_sized(t,v,pixels,font,PJ4EDK_CELL_W,PJ4EDK_CELL_H);
}
static inline int pj4edk_terminal_init_sized(struct pj4edk_terminal *t,const unsigned int *v,
        unsigned short *pixels,const unsigned char *font,unsigned int w,unsigned int h)
{
    if(!pixels || !font) return -1;
    return pj4edk_terminal_layout_sized(t,v,pixels,font,w,h);
}
static inline int pj4edk_terminal_init(struct pj4edk_terminal *t, const unsigned int *v,
                               unsigned short *pixels, const unsigned char *font)
{
    if (!pixels || !font) return -1;
    return pj4edk_terminal_layout(t,v,pixels,font);
}

static inline void pj4edk_newline(struct pj4edk_terminal *t)
{
	unsigned int i, j;

	t->col = 0;
	if (++t->row < t->rows)
		return;
	t->row = t->rows - 1;
	for (i = 1; i < t->rows; i++)
		for (j = 0; j < t->cols; j++)
			t->text[i - 1][j] = t->text[i][j];
	for (j = 0; j < t->cols; j++)
		t->text[t->row][j] = ' ';
	t->dirty = (t->rows == 64 ? ~0ULL : (1ULL << t->rows) - 1);
}

static inline void pj4edk_character(struct pj4edk_terminal *t, unsigned char ch)
{
	if (ch == '\r') {
		t->col = 0;
		return;
	}
	if (ch == '\n') {
		pj4edk_newline(t);
		return;
	}
	if (ch == '\t') {
		unsigned int n = 4 - (t->col % 4);

		while (n--)
			pj4edk_character(t, ' ');
		return;
	}
	if (ch < 32)
		return;
	if (t->col == t->cols)
		pj4edk_newline(t);
	t->text[t->row][t->col++] = ch < 127 ? ch : '?';
	t->dirty |= 1ULL << t->row;
}

/* Each glyph column is contiguous in the inherited rotated scanout. */
static inline void pj4edk_draw_cell(struct pj4edk_terminal *t, unsigned int row, unsigned int col)
{
	unsigned int x, y, ch = t->text[row][col];

	for (x = 0; x < t->cell_w; x++) {
		if (t->glyphs && t->format == 1 && t->rotation == 90 &&
            t->cell_w == PJ4EDK_CELL_W && t->cell_h == PJ4EDK_CELL_H &&
            t->physical_w == 1200 && t->stride == 2400) {
			unsigned int nx = t->left + col * PJ4EDK_CELL_W + x;
			unsigned int py = 1200 - t->top - (row + 1) * PJ4EDK_CELL_H;

			__builtin_memcpy(t->pixels + nx * 1200 + py,
					t->glyphs->pixels[ch - 32][x],
					PJ4EDK_CELL_H * sizeof(unsigned short));
			continue;
		}
		for (y = 0; y < t->cell_h; y++) {
			unsigned int bits = t->font[ch * 16 + y * 16 / t->cell_h];
			unsigned short color = bits & (0x80 >> (x * 8 / t->cell_w)) ? 0xdedb : 0;
			unsigned int nx = t->left + col * t->cell_w + x;
			unsigned int ny = t->top + row * t->cell_h + y;

			unsigned int px=nx, py=ny;
            unsigned char *target;
            switch (t->rotation) {
            case 90: px=t->physical_w-1-ny; py=nx; break;
            case 180: px=t->physical_w-1-nx; py=t->physical_h-1-ny; break;
            case 270: px=ny; py=t->physical_h-1-nx; break;
            }
            target=(unsigned char *)t->pixels + (unsigned long long)py*t->stride;
            if (t->format == 1) {
                target+=px*2; target[0]=color; target[1]=color >> 8;
            } else {
                /* Neutral foreground is identical in RGBX and BGRX. */
                target+=px*4;
                target[0]=target[1]=target[2]=color ? 0xde : 0; target[3]=0;
            }
		}
	}
}

struct pj4edk_damage {
	/* Half-open changed row interval for each glyph column. */
	unsigned char first[PJ4EDK_COLS], end[PJ4EDK_COLS];
	unsigned int cells;
};

struct pj4edk_shadow {
	unsigned long long valid;
	unsigned char text[PJ4EDK_ROWS][PJ4EDK_COLS];
};

/* Compare the final batched text with the last drawn text. Scrolling need not
 * redraw identical spaces/glyphs. Cache cleaning merges adjacent dirty rows
 * into one interval per physical scanline (at most cols * 12 calls/frame).
 */
static inline void pj4edk_draw_damage(struct pj4edk_terminal *t, struct pj4edk_shadow *shadow,
			    struct pj4edk_damage *damage)
{
	unsigned int row, col;

	damage->cells = 0;
	for (col = 0; col < t->cols; col++) {
		damage->first[col] = t->rows;
		damage->end[col] = 0;
	}
	for (row = 0; row < t->rows; row++) {
		if (!(t->dirty & (1ULL << row)))
			continue;
		for (col = 0; col < t->cols; col++) {
			if ((shadow->valid & (1ULL << row)) &&
			    shadow->text[row][col] == t->text[row][col])
				continue;
			pj4edk_draw_cell(t, row, col);
			shadow->text[row][col] = t->text[row][col];
			if (damage->first[col] > row)
				damage->first[col] = row;
			damage->end[col] = row + 1;
			damage->cells++;
		}
		shadow->valid |= 1ULL << row;
	}
}
#endif
