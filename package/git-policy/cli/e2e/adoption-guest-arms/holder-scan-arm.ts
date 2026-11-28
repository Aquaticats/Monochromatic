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

 What the guest runs is `holder-scan-arm-script-fixture.ts`, which a unit test holds
 without a guest; this file starts that script and judges the log it leaves.

 @example
 ```ts
 const pulled = await holderScanArms({ chosen });
 ```
 */

import { join, } from 'node:path';

import { guestBackend, } from '../../bin/mvm-guest-backend.ts';
import {
  requireMarkers,
} from '../../bin/mvm-guest-arm-log.ts';
import type {
  GuestFacts,
} from '../../bin/mvm-guest-guest-facts.ts';
import {
  host,
} from '../../bin/mvm-guest-host-command.ts';
import {
  readLog,
} from '../../bin/mvm-guest-step-command.ts';

import {
  armTimeoutMilliseconds,
  expectedScanMarkers,
  finishMarginMilliseconds,
} from './adoption-fixture.ts';
import { detachedStep, } from './detached-step.ts';
import { holderScanScript, } from './holder-scan-arm-script-fixture.ts';

/**
 The backend this run drives, read from the one place every guest step reads it from.
 */
const backend = guestBackend();


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
  const script = holderScanScript({
    chosen,
    log: armLog,
  },);
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
      ...backend.mvmArgumentWords,
      'pull',
      chosen.name,
      `${backend.guestSharedFolder}/${ armLog }`,
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
