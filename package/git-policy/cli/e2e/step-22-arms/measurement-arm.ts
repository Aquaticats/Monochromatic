/**
 Lay the two file-backed pools step 22's arms measure in, and run the measurement
 arm itself: the add against ordinary Git under the same workload with an
 interleaved control, the mount request's wait, the unmount's, the sweep's cost
 and the lost-event behaviour, what expiration and removal free, the forced arm's
 three waits, and the request budget.

 The guest's own `git-policy-cli-store` pool belongs to the holder-scan arm beside
 this one, and the two arms never read each other's datasets, so a failure of one
 is not a number the other reports.

 @example
 ```ts
 const pulled = await measureStep22({ chosen });
 ```
 */

import { guestBackend, } from '../../bin/mvm-guest-backend.ts';
import {
  holdsLineStartingWith,
  requireMarkers,
} from '../../bin/mvm-guest-arm-log.ts';
import type {
  GuestFacts,
} from '../../bin/mvm-guest-guest-facts.ts';


import { detachedStep, } from './detached-step.ts';
import { pullSharedLog, } from './host-shared-log.ts';
import {
  armLog,
  armTimeoutMilliseconds,
  checkout,
  crateDirectory,
  expectedMarkers,
  fixtureRoot,
  finishMarginMilliseconds,
  guestVariable,
  longFinishMarginMilliseconds,
  openZfsPath,
  openZfsPathWithCargo,
  openZfsSbin,
  pool,
  poolTimeoutMilliseconds,
  scanPool,
} from './measurement-fixture.ts';
/**
 The backend this run drives, read from the one place every guest step reads it from.
 */
const backend = guestBackend();

/**
 The log the pool script writes inside the guest's shared folder.
 */
const poolLog = 'guest-pool.log';

/**
 The log the arm writes inside the guest's shared folder.
 */
const armStepLog = 'guest-arm.log';

/**
 Lay the file-backed pools the arm measures in, which live only inside this
 throwaway guest.

 @param chosen - the run's guest and log folder

 @throws Error naming the status the script ended with

 @example
 ```ts
 await layThePools({ chosen });
 ```
 */
export async function layThePools({
  chosen,
}: {
  readonly chosen: GuestFacts;
},): Promise<void> {
  /**
   The two pools, each created only when the guest does not already hold it, so
   a second run of the arm reuses the first one's datasets.
   */
  const script = [
    'set -eu',
    `export PATH=${openZfsPath}`,
    `mkdir --parents ${fixtureRoot}`,
    `mkdir --parents ${checkout}`,
    `if ! zpool list -H -o name ${pool} > /dev/null 2>&1; then truncate --size 2G ${fixtureRoot}/pool.img; zpool create -f -O mountpoint=none ${pool} ${fixtureRoot}/pool.img; fi`,
    // The holder-scan arm beside this one builds its own layout under a pool of
    // its own, so the two arms never read each other's datasets.
    `if ! zpool list -H -o name ${scanPool} > /dev/null 2>&1; then truncate --size 512M ${fixtureRoot}/scan-pool.img; zpool create -f -O mountpoint=none ${scanPool} ${fixtureRoot}/scan-pool.img; fi`,
    `zpool list -H -o name,size ${pool}`,
    `zpool list -H -o name,size ${scanPool}`,
  ].join('\n',);
  await detachedStep({
    chosen,
    script,
    log: poolLog,
    timeoutMilliseconds: poolTimeoutMilliseconds,
    finishTimeoutMilliseconds: poolTimeoutMilliseconds + finishMarginMilliseconds,
  },);
}

/**
 Run step 22's measurement arm in the guest and read the lines it performed out of
 the pulled log.

 @param chosen - the run's guest and log folder

 @returns the arm's log, as the guest wrote it

 @throws Error naming the marker the pulled log lacks, or the log that skipped

 @example
 ```ts
 const pulled = await measureStep22({ chosen });
 ```
 */
export async function measureStep22({
  chosen,
}: {
  readonly chosen: GuestFacts;
},): Promise<string> {
  /**
   The arm itself: the built test target's step 22 measurements, run where the
   guest's own pools are and never on a machine somebody uses.
   */
  const script = [
    'set -eu',
    `export PATH=${openZfsPathWithCargo}`,
    `export ${guestVariable}=${backend.hostnameOf(chosen.name)}`,
    `export CLI_GIT_ZFS_STEP22_POOL=${pool}`,
    `export CLI_GIT_ZFS_STEP22_SBIN=${openZfsSbin}`,
    `export CLI_GIT_ZFS_STEP22_ROOT=${checkout}`,
    `export CLI_GIT_ZFS_STEP22_WORKLOAD=${crateDirectory}/package/git-policy/cli/src`,
    `cd ${crateDirectory}/package/git-policy/cli`,
    'if cargo nextest run --release --locked --test native_binary --no-capture --status-level all '
      + `--filterset "test(the_step_22_guest_measurements)" > /root/${armLog} 2>&1; then status=0; else status=$?; fi`,
    `cp /root/${armLog} /mnt/shared/${armLog} || true`,
    `tail --lines=120 /root/${armLog}`,
    'test "$status" = 0',
  ].join('\n',);
  await detachedStep({
    chosen,
    script,
    log: armStepLog,
    timeoutMilliseconds: armTimeoutMilliseconds,
    finishTimeoutMilliseconds: armTimeoutMilliseconds + longFinishMarginMilliseconds,
  },);
  /**
   The arm's own log, which its script copied into the guest's shared folder and
   which this driver reads from the host side, keeping it as the run's evidence.
   */
  const pulled = await pullSharedLog({
    name: chosen.name,
    file: armLog,
    logDir: chosen.logDir,
  },);
  requireMarkers({
    text: pulled,
    markers: expectedMarkers,
    subject: 'the pulled arm log',
    because: 'the guest ran the arm without performing it',
  },);
  if (holdsLineStartingWith({
    text: pulled,
    word: 'SKIP',
  },))
    throw new Error('the pulled arm log holds a SKIP line, so the arm did not run');
  return pulled;
}
