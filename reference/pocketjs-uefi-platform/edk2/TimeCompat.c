/* Use edk2-libc's real ABI: time_t, timeval and timespec fields are 32 bits. */
#include <Uefi.h>
#include <sys/time.h>
#include <errno.h>

int clock_gettime(clockid_t Clock, struct timespec *Result)
{
  UINT64 Ticks, Frequency;
  struct timeval Wall;
  if (Result == NULL) {
    errno = EINVAL;
    return -1;
  }
  if (Clock == CLOCK_REALTIME) {
    if (gettimeofday(&Wall, NULL) != 0)
      return -1;
    TIMEVAL_TO_TIMESPEC(&Wall, Result);
    return 0;
  }
  if (Clock == CLOCK_MONOTONIC) {
    __asm__ volatile("mrs %0, cntvct_el0" : "=r"(Ticks));
    __asm__ volatile("mrs %0, cntfrq_el0" : "=r"(Frequency));
    if (Frequency != 0) {
      Result->tv_sec = (time_t)(Ticks / Frequency);
      Result->tv_nsec = (LONG32)((Ticks % Frequency) * 1000000000ULL / Frequency);
      return 0;
    }
  }
  errno = EINVAL;
  return -1;
}
