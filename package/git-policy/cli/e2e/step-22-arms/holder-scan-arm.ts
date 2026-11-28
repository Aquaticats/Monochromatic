/**
 Run plan step 9's holder-scan arm in the same guest, which times the scan the
 unmount request path costs.

 Why: the unmount deadline must cover the all-namespace holder scan as well as
 the unmount, and the scan's own arm is the measurement of that share.

 @example
 ```ts
 const pulled = await measureTheHolderScan({ chosen });
 ```
 */

import { guestBackend, } from '../../bin/mvm-guest-backend.ts';
import type {
  GuestFacts,
} from '../../bin/mvm-guest-guest-facts.ts';

import { detachedStep, } from './detached-step.ts';
import { pullSharedLog, } from './host-shared-log.ts';
import {
  armTimeoutMilliseconds,
  crateDirectory,
  finishMarginMilliseconds,
  holderScanLog,
  holderScanMeasurement,
  openZfsPath,
  scanCheckout,
  scanPool,
} from './measurement-fixture.ts';
/**
 The backend this run drives, read from the one place every guest step reads it from.
 */
const backend = guestBackend();

/**
 The log the holder-scan step writes inside the guest's shared folder.
 */
const holderScanStepLog = 'guest-holder-scan.log';

/**
 The pool-variable name the arm reads its pool from.
 */
const scanPoolVariable = 'CLI_GIT_ZFS_SCAN_POOL';

/**
 Run the holder scan against a layout of its own, and read its timing line out of
 the pulled log.

 @param chosen - the run's guest and log folder

 @returns the arm's log, as the guest wrote it

 @throws Error naming the marker the pulled log lacks

 @example
 ```ts
 const pulled = await measureTheHolderScan({ chosen });
 ```
 */
export async function measureTheHolderScan({
  chosen,
}: {
  readonly chosen: GuestFacts;
},): Promise<string> {
  /**
   The arm: the built test target's holder-scan measurements, run against a
   checkout the arm makes for itself.
   */
  const script = [
    'set -eu',
    `export PATH=${openZfsPath}`,
    `mkdir --parents ${scanCheckout}`,
    `cd ${crateDirectory}/package/git-policy/cli`,
    'export PATH=/root/.cargo/bin:$PATH',
    `export CLI_GIT_ZFS_SCAN_GUEST=${backend.hostnameOf(chosen.name)}`,
    `export ${scanPoolVariable}=${scanPool}`,
    'export CLI_GIT_ZFS_SCAN_SBIN=/opt/openzfs-2.4.4/sbin',
    `export CLI_GIT_ZFS_SCAN_CHECKOUT=${scanCheckout}`,
    'if cargo nextest run --release --locked --test native_binary --no-capture --status-level all '
      + `--filterset "test(zfs_holder_scan_guest)" > /root/${holderScanLog} 2>&1; then status=0; else status=$?; fi`,
    `cp /root/${holderScanLog} /mnt/shared/${holderScanLog} || true`,
    `tail --lines=40 /root/${holderScanLog}`,
    'test "$status" = 0',
  ].join('\n',);
  await detachedStep({
    chosen,
    script,
    log: holderScanStepLog,
    timeoutMilliseconds: armTimeoutMilliseconds,
    finishTimeoutMilliseconds: armTimeoutMilliseconds + finishMarginMilliseconds,
  },);
  /**
   The arm's own log, which its script copied into the guest's shared folder and
   which this driver reads from the host side, keeping it as the run's evidence.
   */
  const pulled = await pullSharedLog({
    name: chosen.name,
    file: holderScanLog,
    logDir: chosen.logDir,
  },);
  if (!pulled.includes(holderScanMeasurement))
    throw new Error('the pulled holder-scan log lacks its timing line; the arm ran without timing the scan');
  return pulled;
}
