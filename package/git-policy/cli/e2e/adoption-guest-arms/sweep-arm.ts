/**
 Drive plan step 8's arms in the same guest: the built handler's own sweep against
 the guest's pool.

 The arms make their own fixtures as root inside the guest, because the sweep
 attaches a real clone and writes `/var/lib` and `/run`: one unencrypted clone
 parent with a clone that asks for its mount, one encrypted parent whose key is
 unloaded and then loaded, and the handler's refusals.

 @example
 ```ts
 const pulled = await sweepArms({ chosen });
 ```
 */

import {
  requireMarkers,
} from '../../bin/mvm-guest-arm-log.ts';
import {
  shellQuote,
} from '../../bin/mvm-guest-command-line.ts';
import {
  type GuestFacts,
} from '../../bin/mvm-guest-guest-facts.ts';


import {
  armTimeoutMilliseconds,
  crateDirectory,
  expectedSweepMarkers,
  expectedSweepMeasurements,
  openZfsPath,
  storePool,
  sweepCheckout,
  sweepGuestVariable,
  sweepStaleCheckout,
} from './adoption-fixture.ts';
import { pullGuestLog, } from '../../bin/mvm-guest-detached-script.ts';
import { detachedStep, } from './detached-step.ts';

/**
 The log the sweep arm writes inside the guest's shared folder.
 */
const armLog = 'sweep-arm.log';

/**
 The log the arm's step writes inside the guest's shared folder.
 */
const armStepLog = 'guest-sweep.log';

/**
 Milliseconds the host gives the sweep arm past its own bound, which is the pull
 of a log this long.
 */
const sweepFinishMarginMilliseconds = 120_000;

/**
 Run the sweep arms and refuse a log that only returned.

 @param chosen - the run's guest and log folder

 @returns the arm's log, as the guest wrote it

 @throws Error naming the marker the pulled log lacks

 @example
 ```ts
 const pulled = await sweepArms({ chosen });
 ```
 */
export async function sweepArms({
  chosen,
}: {
  readonly chosen: GuestFacts;
},): Promise<string> {
  /**
   The arm itself: a layout an earlier run left mounted is taken down first, so
   the arm runs again on the same guest.
   */
  const script = [
    'set -eu',
    `export PATH=${openZfsPath}`,
    // A layout an earlier run left mounted is taken down first, so the arm runs again on the same guest.
    // Every mount under the checkout goes first, the slots an earlier run attached included, then the checkout itself;
    // the findmnt sweep is the one that also reaches a mount the rollback arm moved beside the checkout.
    `for slot in ${sweepCheckout}/.claude/worktrees/*; do umount --force "$slot" 2>/dev/null || true; done`,
    `for target in $(findmnt --list --noheadings --output TARGET | grep --fixed-strings ${sweepCheckout} || true); do umount "$target" 2>/dev/null || true; done`,
    `umount --force ${sweepCheckout} 2>/dev/null || true`,
    // The stale sub-case's own checkout carries the design's leaf name too, and a run that stopped inside it leaves its
    // thousand slot mounts standing. A legacy dataset whose clone a mount holds cannot be destroyed, so those mounts go
    // before the datasets, or the `zfs create` below meets a dataset that is still there.
    `for slot in ${sweepStaleCheckout}/.claude/worktrees/*; do umount --force "$slot" 2>/dev/null || true; done`,
    `umount --force ${sweepStaleCheckout} 2>/dev/null || true`,
    // The guest's OpenZFS 2.4.4 refuses `zfs destroy --recursive --force` with `invalid option '-'` (measured
    // 2026-11-25: `destroy` takes only `-f -n -p -R -r -v`), so every line below spelled its options long and never
    // destroyed anything, its refusal going to `/dev/null`. The short flags are the only form this tool has, and the
    // refusal now stays in the log rather than being discarded.
    `zfs destroy -r -f ${storePool}/step8 || true`,
    `zfs destroy -r -f ${storePool}/step8-bound || true`,
    `zfs destroy -r -f ${storePool}/step8-encrypted || true`,
    `zfs destroy -r -f ${storePool}/step8-foreign || true`,
    // The stale arm's own checkout carries the design's leaf name too, and the no-configuration shapes that run before
    // it need one instance of that name, so its layout goes before the arm and not only after it.
    `zfs destroy -r -f ${storePool}/step8-stale || true`,
    // The stoppage arm's container and the probe fixture this arm creates live in the same pool.
    `zfs destroy -r -f ${storePool}/account-clones || true`,
    `zfs destroy -r -f ${storePool}/nosnap || true`,
    // Another driver's checkout dataset carries the same leaf name and would be a second instance for the arm's
    // no-configuration shapes; it is taken down for the length of the arm and mounted again afterwards.
    'others=$(findmnt --list --noheadings --output SOURCE --type zfs | grep --fixed-strings monochromatic-git-checkout | grep --invert-match "git-policy-cli-store/step8" | sort --unique || true)',
    'for other in $others; do zfs unmount "$other" 2>/dev/null || true; done',
    `rm --recursive --force ${sweepCheckout}`,
    `mkdir --parents ${sweepCheckout}`,
    `cd ${crateDirectory}/package/git-policy/cli`,
    'export PATH=/root/.cargo/bin:$PATH',
    `export ${sweepGuestVariable}=${chosen.name}`,
    `export CLI_GIT_ZFS_SWEEP_POOL=${storePool}`,
    'export CLI_GIT_ZFS_SWEEP_SBIN=/opt/openzfs-2.4.4/sbin',
    `export CLI_GIT_ZFS_SWEEP_CHECKOUT=${sweepCheckout}`,
    'if cargo nextest run --release --locked --features test-only-switches --test native_binary --no-capture --status-level all '
      + `--filterset "test(zfs_sweep_guest)" > /root/${armLog} 2>&1; then status=0; else status=$?; fi`,
    'tail --lines=60 /root/sweep-arm.log',
    'test "$status" = 0',
    `if grep --quiet '^SKIPPED' /root/${armLog}; then echo 'the sweep arm returned without running'; exit 1; fi`,
    ...expectedSweepMarkers.map(function markerLine(marker,) {
      return `grep --quiet --fixed-strings ${shellQuote(marker,)} /root/${armLog}`;
    },),
    ...expectedSweepMeasurements.map(function measurementLine(marker,) {
      return `grep --quiet --fixed-strings ${shellQuote(marker,)} /root/${armLog}`;
    },),
    `cp /root/${armLog} /mnt/shared/${armLog}`,
    'for other in $others; do zfs mount "$other" 2>/dev/null || true; done',
    // The arm's own layout goes, so a driver that runs afterwards finds one instance of the checkout's leaf name and
    // not two: the probe's no-configuration arm reads the whole mount table.
    `for target in $(findmnt --list --noheadings --output TARGET | grep --fixed-strings ${sweepCheckout} || true); do umount "$target" 2>/dev/null || true; done`,
    `umount ${sweepCheckout} 2>/dev/null || true`,
    `for layout in step8 step8-stale step8-encrypted step8-foreign step8-bound account-clones nosnap; do zfs destroy -r -f ${storePool}/$layout 2>/dev/null || true; done`,
  ].join('\n',);
  await detachedStep({
    chosen,
    script,
    log: armStepLog,
    timeoutMilliseconds: armTimeoutMilliseconds * 2,
    finishTimeoutMilliseconds: (armTimeoutMilliseconds * 2) + sweepFinishMarginMilliseconds,
  },);
  /**
   The arm's own log, which the step copied into the guest's shared folder and
   which this pull keeps as the run's evidence.
   */
  const pulled = await pullGuestLog({
    chosen,
    file: armLog,
    log: 'host-pull-sweep.log',
  },);
  requireMarkers({
    text: pulled,
    markers: expectedSweepMarkers,
    subject: 'the pulled sweep-arm log',
    because: 'the guest ran the arm without performing it',
  },);
  return pulled;
}
