/**
 Drive plan step 9's staging mechanics in the same guest: the record before the
 bind, the private bind, a mount moved onto it, the later sweep from the record's
 handle, a record from another boot, and the cleanup's allowed and rejected
 fixtures.

 The arm makes a tmpfs stand in for a clone, because the design's ZFS layout arms
 and M1 to M9 remain owed; it runs the built module's calls, the record and the
 sweep, and it prints one marker per arm.

 @example
 ```ts
 const pulled = await unmountArms({ chosen });
 ```
 */

import { join, } from 'node:path';

import type { GuestRun, } from '../guest-host/guest-run.ts';
import { host, } from '../guest-host/host-side-commands.ts';
import {
  readLog,
  requireMarkers,
  shellQuote,
} from '../guest-host/guest-host-rules-fixture.ts';
import {
  armTimeoutMilliseconds,
  crateDirectory,
  expectedUnmountMarkers,
  finishMarginMilliseconds,
  storePool,
  sweepCheckout,
  unmountGuestVariable,
} from './adoption-fixture.ts';
import { detachedStep, } from './detached-step.ts';

/**
 The log the unmount arm writes inside the guest's shared folder.
 */
const armLog = 'unmount-arm.log';

/**
 The log the arm's step writes inside the guest's shared folder.
 */
const armStepLog = 'guest-unmount.log';

/**
 Milliseconds one unmount arm may take, which its staging arms and their kills
 need twice over.
 */
const unmountArmTimeoutMilliseconds = armTimeoutMilliseconds * 2;

/**
 Run the unmount arms and refuse a log that only returned.

 @param chosen - the run's guest and log folder

 @returns the arm's log, as the guest wrote it

 @throws Error naming the marker the pulled log lacks

 @example
 ```ts
 const pulled = await unmountArms({ chosen });
 ```
 */
export async function unmountArms({
  chosen,
}: {
  readonly chosen: GuestRun;
},): Promise<string> {
  /**
   The arm itself, which stages and kills its own mounts inside the guest.
   */
  const script = [
    'set -eu',
    `cd ${crateDirectory}/package/git-policy/cli`,
    'export PATH=/root/.cargo/bin:$PATH',
    `export ${unmountGuestVariable}=${chosen.name}`,
    `export CLI_GIT_ZFS_SWEEP_POOL=${storePool}`,
    'export CLI_GIT_ZFS_SWEEP_SBIN=/opt/openzfs-2.4.4/sbin',
    `export CLI_GIT_ZFS_SWEEP_CHECKOUT=${sweepCheckout}`,
    'if cargo nextest run --release --locked --features test-only-switches --test native_binary --no-capture --status-level all '
      + `--filterset "test(zfs_unmount_guest)" > /root/${armLog} 2>&1; then status=0; else status=$?; fi`,
    'tail --lines=60 /root/unmount-arm.log',
    'test "$status" = 0',
    `if grep --quiet '^SKIPPED' /root/${armLog}; then echo 'the unmount arm returned without running'; exit 1; fi`,
    ...expectedUnmountMarkers.map(function markerLine(marker,) {
      return `grep --quiet --fixed-strings ${shellQuote(marker,)} /root/${armLog}`;
    },),
    `cp /root/${armLog} /mnt/shared/${armLog}`,
  ].join('\n',);
  await detachedStep({
    chosen,
    script,
    log: armStepLog,
    timeout: unmountArmTimeoutMilliseconds,
    finishTimeout: unmountArmTimeoutMilliseconds + finishMarginMilliseconds,
  },);
  /**
   The host's copy of the arm's log, which the pull below writes.
   */
  const pulledPath = join(
    chosen.logDir,
    armLog,
  );
  await host({
    chosen,
    command: 'mvm',
    args: [
      '--backend',
      'libvirt',
      'pull',
      chosen.name,
      `/mnt/shared/${armLog}`,
      pulledPath,
    ],
    log: 'host-pull-unmount.log',
  },);
  /**
   The arm's answers, as `mvm` copied them out.
   */
  const pulled = await readLog({ path: pulledPath, },);
  requireMarkers({
    pulled,
    markers: expectedUnmountMarkers,
    subject: 'the pulled unmount-arm log',
    because: 'the guest ran the arm without performing it',
  },);
  return pulled;
}
