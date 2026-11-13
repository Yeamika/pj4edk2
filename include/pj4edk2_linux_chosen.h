/* SPDX-License-Identifier: BSD-2-Clause */
#ifndef PJ4EDK2_LINUX_CHOSEN_H
#define PJ4EDK2_LINUX_CHOSEN_H
/* linux,initrd-end is exclusive. Reject zero-length and wrapping ranges. */
static inline int pj4edk2_linux_initrd_range(unsigned long long start,
        unsigned long long bytes, unsigned long long *end)
{
    if (!end || !bytes || start > ~0ULL - bytes)
        return 0;
    *end = start + bytes;
    return 1;
}
/* Canonical GPT UUID, without terminator. No whitespace or embedded NUL. */
#define PJ4EDK2_ROOT_PROPERTY "pj4edk2,root-partuuid"
static inline int pj4edk2_linux_partuuid_valid(const char *text, unsigned long bytes)
{
    unsigned long i;
    if (!text || bytes != 36) return 0;
    for (i = 0; i < 36; ++i) {
        unsigned char c = (unsigned char)text[i];
        if (i == 8 || i == 13 || i == 18 || i == 23) {
            if (c != '-') return 0;
        } else if (!((c >= '0' && c <= '9') ||
                     (c >= 'a' && c <= 'f') || (c >= 'A' && c <= 'F'))) return 0;
    }
    return 1;
}
#endif
