/**
 Drive plan step 9's holder-scan arms in the same guest: the built handler's scan
 against the design's layout.

 The arms build their own checkout, clone parent and clone, attach the clone with
 the built sweep, and then hold it from a second mount namespace, a namespace made
 after the attach (under both intents), a confined user namespace, a thread with a
 mount namespace of its own, a task whose leader has ended, a namespace whose
 lowest task is chrooted, and a namespace pinned by a handle while no task is left
 in it; one arm detaches a mount while a working directory pins it. Task churn
 runs inside the guest only, and one scan is timed with a small and a large number
 of live tasks. Every one of those reads `/proc` or mounts a dataset, so no arm may
 run on the host.

 @example
 ```ts
 const pulled = await holderScanArms({ chosen });
 ```
 */

import { join, } from 'node:path';

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
  host,
} from '../../bin/mvm-guest-host-command.ts';
import {
  readLog,
} from '../../bin/mvm-guest-step-command.ts';



import {
  armTimeoutMilliseconds,
  crateDirectory,
  expectedScanMarkers,
  expectedScanMeasurements,
  finishMarginMilliseconds,
  openZfsPath,
  scanCheckout,
  scanGuestVariable,
  storePool,
} from './adoption-fixture.ts';
import { detachedStep, } from './detached-step.ts';

/**
 The log the holder-scan arm writes inside the guest's shared folder.
 */
const armLog = 'holder-scan-arm.log';

/**
 The log the arm's step writes inside the guest's shared folder.
 */
const armStepLog = 'guest-holder-scan.log';

/**
 How many arm bounds one holder-scan step gets: its churn arm runs twenty scans,
 and each scan reads every namespace on its own.
 */
const scanArmBounds = 3;

/**
 Milliseconds one holder-scan step may take.
 */
const scanArmTimeoutMilliseconds = armTimeoutMilliseconds * scanArmBounds;

/**
 Run the holder-scan arms and refuse a log that only returned.

 @param chosen - the run's guest and log folder

 @returns the arm's log, as the guest wrote it

 @throws Error naming the marker the pulled log lacks

 @example
 ```ts
 const pulled = await holderScanArms({ chosen });
 ```
 */
export async function holderScanArms({
  chosen,
}: {
  readonly chosen: GuestFacts;
},): Promise<string> {
  /**
   The arm itself, against a checkout and a clone parent the arm makes for
   itself.
   */
  const script = [
    'set -eu',
    `export PATH=${openZfsPath}`,
    `rm --recursive --force ${scanCheckout}`,
    `mkdir --parents ${scanCheckout}`,
    `cd ${crateDirectory}/package/git-policy/cli`,
    'export PATH=/root/.cargo/bin:$PATH',
    `export ${scanGuestVariable}=${chosen.name}`,
    `export CLI_GIT_ZFS_SCAN_POOL=${storePool}`,
    'export CLI_GIT_ZFS_SCAN_SBIN=/opt/openzfs-2.4.4/sbin',
    `export CLI_GIT_ZFS_SCAN_CHECKOUT=${scanCheckout}`,
    'if cargo nextest run --release --locked --features test-only-switches --test native_binary --no-capture --status-level all '
      + `--filterset "test(zfs_holder_scan_guest)" > /root/${armLog} 2>&1; then status=0; else status=$?; fi`,
    'tail --lines=80 /root/holder-scan-arm.log',
    'test "$status" = 0',
    `if grep --quiet '^SKIPPED' /root/${armLog}; then echo 'the holder-scan arm returned without running'; exit 1; fi`,
    ...expectedScanMarkers.map(function markerLine(marker,) {
      return `grep --quiet --fixed-strings ${shellQuote(marker,)} /root/${armLog}`;
    },),
    ...expectedScanMeasurements.map(function measurementLine(marker,) {
      return `grep --quiet --fixed-strings ${shellQuote(marker,)} /root/${armLog}`;
    },),
    `cp /root/${armLog} /mnt/shared/${armLog}`,
  ].join('\n',);
  await detachedStep({
    chosen,
    script,
    log: armStepLog,
    timeoutMilliseconds: scanArmTimeoutMilliseconds,
    finishTimeoutMilliseconds: scanArmTimeoutMilliseconds + finishMarginMilliseconds,
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
    log: 'host-pull-holder-scan.log',
  },);
  /**
   The arm's answers, as `mvm` copied them out.
   */
  const pulled = await readLog({ path: pulledPath, },);
  requireMarkers({
    text: pulled,
    markers: expectedScanMarkers,
    subject: 'the pulled holder-scan log',
    because: 'the guest ran the arm without performing it',
  },);
  return pulled;
}
