// SPDX-License-Identifier: GPL-2.0-only
/* A virtual DRM/KMS endpoint for protocol integration testing, not a board driver.
 * Uses GEM shadow buffers and NEVER maps the address advertised in the fixture. */
#include <linux/module.h>
#include <linux/platform_device.h>
#include <linux/of.h>
#include <drm/drm_atomic_helper.h>
#include <drm/drm_drv.h>
#include <drm/drm_fourcc.h>
#include <drm/drm_gem_framebuffer_helper.h>
#include <drm/drm_gem_shmem_helper.h>
#include <drm/drm_modes.h>
#include <drm/drm_probe_helper.h>
#include <drm/drm_simple_kms_helper.h>
#include "pj4edk2_boot_display.h"

struct pj4edk2_device { struct drm_device drm; struct drm_simple_display_pipe pipe; struct drm_connector connector; };
static struct platform_device *pdev;
static int modes(struct drm_connector *connector)
{
    struct drm_display_mode *mode=drm_cvt_mode(connector->dev,320,200,60,false,false,false);
    if(!mode)return 0;
    mode->type|=DRM_MODE_TYPE_PREFERRED;drm_mode_probed_add(connector,mode);return 1;
}
static const struct drm_connector_helper_funcs connector_helper={.get_modes=modes};
static const struct drm_connector_funcs connector_funcs={
    .fill_modes=drm_helper_probe_single_connector_modes,.destroy=drm_connector_cleanup,
    .reset=drm_atomic_helper_connector_reset,.atomic_duplicate_state=drm_atomic_helper_connector_duplicate_state,
    .atomic_destroy_state=drm_atomic_helper_connector_destroy_state,
};
static const struct drm_simple_display_pipe_funcs pipe_funcs={};
static const struct drm_mode_config_funcs config_funcs={
    .fb_create=drm_gem_fb_create,.atomic_check=drm_atomic_helper_check,.atomic_commit=drm_atomic_helper_commit,
};
DEFINE_DRM_GEM_FOPS(pj4edk2_fops);
static const struct drm_driver driver={
    .driver_features=DRIVER_GEM|DRIVER_MODESET|DRIVER_ATOMIC,
    .fops=&pj4edk2_fops,.name="pj4edk2",.desc="PocketJS protocol test DRM",.date="20261110",
    .major=1,.minor=0,DRM_GEM_SHMEM_DRIVER_OPS,
};
static int probe(struct platform_device *platform)
{
    static const u32 formats[]={DRM_FORMAT_RGB565,DRM_FORMAT_XRGB8888};
    struct pj4edk2_device *dev;
    struct device_node *chosen;
    const void *wire;
    unsigned int cells[20];int bytes,ret;
    chosen=of_find_node_by_path("/chosen");
    wire=chosen ? of_get_property(chosen,PJ4EDK2_DISPLAY_PROPERTY,&bytes):NULL;
    if(wire && !pj4edk2_display_decode_be(wire,bytes,cells)){of_node_put(chosen);return -EINVAL;}
    pr_info("PJ4EDK2_DRM protocol=%s; virtual DRM/KMS validation only\n",wire?"PASS":"absent");
    of_node_put(chosen);
    dev=devm_drm_dev_alloc(&platform->dev,&driver,struct pj4edk2_device,drm);
    if(IS_ERR(dev))return PTR_ERR(dev);
    platform_set_drvdata(platform,dev);
    ret=drmm_mode_config_init(&dev->drm);if(ret)return ret;
    dev->drm.mode_config.funcs=&config_funcs;
    dev->drm.mode_config.min_width=dev->drm.mode_config.min_height=1;
    dev->drm.mode_config.max_width=dev->drm.mode_config.max_height=4096;
    ret=drm_connector_init(&dev->drm,&dev->connector,&connector_funcs,DRM_MODE_CONNECTOR_VIRTUAL);if(ret)return ret;
    drm_connector_helper_add(&dev->connector,&connector_helper);
    ret=drm_simple_display_pipe_init(&dev->drm,&dev->pipe,&pipe_funcs,formats,ARRAY_SIZE(formats),NULL,&dev->connector);if(ret)return ret;
    drm_mode_config_reset(&dev->drm);
    ret=drm_dev_register(&dev->drm,0);if(ret)return ret;
    pr_info("PJ4EDK2_DRM registered virtual KMS device\n");return 0;
}
static void remove_device(struct platform_device *platform)
{
    struct pj4edk2_device *dev=platform_get_drvdata(platform);
    drm_dev_unregister(&dev->drm);drm_atomic_helper_shutdown(&dev->drm);
}
static struct platform_driver platform_driver={.probe=probe,.remove=remove_device,.driver={.name="pj4edk2-drm"}};
static int __init start(void)
{
    int ret=platform_driver_register(&platform_driver);if(ret)return ret;
    pdev=platform_device_register_simple("pj4edk2-drm",-1,NULL,0);
    if(IS_ERR(pdev)){ret=PTR_ERR(pdev);platform_driver_unregister(&platform_driver);return ret;}
    return 0;
}
static void __exit stop(void){platform_device_unregister(pdev);platform_driver_unregister(&platform_driver);}
module_init(start);module_exit(stop);
MODULE_LICENSE("GPL");MODULE_DESCRIPTION("PocketJS virtual DRM protocol test device");
