/* SPDX-License-Identifier: BSD-2-Clause */
#include <assert.h>
#include <stdio.h>
#include <string.h>
#include "../include/pj4edk2_linux_config.h"
#define BASE "kernel=/EFI/Test/Image\ninitramfs=/EFI/Test/initramfs.lz4\ndtb=/EFI/Test/board.dtb\nmetadata=/EFI/Test/payload.bin\nroot-partuuid=01234567-89ab-cdef-0123-456789abcdef\n"
static int parse(const char *s) { struct pj4edk2_linux_config c; return pj4edk2_linux_config_parse(s,strlen(s),&c); }
int main(int argc, char **argv) {
    if (argc == 2) {
        FILE *file = fopen(argv[1], "rb");
        char text[4097]; size_t bytes;
        struct pj4edk2_linux_config selected;
        assert(file); bytes = fread(text, 1, sizeof(text), file); assert(!ferror(file)); fclose(file);
        assert(pj4edk2_linux_config_parse(text, bytes, &selected));
        puts("PASS: generated ESP LinuxBoot configuration parsed");
        return 0;
    }
    struct pj4edk2_linux_config c;
    const char good[] = BASE "options=systemd.unit=graphical.target quiet\n";
    assert(pj4edk2_linux_config_parse(good,sizeof(good)-1,&c));
    assert(!strcmp(c.kernel,"/EFI/Test/Image"));
    assert(!strcmp(c.options,"systemd.unit=graphical.target quiet"));
    assert(parse(BASE));
    assert(!parse(BASE "kernel=/Other/Image\n"));
    assert(!parse(BASE "unexpected=yes\n"));
    assert(!parse("kernel=/EFI/Test/Image\n"));
    assert(!parse(BASE "options=bad\rcommand\n"));
    assert(!pj4edk2_config_path("/EFI/../Image",13));
    assert(!pj4edk2_config_path("/EFI//Image",11));
    assert(!pj4edk2_config_path("relative/Image",14));
    assert(!pj4edk2_linux_config_parse(good,4097,&c));
    puts("PASS: Linux EFI config values, required fields, duplicates, paths and bounds");
    return 0;
}
