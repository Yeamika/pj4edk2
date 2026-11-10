#include <assert.h>
#include <stdio.h>
#include "pjb_linux_chosen.h"
int main(void)
{
    unsigned long long end=123;
    assert(!pjb_linux_initrd_range(0xd0000000,0,&end) && end==123);
    assert(!pjb_linux_initrd_range(~0ULL-10,11,&end) && end==123);
    assert(!pjb_linux_initrd_range(1,1,0));
    assert(pjb_linux_initrd_range(0xd0000000,0x1000,&end) && end==0xd0001000);
    assert(pjb_linux_initrd_range(~0ULL-10,10,&end) && end==~0ULL);
    puts("PASS: initrd exclusive-end, empty and overflowing ranges");
    return 0;
}
