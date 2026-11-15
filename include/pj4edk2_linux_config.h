/* SPDX-License-Identifier: BSD-2-Clause */
#ifndef PJ4EDK2_LINUX_CONFIG_H
#define PJ4EDK2_LINUX_CONFIG_H
#include "pj4edk2_linux_chosen.h"
/* ASCII key=value configuration; paths are absolute within the loaded ESP. */
struct pj4edk2_linux_config {
    char kernel[256], initramfs[256], dtb[256], metadata[256];
    char root_partuuid[37], options[1024];
};
static inline int pj4edk2_config_key(const char *p, unsigned long n, const char *key)
{
    unsigned long i;
    for (i = 0; i < n; ++i) if (!key[i] || p[i] != key[i]) return 0;
    return !key[n];
}
static inline int pj4edk2_config_path(const char *p, unsigned long n)
{
    unsigned long i, start = 1;
    if (n < 2 || p[0] != '/' || p[n-1] == '/') return 0;
    for (i = 1; i <= n; ++i) {
        if (i < n && (p[i] < 33 || p[i] > 126 || p[i] == '\\' || p[i] == ':')) return 0;
        if (i == n || p[i] == '/') {
            if (i == start || (i-start == 1 && p[start] == '.') ||
                (i-start == 2 && p[start] == '.' && p[start+1] == '.')) return 0;
            start = i+1;
        }
    }
    return 1;
}
static inline int pj4edk2_linux_config_parse(const char *text, unsigned long bytes,
                                            struct pj4edk2_linux_config *out)
{
    unsigned long at = 0, i; unsigned int seen = 0;
    if (!text || !out || !bytes || bytes > 4096) return 0;
    for (i = 0; i < sizeof(*out); ++i) ((char *)out)[i] = 0;
    while (at < bytes) {
        unsigned long start = at, end, equal, keyend, value, n, capacity;
        unsigned int bit; char *destination;
        while (at < bytes && text[at] != '\n') {
            unsigned char c = (unsigned char)text[at++];
            if ((c < 32 && c != '\t' && c != '\r') || c > 126) return 0;
        }
        end = at; if (at < bytes) ++at;
        if (end > start && text[end-1] == '\r') --end;
        while (start < end && (text[start] == ' ' || text[start] == '\t')) ++start;
        while (end > start && (text[end-1] == ' ' || text[end-1] == '\t')) --end;
        if (start == end || text[start] == '#') continue;
        for (equal = start; equal < end && text[equal] != '='; ++equal) {}
        if (equal == end) return 0;
        keyend = equal;
        while (keyend > start && (text[keyend-1] == ' ' || text[keyend-1] == '\t')) --keyend;
        value = equal+1;
        while (value < end && (text[value] == ' ' || text[value] == '\t')) ++value;
        n = end-value;
#define PJ4EDK2_CONFIG_FIELD(key, field, mask) \
        if (pj4edk2_config_key(text+start,keyend-start,key)) { \
            destination = out->field; capacity = sizeof(out->field); bit = mask; \
        }
        PJ4EDK2_CONFIG_FIELD("kernel",kernel,1)
        else PJ4EDK2_CONFIG_FIELD("initramfs",initramfs,2)
        else PJ4EDK2_CONFIG_FIELD("dtb",dtb,4)
        else PJ4EDK2_CONFIG_FIELD("metadata",metadata,8)
        else PJ4EDK2_CONFIG_FIELD("root-partuuid",root_partuuid,16)
        else PJ4EDK2_CONFIG_FIELD("options",options,32)
        else return 0;
#undef PJ4EDK2_CONFIG_FIELD
        if ((seen & bit) || n >= capacity || (!n && bit != 32)) return 0;
        if (bit < 16 && !pj4edk2_config_path(text+value,n)) return 0;
        if (bit == 16 && !pj4edk2_linux_partuuid_valid(text+value,n)) return 0;
        for (i = 0; i < n; ++i) {
            if (text[value+i] == '\r') return 0;
            destination[i] = text[value+i];
        }
        destination[n] = 0; seen |= bit;
    }
    return (seen & 31) == 31;
}
#endif
