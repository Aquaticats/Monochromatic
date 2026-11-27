/* poll_the_published_name: counts how often the name a publish replaces is absent while that publish runs.
 *
 * usage: poll_the_published_name <path> <span-ms> <pause-us> <inode-of-the-copy-being-replaced>
 *
 * One poll is one stat of <path>, so a poll costs a few microseconds and the poller can resolve an absence of tens
 * of microseconds:
 *
 *   - the stat answers ENOENT: the name is absent.  This is the state a concurrent start meets as ENOENT, not as
 *     the busy-program refusal.
 *   - the stat answers an inode equal to the one given: the copy the publish replaces still holds the name.
 *   - any other inode: the finished copy holds the name.
 *
 * One absence is counted from the poll that first finds nothing to the first poll that finds a file again, and
 * its length is that pair's wall-clock difference, which is therefore an upper bound for the true absence by one
 * poll's own cost.
 *
 * The poller never writes the path, never opens it, and holds no descriptor on it, so it cannot itself make a
 * start fail.
 */
#define _POSIX_C_SOURCE 200809L

#include <errno.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/stat.h>
#include <time.h>

static long long now_us(void) {
  struct timespec moment;
  clock_gettime(CLOCK_MONOTONIC, &moment);
  return (long long)moment.tv_sec * 1000000LL + moment.tv_nsec / 1000;
}

/* What the name holds this moment: 0 absent, 1 the copy being replaced, 2 another inode, 3 an error. */
static int classify(const char *path, long long replaced_inode) {
  struct stat record;
  if (stat(path, &record) == 0) {
    return (long long)record.st_ino == replaced_inode ? 1 : 2;
  }
  return errno == ENOENT ? 0 : 3;
}

int main(int argc, char **argv) {
  if (argc != 5) {
    fprintf(stderr, "usage: %s <path> <span-ms> <pause-us> <inode-of-the-copy-being-replaced>\n", argv[0]);
    return 2;
  }
  const char *path = argv[1];
  const long long span_us = atoll(argv[2]) * 1000LL;
  const long long pause_us = atoll(argv[3]);
  const long long replaced_inode = atoll(argv[4]);
  unsigned long polls = 0, absent_polls = 0, old = 0, new = 0, other = 0, absences = 0;
  long long longest_us = 0, absent_since = 0;
  int previous = -1;
  int first_seen = -1, after_absence = -1;
  const long long start = now_us();
  for (;;) {
    long long moment = now_us();
    if (moment - start >= span_us) {
      break;
    }
    int state = classify(path, replaced_inode);
    polls++;
    if (state == 0) {
      absent_polls++;
      if (previous != 0) {
        absent_since = moment;
        absences++;
      }
    } else {
      if (state == 1) {
        old++;
      } else if (state == 2) {
        new++;
      } else {
        other++;
      }
      if (previous == 0) {
        long long length = moment - absent_since;
        if (length > longest_us) {
          longest_us = length;
        }
        if (after_absence < 0 && state != 3) {
          after_absence = state;
        }
      }
      if (first_seen < 0 && state != 3) {
        first_seen = state;
      }
    }
    previous = state;
    if (pause_us > 0) {
      struct timespec rest = {(time_t)(pause_us / 1000000LL), (long)((pause_us % 1000000LL) * 1000LL)};
      nanosleep(&rest, NULL);
    }
  }
  const char *seen = first_seen < 0 ? "nothing" : first_seen == 1 ? "old" : first_seen == 2 ? "new" : "other";
  const char *then = after_absence < 0 ? "" : after_absence == 1 ? ">old" : after_absence == 2 ? ">new" : ">other";
  printf("POLL path=%s span_ms=%s pause_us=%s replaced_inode=%lld polls=%lu absent=%lu old=%lu new=%lu other=%lu absences=%lu longest_absence_us=%lld seen=%s%s\n",
         path, argv[2], argv[3], replaced_inode, polls, absent_polls, old, new, other, absences, longest_us, seen, then);
  return 0;
}
