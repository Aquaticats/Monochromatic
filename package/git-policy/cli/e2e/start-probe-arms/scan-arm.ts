/**
 M7's failure branches: a mounted source before the run, a late mount during it,
 and a pinned empty namespace.

 The late mount must land after the probe has mounted its source and before its
 first scan. The exclusive create makes no row, so the arm waits for the probe's own
 child to show the source in its namespace and then releases a mounter that has been
 waiting ready in its own namespace. The whole-table scan the other arms use takes
 about two seconds in this guest (242 namespaces), longer than the hold, so both
 waits read the one namespace they are about.

 @example
 ```ts
 await m7Scan({ chosen });
 ```
 */

import type {
  GuestFacts,
} from '../../bin/mvm-guest-guest-facts.ts';
import {
  guest,
} from '../../bin/mvm-guest-script.ts';


import {
  armTimeoutMilliseconds,
  probeDataset,
  probeScript,
} from './probe-fixture.ts';

/**
 The log M7's arms write inside the run's log folder.
 */
const scanLog = 'guest-m7-scan.log';

/**
 Run M7's three arms.

 @param chosen - the run's guest and log folder

 @throws Error naming the guest's status, which a failing check makes nonzero

 @example
 ```ts
 await m7Scan({ chosen });
 ```
 */
export async function m7Scan({
  chosen,
}: {
  readonly chosen: GuestFacts;
},): Promise<void> {
  /**
   The arms: the baseline scans, a source held before the run, a mount that lands
   during it, and a namespace pinned empty while no task is left in it.
   */
  const script = probeScript({
    lines: [
      '$H start-probe > /root/m7-baseline.log 2>&1 || true',
      'if grep -q "^step scan-while-held passed" $REC && grep -q "^step scan-after-holder passed" $REC && grep -q "^step closing-scan passed" $REC; then echo "M7 baseline-scans-passed"; else echo "M7 baseline-scans-failed"; cat $REC; exit 1; fi',
      'mkdir --parents /root/m7-held-mount',
      `unshare --mount sh -c 'mount -t zfs ${probeDataset}@ready /root/m7-held-mount && touch /root/m7-held.txt && sleep 30' &`,
      'for wait in 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17 18 19 20; do test -f /root/m7-held.txt && break; sleep 0.5; done',
      'test -f /root/m7-held.txt',
      '$H start-probe > /root/m7-held.log 2>&1 || true',
      'if grep -q "^step attach-exclusive-create not-exercised" $REC && grep -q "reason source-held" $REC && grep -q "errno EBUSY" $REC; then echo "M7 held-source-not-exercised"; else echo "M7 held-source-not-seen"; cat $REC; exit 1; fi',
      'pkill --signal KILL --full "^/sbin/mount.zfs" || true',
      'wait || true',
      'mkdir --parents /root/m7-late',
      // The late mount must land after the probe has mounted its source and before its first scan. The exclusive create
      // makes no row, so the arm waits for the probe's own child to show the source in its namespace and then releases a
      // mounter that has been waiting ready in its own namespace. The whole-table scan the other arms use takes about
      // two seconds in this guest (242 namespaces), longer than the hold, so both waits read the one namespace they are
      // about.
      'rm --force /root/m7-go',
      `unshare --mount sh -c 'while test ! -f /root/m7-go; do sleep 0.01; done; mount -t zfs ${probeDataset}@ready /root/m7-late; exec sleep 30' > /dev/null 2>&1 & late=$!`,
      'CLI_GIT_TEST_ONLY_START_PROBE=hold-attach-place CLI_GIT_TEST_ONLY_START_PROBE_DEADLINE_MS=900000 $R start-probe > /root/m7-late-run.log 2>&1 & probe=$!',
      'attached=false',
      'for attempt in 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17 18 19 20 21 22 23 24 25 26 27 28 29 30 31 32 33 34 35 36 37 38 39 40; do',
      '  for child in $(cat /proc/$probe/task/$probe/children 2>/dev/null); do',
      `    if nsenter --mount="/proc/$child/ns/mnt" cat /proc/self/mountinfo 2>/dev/null | grep -q "git-worktree-probe@ready"; then attached=true; break; fi`,
      '  done',
      '  test "$attached" = true && break',
      '  sleep 0.02',
      'done',
      'test "$attached" = true || { echo "M7 the probe never attached"; cat /root/m7-late-run.log; kill -KILL $late 2>/dev/null || true; exit 1; }',
      'touch /root/m7-go',
      'landed=false',
      'for attempt in 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17 18 19 20 21 22 23 24 25 26 27 28 29 30 31 32 33 34 35 36 37 38 39 40; do',
      `  if nsenter --mount="/proc/$late/ns/mnt" cat /proc/self/mountinfo 2>/dev/null | grep -q "git-worktree-probe@ready"; then landed=true; break; fi`,
      '  sleep 0.02',
      'done',
      'test "$landed" = true || { echo "M7 the late mount never landed"; cat /root/m7-late-run.log; kill -KILL $late 2>/dev/null || true; exit 1; }',
      'wait $probe || true',
      'kill -KILL $late 2>/dev/null || true',
      'wait $late 2>/dev/null || true',
      'if grep -q "^step scan-while-held failed" $REC; then echo "M7 late-mount-failed-the-first-pass"; else echo "M7 late-mount-not-seen"; cat $REC; exit 1; fi',
      'mkdir --parents /root/m7-empty',
      'unshare --mount sh -c "exec sleep 30" > /dev/null 2>&1 & pinned=$!',
      'exec 9</proc/$pinned/ns/mnt',
      'kill -KILL $pinned 2>/dev/null || true',
      'wait $pinned 2>/dev/null || true',
      '$H start-probe > /root/m7-empty-run.log 2>&1 || true',
      'exec 9<&-',
      'if grep -q "^verdict passed" $REC; then echo "M7 empty-pinned-namespace-invisible"; else echo "M7 empty-pinned-namespace-named"; cat $REC; exit 1; fi',
      'echo "M7 scan arms done"',
    ],
  },);
  await guest({
    chosen,
    script,
    log: scanLog,
    timeoutMilliseconds: armTimeoutMilliseconds,
  },);
}
