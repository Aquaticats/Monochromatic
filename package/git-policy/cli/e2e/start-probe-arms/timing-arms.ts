/**
 The two timing arms: warm runs, cache-dropped runs and a guest with many mount
 namespaces, every round kept; and the first run after a reboot with five warm
 rounds after it.

 Both arms alternate the probe with the same build with the probe disabled, so each
 pair is one interleaved comparison on unchanged input, and the guest's boot first
 probe is the cold reading no page cache is left over from.

 @example
 ```ts
 await timings({ chosen });
 ```
 */

import { wait, } from '@monochromatic-dev/async-time-core/ts';

import { guestBackend, } from '../../bin/mvm-guest-backend.ts';
import type {
  GuestFacts,
} from '../../bin/mvm-guest-guest-facts.ts';
import {
  guest,
} from '../../bin/mvm-guest-script.ts';
import {
  statusOf,
} from '../../bin/mvm-guest-step-command.ts';

import {
  armTimeoutMilliseconds,
  longArmTimeoutMilliseconds,
  pool,
  probeScript,
} from './probe-fixture.ts';

/**
 The backend this run drives, read from the one place every guest step reads it from.
 */
const backend = guestBackend();


/**
 The log the warm and cold timing arm writes inside the run's log folder.
 */
const timingsLog = 'guest-timings.log';

/**
 The log the boot-first arm writes inside the run's log folder.
 */
const bootFirstLog = 'guest-timings-boot-first.log';

/**
 How many times the host asks a rebooted guest whether it is back: a guest's
 reboot took most of a minute in this driver's own runs, so the count is generous
 and every answer is the guest's own.
 */
const rebootPolls = 180;

/**
 Milliseconds one answer from a rebooted guest may take.
 */
const rebootPollTimeoutMilliseconds = 10_000;

/**
 The pause between two answers from a rebooted guest, in milliseconds.
 */
const rebootPollPauseMilliseconds = 1_000;

/**
 Milliseconds the host gives the guest's own reboot command, which ends with the
 guest going down and so never answers by itself.
 */
const rebootTimeoutMilliseconds = 60_000;

/**
 Run the warm, cold and many-namespace rounds.

 @param chosen - the run's guest and log folder

 @throws Error naming the guest's status, which a failing check makes nonzero

 @example
 ```ts
 await timings({ chosen });
 ```
 */
export async function timings({
  chosen,
}: {
  readonly chosen: GuestFacts;
},): Promise<void> {
  /**
   The arm: five warm rounds, three cold ones with the page cache dropped, then
   five rounds each at a hundred and at three hundred extra namespaces.
   */
  const script = probeScript({
    lines: [
      'echo "TIMING start $(date --iso-8601=seconds)"',
      'for round in 1 2 3 4 5; do',
      '  p0=$(now_us); $H start-probe > /root/timing-probe.log 2>&1 || true; p1=$(now_us); pv=$(grep "^verdict " $REC | cut -d" " -f2)',
      '  c0=$(now_us); CLI_GIT_TEST_ONLY_START_PROBE=disabled $H start-probe > /root/timing-control.log 2>&1 || true; c1=$(now_us); cv=$(grep "^verdict " $REC | cut -d" " -f2)',
      '  echo "WARM round=$round probe=$((p1-p0)) control=$((c1-c0)) worker-estimate=$(( (p1-p0) - (c1-c0) )) probe-verdict=$pv control-verdict=$cv"',
      'done',
      'for round in 1 2 3; do',
      `  zpool export ${pool}`,
      '  sync; echo 3 > /proc/sys/vm/drop_caches',
      `  zpool import -d /home -o cachefile=none ${pool}`,
      '  sync; echo 3 > /proc/sys/vm/drop_caches',
      '  p0=$(now_us); $H start-probe > /root/timing-cold.log 2>&1 || true; p1=$(now_us); pv=$(grep "^verdict " $REC | cut -d" " -f2)',
      '  c0=$(now_us); CLI_GIT_TEST_ONLY_START_PROBE=disabled $H start-probe > /root/timing-control.log 2>&1 || true; c1=$(now_us); cv=$(grep "^verdict " $REC | cut -d" " -f2)',
      '  echo "COLD round=$round probe=$((p1-p0)) control=$((c1-c0)) worker-estimate=$(( (p1-p0) - (c1-c0) )) probe-verdict=$pv control-verdict=$cv"',
      '  p0=$(now_us); $H start-probe > /root/timing-after-cold.log 2>&1 || true; p1=$(now_us); pv=$(grep "^verdict " $REC | cut -d" " -f2)',
      '  echo "AFTER-COLD round=$round probe=$((p1-p0)) probe-verdict=$pv"',
      'done',
      String.raw`ns_make() { count=$1; i=0; while test $i -lt $count; do unshare --mount sh -c "j=0; while test \$j -lt 20; do mkdir -p /mnt/t\$j; mount -t tmpfs t /mnt/t\$j; j=\$((j+1)); done; exec sleep 900" > /dev/null 2>&1 & i=$((i+1)); done; sleep 2; }`,
      'echo "SCALE namespaces=$(for p in /proc/[0-9]*/ns/mnt; do readlink $p; done 2>/dev/null | sort -u | wc -l)"',
      'ns_make 100',
      'for round in 1 2 3 4 5; do',
      '  p0=$(now_us); $H start-probe > /root/timing-scale.log 2>&1 || true; p1=$(now_us); pv=$(grep "^verdict " $REC | cut -d" " -f2)',
      '  c0=$(now_us); CLI_GIT_TEST_ONLY_START_PROBE=disabled $H start-probe > /root/timing-control.log 2>&1 || true; c1=$(now_us); cv=$(grep "^verdict " $REC | cut -d" " -f2)',
      '  echo "MANY-NS namespaces=$(for p in /proc/[0-9]*/ns/mnt; do readlink $p; done 2>/dev/null | sort -u | wc -l) round=$round probe=$((p1-p0)) control=$((c1-c0)) worker-estimate=$(( (p1-p0) - (c1-c0) )) probe-verdict=$pv control-verdict=$cv"',
      'done',
      'ns_make 200',
      'for round in 1 2 3 4 5; do',
      '  p0=$(now_us); $H start-probe > /root/timing-scale.log 2>&1 || true; p1=$(now_us); pv=$(grep "^verdict " $REC | cut -d" " -f2)',
      '  c0=$(now_us); CLI_GIT_TEST_ONLY_START_PROBE=disabled $H start-probe > /root/timing-control.log 2>&1 || true; c1=$(now_us); cv=$(grep "^verdict " $REC | cut -d" " -f2)',
      '  echo "MANY-NS namespaces=$(for p in /proc/[0-9]*/ns/mnt; do readlink $p; done 2>/dev/null | sort -u | wc -l) round=$round probe=$((p1-p0)) control=$((c1-c0)) worker-estimate=$(( (p1-p0) - (c1-c0) )) probe-verdict=$pv control-verdict=$cv"',
      'done',
      'pkill -x sleep || true',
      'echo "TIMING done"',
    ],
  },);
  await guest({
    chosen,
    script,
    log: timingsLog,
    timeoutMilliseconds: longArmTimeoutMilliseconds,
  },);
}

/**
 Reboot the guest, then keep the first probe run after that boot and five warm
 rounds after it.

 @param chosen - the run's guest and log folder

 @throws Error naming the guest that did not answer, or its status

 @example
 ```ts
 await timingsBootFirst({ chosen });
 ```
 */
export async function timingsBootFirst({
  chosen,
}: {
  readonly chosen: GuestFacts;
},): Promise<void> {
  await statusOf({
    command: 'mvm',
    args: [
      ...backend.mvmArgumentWords,
      'exec',
      chosen.name,
      '--',
      'systemctl reboot',
    ],
    timeoutMilliseconds: rebootTimeoutMilliseconds,
  },);
  /**
   Whether the guest answered before the polls ran out.
   */
  const ready = await waitForTheGuestToAnswer({
    chosen,
    attemptsLeft: rebootPolls,
  },);
  if (!ready)
    throw new Error(`the guest ${chosen.name} did not answer after its reboot`);
  /**
   The arm: the modules loaded again, the pool imported, one boot-first run and
   five warm rounds after it.
   */
  const script = probeScript({
    lines: [
      'echo "BOOT-FIRST start $(date --iso-8601=seconds)"',
      'systemctl stop zfs-zed.service zfs-mount.service zfs-load-module.service 2>/dev/null || true',
      'if test -d /sys/module/zfs; then modprobe --remove zfs || true; fi',
      'if test -d /sys/module/spl; then modprobe --remove spl || true; fi',
      'insmod /lib/modules/$(uname --kernel-release)/extra/spl.ko',
      'insmod /lib/modules/$(uname --kernel-release)/extra/zfs.ko',
      `zpool import -d /home -o cachefile=none ${pool}`,
      'p0=$(now_us); $H start-probe > /root/boot-first.log 2>&1 || true; p1=$(now_us)',
      'echo "BOOT-FIRST probe=$((p1-p0)) verdict=$(grep "^verdict " $REC | cut -d" " -f2)"',
      'for round in 1 2 3 4 5; do',
      '  p0=$(now_us); $H start-probe > /root/timing-probe.log 2>&1 || true; p1=$(now_us); pv=$(grep "^verdict " $REC | cut -d" " -f2)',
      '  c0=$(now_us); CLI_GIT_TEST_ONLY_START_PROBE=disabled $H start-probe > /root/timing-control.log 2>&1 || true; c1=$(now_us); cv=$(grep "^verdict " $REC | cut -d" " -f2)',
      '  echo "AFTER-BOOT round=$round probe=$((p1-p0)) control=$((c1-c0)) worker-estimate=$(( (p1-p0) - (c1-c0) )) probe-verdict=$pv control-verdict=$cv"',
      'done',
      'echo "BOOT-FIRST done"',
    ],
  },);
  await guest({
    chosen,
    script,
    log: bootFirstLog,
    timeoutMilliseconds: armTimeoutMilliseconds,
  },);
}

/**
 Ask one rebooted guest whether it is back, and ask again until it answers or the
 questions run out.

 @param chosen - the run's guest and log folder

 @param attemptsLeft - how many more times the host may ask

 @returns whether the guest answered

 @example
 ```ts
 await waitForTheGuestToAnswer({ chosen, attemptsLeft: 180 });
 ```
 */
async function waitForTheGuestToAnswer({
  chosen,
  attemptsLeft,
}: {
  readonly chosen: GuestFacts;
  readonly attemptsLeft: number;
},): Promise<boolean> {
  if (attemptsLeft <= 0) return false;
  /**
   What the guest answered, which is zero only when it is back.
   */
  const status = await statusOf({
    command: 'mvm',
    args: [
      ...backend.mvmArgumentWords,
      'exec',
      chosen.name,
      '--',
      'true',
    ],
    timeoutMilliseconds: rebootPollTimeoutMilliseconds,
  },);
  if (status === 0) return true;
  await wait(rebootPollPauseMilliseconds,);
  return waitForTheGuestToAnswer({
    chosen,
    attemptsLeft: attemptsLeft - 1,
  },);
}
