/* SPDX-License-Identifier: BSD-2-Clause */
#ifndef PJB_LINUX_CHOSEN_H
#define PJB_LINUX_CHOSEN_H
/* linux,initrd-end is exclusive. Reject zero-length and wrapping ranges. */
static inline int pjb_linux_initrd_range(unsigned long long start,
        unsigned long long bytes, unsigned long long *end)
{
    if (!end || !bytes || start > ~0ULL - bytes)
        return 0;
    *end = start + bytes;
    return 1;
}
#endif
