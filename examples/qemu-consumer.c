// SPDX-License-Identifier: GPL-2.0-only
/* Protocol-only fixture receiver. Never maps or renders the declared address. */
#include <linux/init.h>
#include <linux/of.h>
#include <linux/printk.h>
#include "pj4edk2_boot_display.h"

static int __init pj4edk2_fixture_check(void)
{
    struct device_node *chosen = of_find_node_by_path("/chosen");
    const void *wire;
    unsigned int cells[PJ4EDK2_DISPLAY_CELLS];
    int bytes;
    if (!chosen)
        return 0;
    wire = of_get_property(chosen, PJ4EDK2_DISPLAY_PROPERTY, &bytes);
    if (!wire)
        pr_info("PJ4EDK2_DISPLAY absent (normal without fixture DTB)\n");
    else if (!pj4edk2_display_decode_be(wire, bytes, cells))
        pr_err("PJ4EDK2_DISPLAY FAIL invalid descriptor\n");
    else
        pr_info("PJ4EDK2_DISPLAY PASS %ux%u rotation=%u rectangle=%u,%u %ux%u; no framebuffer access\n",
            cells[6], cells[7], cells[16], cells[12], cells[13], cells[14], cells[15]);
    of_node_put(chosen);
    return 0;
}
late_initcall(pj4edk2_fixture_check);
