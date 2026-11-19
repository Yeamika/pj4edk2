/* SPDX-License-Identifier: GPL-2.0-only */
#ifndef PJ4EDK2_LINUX_FBCON_H
#define PJ4EDK2_LINUX_FBCON_H

#include <linux/err.h>
#include <linux/fb.h>
#include <linux/module.h>
#include "pj4edk2_fbcon_viewport.h"

/* Header implementation lets built-in firmware and native DRM adapters use
 * the same fbcon path without an exported dependency between their modules.
 * damage() must be atomic-safe: queue platform cache/present work only. */
struct pj4edk2_fbcon {
    struct pj4edk2_fbcon_viewport view;
    void (*damage)(void *context, unsigned int y, unsigned int height);
    void *context;
    u32 palette[16];
};

static inline bool pj4edk2_fbcon_rect(struct fb_info *info, u32 x, u32 y,
                                    u32 width, u32 height)
{
    return pj4edk2_display_rectangle_valid(info->var.xres, info->var.yres,
                                           x, y, width, height);
}

static void pj4edk2_fbcon_damage(struct fb_info *info, u32 y, u32 height)
{
    struct pj4edk2_fbcon *fb = info->par;

    if (fb->damage)
        fb->damage(fb->context, fb->view.y + y, height);
}

static void pj4edk2_fbcon_fillrect(struct fb_info *info,
                                 const struct fb_fillrect *rect)
{
    if (!pj4edk2_fbcon_rect(info, rect->dx, rect->dy, rect->width, rect->height))
        return;
    sys_fillrect(info, rect);
    pj4edk2_fbcon_damage(info, rect->dy, rect->height);
}

static void pj4edk2_fbcon_copyarea(struct fb_info *info,
                                 const struct fb_copyarea *area)
{
    if (!pj4edk2_fbcon_rect(info, area->dx, area->dy, area->width, area->height) ||
        !pj4edk2_fbcon_rect(info, area->sx, area->sy, area->width, area->height))
        return;
    sys_copyarea(info, area);
    pj4edk2_fbcon_damage(info, area->dy, area->height);
}

static void pj4edk2_fbcon_imageblit(struct fb_info *info,
                                  const struct fb_image *image)
{
    if (!pj4edk2_fbcon_rect(info, image->dx, image->dy, image->width, image->height))
        return;
    sys_imageblit(info, image);
    pj4edk2_fbcon_damage(info, image->dy, image->height);
}

static int pj4edk2_fbcon_setcolreg(u32 regno, u32 red, u32 green, u32 blue,
                                 u32 transp, struct fb_info *info)
{
    struct pj4edk2_fbcon *fb = info->par;

    if (regno >= ARRAY_SIZE(fb->palette))
        return -EINVAL;
    fb->palette[regno] =
        ((red >> (16 - info->var.red.length)) << info->var.red.offset) |
        ((green >> (16 - info->var.green.length)) << info->var.green.offset) |
        ((blue >> (16 - info->var.blue.length)) << info->var.blue.offset);
    return 0;
}

/* A window with a full-frame stride must not expose padding/the surrounding
 * UI through /dev/fbN. fbcon opens with user=0. No userspace mmap/read/write. */
static int pj4edk2_fbcon_open(struct fb_info *info, int user)
{
    return user ? -ENODEV : 0;
}

static void pj4edk2_fbcon_destroy(struct fb_info *info)
{
    fb_dealloc_cmap(&info->cmap);
    framebuffer_release(info);
}

static struct fb_info *pj4edk2_fbcon_register(struct device *dev,
        struct module *owner, const unsigned int *cells, void *pixels,
        size_t bytes, void (*damage)(void *, unsigned int, unsigned int),
        void *context)
{
    struct pj4edk2_fbcon_viewport view;
    struct fb_info *info;
    struct pj4edk2_fbcon *fb;
    struct fb_ops *ops;
    int ret;

    if (!pixels || !pj4edk2_fbcon_viewport(cells, &view) ||
        view.offset > bytes || view.size > bytes - view.offset)
        return ERR_PTR(-EINVAL);
    if (view.rotation && !IS_ENABLED(CONFIG_FRAMEBUFFER_CONSOLE_ROTATION))
        return ERR_PTR(-EOPNOTSUPP);
    info = framebuffer_alloc(sizeof(*fb) + sizeof(*ops), dev);
    if (!info)
        return ERR_PTR(-ENOMEM);
    fb = info->par;
    ops = (void *)(fb + 1);
    fb->view = view;
    fb->damage = damage;
    fb->context = context;
    ops->owner = owner;
    ops->fb_open = pj4edk2_fbcon_open;
    ops->fb_fillrect = pj4edk2_fbcon_fillrect;
    ops->fb_copyarea = pj4edk2_fbcon_copyarea;
    ops->fb_imageblit = pj4edk2_fbcon_imageblit;
    ops->fb_setcolreg = pj4edk2_fbcon_setcolreg;
    ops->fb_destroy = pj4edk2_fbcon_destroy;
    info->fbops = ops;
    info->flags = FBINFO_VIRTFB | FBINFO_HIDE_SMEM_START;
    info->screen_buffer = (char *)pixels + view.offset;
    info->screen_size = view.size;
    info->pseudo_palette = fb->palette;
    strscpy(info->fix.id, "PJ4EDK2", sizeof(info->fix.id));
    info->fix.type = FB_TYPE_PACKED_PIXELS;
    info->fix.visual = FB_VISUAL_TRUECOLOR;
    info->fix.accel = FB_ACCEL_NONE;
    info->fix.line_length = view.stride;
    info->fix.smem_len = view.size;
    info->var.xres = info->var.xres_virtual = view.width;
    info->var.yres = info->var.yres_virtual = view.height;
    info->var.width = info->var.height = -1;
    info->var.bits_per_pixel = view.bytes_per_pixel * 8;
    info->var.activate = FB_ACTIVATE_NOW;
    info->var.vmode = FB_VMODE_NONINTERLACED;
    info->var.rotate = view.rotation;
    info->fbcon_rotate_hint = view.rotation;
    info->var.red.length = info->var.blue.length = view.bytes_per_pixel == 2 ? 5 : 8;
    info->var.green.length = view.bytes_per_pixel == 2 ? 6 : 8;
    info->var.green.offset = view.bytes_per_pixel == 2 ? 5 : 8;
    info->var.red.offset = cells[9] == 1 ? 11 : cells[9] == 2 ? 16 : 0;
    info->var.blue.offset = cells[9] == 3 ? 16 : 0;
    ret = fb_alloc_cmap(&info->cmap, 256, 0);
    if (ret) {
        framebuffer_release(info);
        return ERR_PTR(ret);
    }
    ret = register_framebuffer(info);
    if (ret) {
        pj4edk2_fbcon_destroy(info);
        return ERR_PTR(ret);
    }
    return info;
}

/* unregister_framebuffer detaches fbcon under the console lock. fb_destroy
 * frees the allocation at the final fb_info reference; do not free it twice.
 * The caller drains its present worker before releasing mappings/clocks. */
static inline void pj4edk2_fbcon_unregister(struct fb_info *info)
{
    if (info)
        unregister_framebuffer(info);
}

#endif
