/**
 Run the root modes' adoption control inside the guest: the adoption the root
 mode performs, the store folder publishing a record as root on the dataset it
 lives on and reading it back, and a writer killed between its bytes and its link,
 whose staging entry the adoption removes.

 The killed process is the arm's own child re-running the publication's primitives
 and killing itself, not the shipped publisher stopped at a window hook, which is
 a state a kill leaves rather than a code path.

 @example
 ```ts
 const pulled = await runTheAdoption({ chosen });
 ```
 */

import { guestBackend, } from '../../bin/mvm-guest-backend.ts';
import {
  holdsLineStartingWith,
  requireMarkers,
} from '../../bin/mvm-guest-arm-log.ts';
import {
  shellQuote,
} from '../../bin/mvm-guest-command-line.ts';
import type {
  GuestFacts,
} from '../../bin/mvm-guest-guest-facts.ts';


import {
  adoptionGuestVariable,
  armTimeoutMilliseconds,
  crateDirectory,
  expectedMarkers,
  finishMarginMilliseconds,
} from './adoption-fixture.ts';
import { pullGuestLog, } from '../../bin/mvm-guest-detached-script.ts';
import { detachedStep, } from './detached-step.ts';
/**
 The backend this run drives, read from the one place every guest step reads it from.
 */
const backend = guestBackend();

/**
 The log the adoption arm writes inside the guest's shared folder.
 */
const armLog = 'adoption-arm.log';

/**
 The log the arm's step writes inside the guest's shared folder.
 */
const armStepLog = 'guest-adoption.log';

/**
 Run the adoption control and refuse a log that only returned.

 @param chosen - the run's guest and log folder

 @returns the arm's log, as the guest wrote it

 @throws Error naming the marker the pulled log lacks, or the log that skipped

 @example
 ```ts
 const pulled = await runTheAdoption({ chosen });
 ```
 */
export async function runTheAdoption({
  chosen,
}: {
  readonly chosen: GuestFacts;
},): Promise<string> {
  /**
   The arm itself: the built test target's adoption and store controls, run
   where the store's `/var/lib` path is a dataset of the guest's own pool.
   */
  const script = [
    'set -eu',
    `cd ${crateDirectory}/package/git-policy/cli`,
    'export PATH=/root/.cargo/bin:$PATH',
    `export ${adoptionGuestVariable}=${backend.hostnameOf(chosen.name)}`,
    'if cargo nextest run --release --locked --features test-only-switches --test native_binary --no-capture --status-level all '
      + `--filterset "test(zfs_adoption_guest)" > /root/${armLog} 2>&1; then status=0; else status=$?; fi`,
    'tail --lines=40 /root/adoption-arm.log',
    'test "$status" = 0',
    // A returning arm is not a passing arm, which the recipe for the binding's live suites also holds.
    `if grep --quiet '^SKIPPED' /root/${armLog}; then echo 'the adoption arm returned without running'; exit 1; fi`,
    ...expectedMarkers.map(function markerLine(marker,) {
      return `grep --quiet --fixed-strings ${shellQuote(marker,)} /root/${armLog}`;
    },),
    `cp /root/${armLog} /mnt/shared/${armLog}`,
  ].join('\n',);
  await detachedStep({
    chosen,
    script,
    log: armStepLog,
    timeoutMilliseconds: armTimeoutMilliseconds,
    finishTimeoutMilliseconds: armTimeoutMilliseconds + finishMarginMilliseconds,
  },);
  /**
   The arm's own log, which the step copied into the guest's shared folder and
   which this pull keeps as the run's evidence.
   */
  const pulled = await pullGuestLog({
    chosen,
    file: armLog,
    log: 'host-pull-adoption.log',
  },);
  requireMarkers({
    text: pulled,
    markers: expectedMarkers,
    subject: 'the pulled arm log',
    because: 'the guest ran the arm without performing it',
  },);
  if (holdsLineStartingWith({
    text: pulled,
    word: 'SKIPPED',
  },))
    throw new Error('the pulled arm log holds a SKIPPED line, so the adoption arm did not run');
  return pulled;
}
