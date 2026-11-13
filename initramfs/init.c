/* SPDX-License-Identifier: BSD-2-Clause */
/* Tiny static PID 1: no distribution, shell, systemd, or GPU userspace needed. */
#define _GNU_SOURCE
#include <fcntl.h>
#include <stdio.h>
#include <stdarg.h>
#include <string.h>
#include <sys/ioctl.h>
#include <sys/mount.h>
#include <sys/stat.h>
#include <sys/syscall.h>
#include <errno.h>
#include <unistd.h>
#include <linux/fb.h>
#include "pj4edk2_boot_display.h"
struct drm_version {int major,minor,patch;size_t name_len;char *name;size_t date_len;char *date;size_t desc_len;char *desc;};
#define DRM_VERSION _IOWR('d',0,struct drm_version)
/* One record per message keeps PID1 diagnostics below /dev/kmsg rate limits. */
static int message(const char *format,...)
{
    char buffer[1024];va_list args;int n;
    va_start(args,format);n=vsnprintf(buffer,sizeof(buffer),format,args);va_end(args);
    if(n<0)return n;
    if((size_t)n>=sizeof(buffer))n=sizeof(buffer)-1;
    return (int)write(1,buffer,n);
}
#define printf message
#define puts(text) message("%s\n",text)
int main(void)
{
    int fd=-1, console, tries, retained=0;

    char driver[64]={0};
    struct drm_version version={.name_len=sizeof(driver)-1,.name=driver};
    mkdir("/dev",0755);mkdir("/proc",0755);mkdir("/sys",0755);
    mount("devtmpfs","/dev","devtmpfs",0,NULL);
    mount("proc","/proc","proc",0,NULL);mount("sysfs","/sys","sysfs",0,NULL);
    console=open("/dev/console",O_RDWR);
    if(console>=0){dup2(console,0);dup2(console,1);dup2(console,2);}
    fd=open("/dev/kmsg",O_WRONLY);
    if(fd>=0){dup2(fd,1);close(fd);}
    setvbuf(stdout,NULL,_IONBF,0);
    puts("PJ4EDK2_SMALL_INIT started");
    retained=access("/proc/device-tree/chosen/pocketjs,boot-display-memory",F_OK)==0;
    if(!retained) {
    for(tries=0;tries<100;++tries){fd=open("/dev/dri/card0",O_RDWR);if(fd>=0)break;
            usleep(100000);}
    if(fd>=0 && ioctl(fd,DRM_VERSION,&version)==0 && !strcmp(driver,"virtio_gpu"))
        printf("PJ4EDK2_DRM PASS driver=%s version=%d.%d\n",driver,version.major,version.minor);
    else printf("PJ4EDK2_DRM FAIL card0 missing or unexpected driver=%s\n",driver);
    if(fd>=0)close(fd);
    }
    fd=open("/proc/device-tree/chosen/" PJ4EDK2_DISPLAY_PROPERTY,O_RDONLY);
    if(fd>=0){
        unsigned char wire[81];unsigned int cells[20];ssize_t n=read(fd,wire,sizeof(wire));close(fd);
        puts(n==80 && pj4edk2_display_decode_be(wire,n,cells) ?
            "PJ4EDK2_DT PASS display protocol" : "PJ4EDK2_DT FAIL invalid fixture");
    }else puts("PJ4EDK2_DT absent (no fixture DTB supplied)");
    fd=open("/pj4edk2_drm.ko",O_RDONLY);
    if(fd>=0){
        if(syscall(SYS_finit_module,fd,"",0)==0)puts("PJ4EDK2_MODULE loaded");
        else printf("PJ4EDK2_MODULE FAIL errno=%d\n",errno);
        close(fd);
        for(tries=0;tries<30;++tries){
            fd=open(retained?"/dev/dri/card0":"/dev/dri/card1",O_RDWR);
            if(fd>=0)break;
            usleep(100000);
        }
        memset(driver,0,sizeof(driver));version.name_len=sizeof(driver)-1;
        if(fd>=0 && ioctl(fd,DRM_VERSION,&version)==0 && !strcmp(driver,"pj4edk2"))
            puts("PJ4EDK2_MODULE PASS virtual DRM/KMS registered");
        else puts("PJ4EDK2_MODULE FAIL virtual DRM missing");
        if(fd>=0)close(fd);
    }else puts("PJ4EDK2_MODULE absent (embedded-only smoke initramfs)");
    if(!retained) {
    fd=open("/sys/class/graphics/fb0/name",O_RDONLY);
    if(fd>=0){char name[64]={0};ssize_t n=read(fd,name,sizeof(name)-1);close(fd);
        if(n>0)printf("PJ4EDK2_FB PASS %s",name);else puts("PJ4EDK2_FB FAIL read");
    }else puts("PJ4EDK2_FB FAIL no fbdev emulation");
    }
    puts("PJ4EDK2_SMALL_READY: DRM test finished; screenshot the console or stop QEMU.");
    for(;;)pause();
}
