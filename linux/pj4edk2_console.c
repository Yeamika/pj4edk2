// SPDX-License-Identifier: GPL-2.0-only
/* Built-in early printk receiver for coherent, reserved RAM scanout.
 * Register during console_init, collect text immediately, and render from the
 * core-init worker onward. Not a late initramfs module and not a tty/fbdev.
 * Platforms needing cache operations or MMIO use the terminal with their own
 * mapping/present/ownership adapter (e.g. the W202DS inherited DPU path). */
#include <linux/console.h>
#include <linux/font.h>
#include <linux/init.h>
#include <linux/io.h>
#include <linux/libfdt.h>
#include <linux/memblock.h>
#include <linux/of_fdt.h>
#include <linux/spinlock.h>
#include <linux/workqueue.h>
#define PJ4EDK_COLS 256
#define PJ4EDK_ROWS 64
#include "pj4edk2_terminal.h"
static struct pj4edk_terminal text, frame;
static struct pj4edk_shadow shadow;
static struct pj4edk_damage damage;
static DEFINE_RAW_SPINLOCK(text_lock);
static bool owned, ready;
static void render(struct work_struct *work);
static DECLARE_WORK(render_work,render);
static void render(struct work_struct *work)
{
    unsigned long flags;
    (void)work;
    raw_spin_lock_irqsave(&text_lock,flags);
    if(!owned) {raw_spin_unlock_irqrestore(&text_lock,flags);return;}
    frame=text; text.dirty=0;
    raw_spin_unlock_irqrestore(&text_lock,flags);
    pj4edk_draw_damage(&frame,&shadow,&damage);
    wmb(); /* Explicitly coherent RAM; no device-register writes here. */
    raw_spin_lock_irqsave(&text_lock,flags);
    if(owned && text.dirty) schedule_work(&render_work);
    raw_spin_unlock_irqrestore(&text_lock,flags);
}
static void write_log(struct console *con,const char *s,unsigned int n)
{
    unsigned long flags;
    (void)con;
    raw_spin_lock_irqsave(&text_lock,flags);
    if(owned) {
        while(n--) pj4edk_character(&text,*s++);
        if(ready) schedule_work(&render_work);
    }
    raw_spin_unlock_irqrestore(&text_lock,flags);
}
static struct console log_console={.name="pj4edklog",.write=write_log,
    .flags=CON_ENABLED|CON_PRINTBUFFER,.index=0};
static int __init init_console(void)
{
    const void *fdt=initial_boot_params,*wire;
    const fdt32_t *phandle;
    const fdt64_t *reg;
    struct memblock_region *r;
    unsigned int cells[20];int chosen,node,parent,bytes;
    unsigned long long address,end;
    bool reserved=false;
    void *pixels;
    if(!fdt) return 0;
    chosen=fdt_path_offset(fdt,"/chosen"); if(chosen<0) return 0;
    wire=fdt_getprop(fdt,chosen,PJ4EDK2_DISPLAY_PROPERTY,&bytes);
    if(!wire || !pj4edk2_display_decode_be(wire,bytes,cells)) return 0;
    phandle=fdt_getprop(fdt,chosen,"pocketjs,boot-display-memory",&bytes);
    if(!phandle || bytes!=4) return 0;
    node=fdt_node_offset_by_phandle(fdt,fdt32_to_cpu(*phandle));
    parent=fdt_path_offset(fdt,"/reserved-memory");
    if(node<0 || parent<0 || fdt_parent_offset(fdt,node)!=parent ||
       fdt_address_cells(fdt,parent)!=2 || fdt_size_cells(fdt,parent)!=2 ||
       !fdt_getprop(fdt,node,"pocketjs,coherent-scanout",NULL) ||
       fdt_getprop(fdt,node,"no-map",NULL) || fdt_getprop(fdt,node,"reusable",NULL)) return 0;
    reg=fdt_getprop(fdt,node,"reg",&bytes);
    address=((unsigned long long)cells[2]<<32)|cells[3];end=address+cells[4];
    if(!reg || bytes!=16 || fdt64_ld(reg)!=address || fdt64_ld(reg+1)<cells[4]) return 0;
    /* Full coverage, not memblock_is_region_reserved's intersection test. */
    for_each_reserved_mem_region(r)
        if(r->base<=address && r->size>=end-r->base) reserved=true;
    if(!reserved) return 0;
    pixels=memremap(address,cells[4],MEMREMAP_WB);if(!pixels) return 0;
    if(pj4edk_terminal_init_sized(&text,cells,pixels,font_vga_8x16.data,8,16)) {
        memunmap(pixels);return 0;
    }
    owned=true;register_console(&log_console);
    pr_info("PJ4EDK2_HANDOFF PASS early retained framebuffer console %ux%u font=8x16\n",text.cols,text.rows);
    return 0;
}
console_initcall(init_console);
static int __init start_renderer(void)
{
    unsigned long flags;
    raw_spin_lock_irqsave(&text_lock,flags);ready=true;
    if(owned) schedule_work(&render_work);
    raw_spin_unlock_irqrestore(&text_lock,flags);
    return 0;
}
core_initcall(start_renderer);
