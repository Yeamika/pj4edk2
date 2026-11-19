// SPDX-License-Identifier: GPL-2.0-only
/* Generic fbcon adapter for an explicitly reserved, coherent RAM scanout.
 * Platforms requiring cache/MMIO operations use pj4edk2_fbcon.h with their own
 * ownership adapter. This adapter needs no board addresses or DPU knowledge. */
#include <linux/aperture.h>
#include <linux/init.h>
#include <linux/io.h>
#include <linux/libfdt.h>
#include <linux/memblock.h>
#include <linux/of_fdt.h>
#include <linux/platform_device.h>
#include "pj4edk2_fbcon.h"

static unsigned int display_cells[PJ4EDK2_DISPLAY_CELLS];
static struct fb_info *boot_fb;
static void *boot_pixels;

static int pj4edk2_scanout_probe(struct platform_device *pdev)
{
    u64 address = ((u64)display_cells[2] << 32) | display_cells[3];
    int ret;

    boot_pixels = memremap(address, display_cells[4], MEMREMAP_WB);
    if (!boot_pixels)
        return -ENOMEM;
    ret = devm_aperture_acquire_for_platform_device(pdev, address, display_cells[4]);
    if (ret)
        goto unmap;
    boot_fb = pj4edk2_fbcon_register(&pdev->dev, THIS_MODULE, display_cells,
                                    boot_pixels, display_cells[4], NULL, NULL);
    if (IS_ERR(boot_fb)) {
        ret = PTR_ERR(boot_fb);
        boot_fb = NULL;
        goto unmap;
    }
    dev_info(&pdev->dev, "PJ4EDK2_HANDOFF PASS fbcon window %ux%u rotation=%u\n",
             boot_fb->var.xres, boot_fb->var.yres, boot_fb->fbcon_rotate_hint * 90);
    return 0;
unmap:
    memunmap(boot_pixels);
    boot_pixels = NULL;
    return ret;
}

static void pj4edk2_scanout_remove(struct platform_device *pdev)
{
    pj4edk2_fbcon_unregister(boot_fb);
    boot_fb = NULL;
    memunmap(boot_pixels);
    boot_pixels = NULL;
}

static struct platform_driver pj4edk2_scanout_driver = {
    .probe = pj4edk2_scanout_probe,
    .remove = pj4edk2_scanout_remove,
    .driver = {.name = "pj4edk2-fbcon", .suppress_bind_attrs = true},
};

static int __init pj4edk2_console_init(void)
{
    const void *fdt = initial_boot_params, *wire;
    const fdt32_t *phandle;
    const fdt64_t *reg;
    struct memblock_region *r;
    struct platform_device *pdev;
    struct pj4edk2_fbcon_viewport view;
    int chosen, node, parent, bytes, ret;
    u64 address, end;
    bool reserved = false;

    if (!fdt)
        return 0;
    chosen = fdt_path_offset(fdt, "/chosen");
    if (chosen < 0)
        return 0;
    wire = fdt_getprop(fdt, chosen, PJ4EDK2_DISPLAY_PROPERTY, &bytes);
    if (!wire || !pj4edk2_display_decode_be(wire, bytes, display_cells) ||
        !pj4edk2_fbcon_viewport(display_cells, &view))
        return 0;
    phandle = fdt_getprop(fdt, chosen, "pocketjs,boot-display-memory", &bytes);
    if (!phandle || bytes != 4)
        return 0;
    node = fdt_node_offset_by_phandle(fdt, fdt32_to_cpu(*phandle));
    parent = fdt_path_offset(fdt, "/reserved-memory");
    if (node < 0 || parent < 0 || fdt_parent_offset(fdt, node) != parent ||
        fdt_address_cells(fdt, parent) != 2 || fdt_size_cells(fdt, parent) != 2 ||
        !fdt_getprop(fdt, node, "pocketjs,coherent-scanout", NULL) ||
        fdt_getprop(fdt, node, "no-map", NULL) || fdt_getprop(fdt, node, "reusable", NULL))
        return 0;
    reg = fdt_getprop(fdt, node, "reg", &bytes);
    address = ((u64)display_cells[2] << 32) | display_cells[3];
    end = address + display_cells[4];
    if (!reg || bytes != 16 || fdt64_ld(reg) != address ||
        fdt64_ld(reg + 1) < display_cells[4] ||
        !memblock_is_region_memory(address, display_cells[4]) ||
        !memblock_is_map_memory(address))
        return 0;
    for_each_reserved_mem_region(r)
        if (r->base <= address && r->size >= end - r->base)
            reserved = true;
    if (!reserved)
        return 0;
    ret = platform_driver_register(&pj4edk2_scanout_driver);
    if (ret)
        return ret;
    pdev = platform_device_register_simple("pj4edk2-fbcon", -1, NULL, 0);
    if (IS_ERR(pdev)) {
        platform_driver_unregister(&pj4edk2_scanout_driver);
        return PTR_ERR(pdev);
    }
    return 0;
}
/* fbmem is initialized at subsys_initcall; register before logo initdata is
 * released. fbcon replays buffered printk through the normal tty console. */
device_initcall(pj4edk2_console_init);
