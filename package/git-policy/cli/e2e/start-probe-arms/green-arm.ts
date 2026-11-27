/**
 The arms the design's measurements name, all in one guest script so every reading
 is in one log: M1's attach and its `EBUSY` control, M2's table and txg fences, M3's
 kills and its held deadline, M4's timings, M5's refusal controls, M7's scan, M9's
 accepted release and its filtered refusal, and M10's absent-source reading.

 Every claim is asserted from the record itself, and a missing line fails the arm
 rather than printing nothing, so a run that returned without performing an arm is
 a failed run.

 @example
 ```ts
 await green({ chosen });
 ```
 */

import type { GuestRun, } from '../guest-host/guest-run.ts';
import { guest, } from '../guest-host/guest-side-scripts.ts';
import {
  armTimeoutMilliseconds,
  configuration,
  handler,
  openZfsPath,
  openZfsPrefix,
  pool,
  probeDataset,
  record,
} from './probe-fixture.ts';

/**
 The log the green arm writes inside the run's log folder.
 */
export const greenLog = 'guest-green.log';

/**
 The arms, in the order the design measures them.

 @param chosen - the run's guest and log folder

 @throws Error naming the guest's status, which a failing check makes nonzero

 @example
 ```ts
 await green({ chosen });
 ```
 */
export async function green({
  chosen,
}: {
  readonly chosen: GuestRun;
},): Promise<void> {
  /**
   The whole green script: five interleaved rounds, then every claim the record
   has to hold.
   */
  const script = [
    'set -eu',
    `export PATH=${openZfsPath}`,
    `export LD_LIBRARY_PATH=${openZfsPrefix}/lib`,
    // Every claim is asserted from the record itself, and a missing line fails the arm rather than printing nothing.
    'check() {',
    '  if ! grep --quiet --fixed-strings "$1" "$2"; then',
    '    echo "MISSING $1 in $2"',
    '    cat "$2"',
    '    exit 1',
    '  fi',
    '  echo "PROBE $3"',
    '}',
    `sed --in-place 's|${pool}/nosnap/git-worktree-probe-absent|${probeDataset}|' ${configuration}`,
    'cat /proc/self/mountinfo | sort > /root/mountinfo-before.txt',
    `zpool sync ${pool}`,
    `cat /proc/spl/kstat/zfs/${pool}/txgs > /root/txgs-before.txt`,
    't0=$(awk \'NR > 2 { if ($1 > top) top = $1 } END { print top }\' /root/txgs-before.txt)',
    'echo "PROBE start fence $t0"',
    // M4: five interleaved rounds, and the control is the same binary with the probe disabled.
    'for round in 1 2 3 4 5; do',
    '  start=$(date +%s%N)',
    `  ${handler} start-probe > /root/probe-run.log 2>&1 || true`,
    '  end=$(date +%s%N)',
    '  echo "ROUND $round $(( (end - start) / 1000 )) us"',
    `  cp ${record} /root/record-probe.txt`,
    '  control_start=$(date +%s%N)',
    `  CLI_GIT_TEST_ONLY_START_PROBE=disabled ${handler} start-probe > /root/probe-control.log 2>&1 || true`,
    '  control_end=$(date +%s%N)',
    '  echo "CONTROL $round $(( (control_end - control_start) / 1000 )) us"',
    `  cp ${record} /root/record-control.txt`,
    'done',
    'echo "PROBE record after the passing run:"; cat /root/record-probe.txt',
    `check 'verdict passed' /root/record-probe.txt 'verdict passed'`,
    `check 'step attach-exclusive-create passed' /root/record-probe.txt 'attach passed'`,
    `check 'step walk-refusal-controls passed' /root/record-probe.txt 'refusal controls refused'`,
    `check 'step scan-while-held passed' /root/record-probe.txt 'scan saw the worker and the holder'`,
    `check 'step scan-after-holder passed' /root/record-probe.txt 'scan saw the worker alone'`,
    `check 'step move passed' /root/record-probe.txt 'move passed'`,
    `check 'step unmount passed' /root/record-probe.txt 'unmount passed'`,
    `check 'step closing-scan passed' /root/record-probe.txt 'closing scan clear'`,
    `check 'verdict skipped' /root/record-control.txt 'disabled control wrote a skipped record'`,
    'echo "PROBE disabled control record:"; cat /root/record-control.txt',
    // M2: the end fence commits whatever the probe wrote, and every committed row at or above the start row is empty.
    `zpool sync ${pool}`,
    `cat /proc/spl/kstat/zfs/${pool}/txgs > /root/txgs-after.txt`,
    'if awk -v t0="$t0" \'NR > 2 && $1 >= t0 && ($4 != 0 || $6 != 0 || $8 != 0) { bad = 1 } NR > 2 && $1 >= t0 { seen = 1 } END { exit !(seen && !bad) }\' /root/txgs-after.txt; then',
    '  echo "PROBE txg window clean"',
    'else',
    '  echo "PROBE txg window held a write"; awk -v t0="$t0" \'NR > 2 && $1 >= t0\' /root/txgs-after.txt; exit 1',
    'fi',
    'cat /proc/self/mountinfo | sort > /root/mountinfo-after.txt',
    'if cmp --silent /root/mountinfo-before.txt /root/mountinfo-after.txt; then echo "PROBE host table unchanged"; else echo "PROBE host table changed"; exit 1; fi',
    'for namespace in /proc/[0-9]*/ns/mnt; do',
    '  if nsenter --mount="$namespace" cat /proc/self/mountinfo 2>/dev/null | grep --quiet "git-worktree-probe@ready"; then echo "PROBE source still mounted in $namespace"; exit 1; fi',
    'done',
    'echo "PROBE source absent afterwards"',
    // M1's held-source control: a second namespace mounts the snapshot, so the exclusive create answers EBUSY.
    'rm --force /root/held.txt',
    'rm --recursive --force /root/held-mount',
    'mkdir --parents /root/held-mount',
    `unshare --mount sh -c '/sbin/mount.zfs ${probeDataset}@ready /root/held-mount && touch /root/held.txt && sleep 30' &`,
    'for wait in 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17 18 19 20; do test -f /root/held.txt && break; sleep 0.5; done',
    'test -f /root/held.txt',
    `${handler} start-probe > /root/probe-held.log 2>&1 || true`,
    `cp ${record} /root/record-held.txt`,
    `check 'step attach-exclusive-create not-exercised' /root/record-held.txt 'held source not exercised'`,
    `check 'reason source-held' /root/record-held.txt 'held source names its cause'`,
    `check 'errno EBUSY' /root/record-held.txt 'held source answers EBUSY'`,
    'pkill --signal KILL --full "^/sbin/mount.zfs" || true',
    'wait || true',
    // M10's absent-source reading: the configuration names a dataset the pool does not hold.
    `sed --in-place 's|${probeDataset}|${pool}/nosnap/git-worktree-probe-absent|' ${configuration}`,
    `${handler} start-probe > /root/probe-absent.log 2>&1 || true`,
    `sed --in-place 's|${pool}/nosnap/git-worktree-probe-absent|${probeDataset}|' ${configuration}`,
    `cp ${record} /root/record-absent.txt`,
    `check 'source-existed no' /root/record-absent.txt 'absent source recorded'`,
    `check 'reason missing-source' /root/record-absent.txt 'absent source names its cause'`,
    `check 'errno ENOENT' /root/record-absent.txt 'absent source answers ENOENT'`,
    `check 'verdict blocked' /root/record-absent.txt 'absent source blocks the route'`,
    // M9's refused arm: a filtered move_mount answers EPERM, which fails that step and refuses the probe.
    `systemd-run --unit=git-worktree-probe-test --quiet --wait --collect --property=SystemCallFilter=~move_mount --property=SystemCallErrorNumber=EPERM ${handler} start-probe || true`,
    `cp ${record} /root/record-refused.txt`,
    `check 'step attach-place failed' /root/record-refused.txt 'refused step attach-place errno EPERM'`,
    `check 'errno EPERM' /root/record-refused.txt 'refused step errno EPERM'`,
    // M3's held arm: a test-only hold after the move makes the deadline fire and names the step in flight.
    `CLI_GIT_TEST_ONLY_START_PROBE=hold-move ${handler} start-probe > /root/probe-hold.log 2>&1 || true`,
    `cp ${record} /root/record-timeout.txt`,
    `check 'verdict timed-out' /root/record-timeout.txt 'timed-out'`,
    `check 'in-flight unmount' /root/record-timeout.txt 'timed-out in flight unmount'`,
    `check 'last-finished move' /root/record-timeout.txt 'timed-out last finished move'`,
    // M3's kill arm: hold-unshare parks the worker before any mount,
    // then a kill of the mode leaves the started record and no worker mount.
    // The wait is on the record the mode writes before its fork, bounded, because a fixed sleep races that write:
    // the integration pass of 2026-11-22 saw this control fail in 5 of 10 runs for exactly that reason.
    `CLI_GIT_TEST_ONLY_START_PROBE=hold-unshare ${handler} start-probe > /root/probe-kill.log 2>&1 & mode=$!`,
    'started=false',
    'for attempt in 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17 18 19 20 21 22 23 24 25 26 27 28 29 30 31 32 33 34 35 36 37 38 39 40; do',
    `  if grep --quiet --fixed-strings "state started" ${record} 2>/dev/null; then started=true; break; fi`,
    '  sleep 0.05',
    'done',
    // A mode that never wrote the started record is a failure of this control rather than of the kill, and it names
    // itself instead of surfacing as a missing record line later.
    `test "$started" = true || { echo "MISSING the started record before the kill"; cat ${record} 2>/dev/null; exit 1; }`,
    'kill -KILL "$mode" || true',
    'wait "$mode" 2>/dev/null || true',
    `pkill --signal KILL --full "^${handler} start-probe$" || true`,
    'sleep 0.2',
    `cp ${record} /root/record-killed.txt`,
    `check 'state started' /root/record-killed.txt 'killed mode left a started record'`,
    'echo "GREEN done"',
  ].join('\n',);
  await guest({
    chosen,
    script,
    log: greenLog,
    timeout: armTimeoutMilliseconds,
  },);
}
