#include <assert.h>
#include <stdio.h>
#include "pj4edk2_linux_chosen.h"
int main(void)
{
    unsigned long long end=123;
    assert(!pj4edk2_linux_initrd_range(0xd0000000,0,&end) && end==123);
    assert(!pj4edk2_linux_initrd_range(~0ULL-10,11,&end) && end==123);
    assert(!pj4edk2_linux_initrd_range(1,1,0));
    assert(pj4edk2_linux_initrd_range(0xd0000000,0x1000,&end) && end==0xd0001000);
    assert(pj4edk2_linux_initrd_range(~0ULL-10,10,&end) && end==~0ULL);
    {
        const char good[]="01234567-89ab-cdef-0123-456789ABCDEF";
        unsigned long i;
        char bad[37];
        assert(pj4edk2_linux_partuuid_valid(good,36));
        assert(!pj4edk2_linux_partuuid_valid(0,36));
        assert(!pj4edk2_linux_partuuid_valid(good,35));
        assert(!pj4edk2_linux_partuuid_valid(good,37));
        for (i=0;i<36;++i) {
            unsigned long j;
            for(j=0;j<37;++j) bad[j]=good[j];
            bad[i]='z';
            assert(!pj4edk2_linux_partuuid_valid(bad,36));
            bad[i]=0;
            assert(!pj4edk2_linux_partuuid_valid(bad,36));
        }
    }
    puts("PASS: initrd exclusive-end, empty and overflowing ranges");
    return 0;
}
