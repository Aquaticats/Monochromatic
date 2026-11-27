/**
 Two arms whose subject is a refusal: F10's gate refusal, where a real gate refuses
 in a unit without `/dev/zfs` and the mode writes a skipped record, and M9's extra
 descriptor, where the unmount must answer `EBUSY` with the switch on and pass with
 it off.

 @example
 ```ts
 await gateSkip({ chosen });
 ```
 */

import type { GuestRun, } from '../guest-host/guest-run.ts';
import { guest, } from '../guest-host/guest-side-scripts.ts';
import {
  armTimeoutMilliseconds,
  probeScript,
} from './probe-fixture.ts';

/**
 The log F10's gate refusal writes inside the run's log folder.
 */
const skipLog = 'guest-skip.log';

/**
 The log M9's descriptor fixture writes inside the run's log folder.
 */
const busyLog = 'guest-m9-busy.log';

/**
 Run F10's gate refusal: the mode in a unit that cannot open the device.

 @param chosen - the run's guest and log folder

 @throws Error naming the guest's status, which a failing check makes nonzero

 @example
 ```ts
 await gateSkip({ chosen });
 ```
 */
export async function gateSkip({
  chosen,
}: {
  readonly chosen: GuestRun;
},): Promise<void> {
  /**
   The arm: the mode in a unit whose `/dev/zfs` is inaccessible, then the record
   and the table.
   */
  const script = probeScript({
    lines: [
      'rm -f $REC',
      'sort /proc/self/mountinfo | sed "s/^[0-9]* //" > /root/skip-table-before.txt',
      'systemd-run --unit=git-worktree-probe-skip-test --quiet --wait --collect --pipe --property=InaccessiblePaths=/dev/zfs $H start-probe > /root/skip-run.log 2>&1; echo "SKIP exit=$?"',
      'cat /root/skip-run.log',
      'cat $REC',
      'if grep -q "^verdict skipped" $REC && grep -q "^skip-reason device" $REC; then echo "SKIP gate-refusal-skipped"; else echo "SKIP gate-refusal-not-skipped"; exit 1; fi',
      'sort /proc/self/mountinfo | sed "s/^[0-9]* //" > /root/skip-table-after.txt',
      'if cmp --silent /root/skip-table-before.txt /root/skip-table-after.txt; then echo "SKIP host-table-unchanged"; else echo "SKIP host-table-changed"; exit 1; fi',
      'leaks | tail -n 1',
      'echo "SKIP done"',
    ],
  },);
  await guest({
    chosen,
    script,
    log: skipLog,
    timeout: armTimeoutMilliseconds,
  },);
}

/**
 Run M9's extra-descriptor fixture: the unmount answers `EBUSY` with a descriptor
 held, and the probe passes without it.

 @param chosen - the run's guest and log folder

 @throws Error naming the guest's status, which a failing check makes nonzero

 @example
 ```ts
 await m9Busy({ chosen });
 ```
 */
export async function m9Busy({
  chosen,
}: {
  readonly chosen: GuestRun;
},): Promise<void> {
  /**
   The arm: one run with the switch on, and one with it off.
   */
  const script = probeScript({
    lines: [
      'rm -f $REC',
      'CLI_GIT_TEST_ONLY_START_PROBE_KEEP_DESCRIPTOR=1 $H start-probe > /root/m9-on.log 2>&1 || true',
      'cp $REC /root/m9-on-record.txt',
      'if grep -q "^step unmount failed errno EBUSY" /root/m9-on-record.txt; then echo "M9 extra-descriptor-unmount-EBUSY"; else echo "M9 extra-descriptor-not-seen"; cat /root/m9-on-record.txt; exit 1; fi',
      '$H start-probe > /root/m9-off.log 2>&1 || true',
      'cp $REC /root/m9-off-record.txt',
      'if grep -q "^verdict passed" /root/m9-off-record.txt; then echo "M9 switch-off-passes"; else echo "M9 switch-off-did-not-pass"; cat /root/m9-off-record.txt; exit 1; fi',
      'echo "M9 fixture done"',
    ],
  },);
  await guest({
    chosen,
    script,
    log: busyLog,
    timeout: armTimeoutMilliseconds,
  },);
}
