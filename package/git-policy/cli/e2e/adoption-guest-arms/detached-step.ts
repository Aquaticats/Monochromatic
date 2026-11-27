/**
 One detached guest script, whose end the host reads through the guest agent: start it, wait for the status line the
 guest appends, pull its log with `mvm`'s own copy, and refuse a status that is not zero.

 The adoption guest's build and its arms run for minutes, and the guest agent has lost the status of a long `exec`, so
 the work runs detached and says how it ended in its own log, which the host reads by asking the guest for that log's
 last line.

 This driver's launcher carries the guest-name check, so a run that reached another machine refuses before the step file
 is started at all: that is why `nameCheck` is `launcher` here and `step` in the step 22 driver.

 @example
 ```ts
 const pulled = await detachedStep({ chosen, script: 'id', log: 'guest-id.log', timeoutMilliseconds: 900_000, finishTimeoutMilliseconds: 1_020_000 });
 ```
 */

import {
  awaitDetachedGuestScript,
  startDetachedGuestScript,
} from '../../bin/mvm-guest-detached-script.ts';
import type { GuestFacts, } from '../../bin/mvm-guest-guest-facts.ts';

/**
 Start one detached guest script, wait for its own end, and pull its log.

 @param chosen - the run's guest and log folder

 @param script - the shell script to run inside the guest

 @param log - the log's name inside the guest's shared folder

 @param timeoutMilliseconds - milliseconds the script may take inside the guest

 @param finishTimeoutMilliseconds - milliseconds the host waits for the script to end

 @returns the pulled log, written into the run's log folder

 @throws Error naming the status the script ended with, or the wait that ended

 @example
 ```ts
 const pulled = await detachedStep({ chosen, script: 'id', log: 'guest-id.log', timeoutMilliseconds: 900_000, finishTimeoutMilliseconds: 1_020_000 });
 ```
 */
export async function detachedStep({
  chosen,
  script,
  log,
  timeoutMilliseconds,
  finishTimeoutMilliseconds,
}: {
  readonly chosen: GuestFacts;
  readonly script: string;
  readonly log: string;
  readonly timeoutMilliseconds: number;
  readonly finishTimeoutMilliseconds: number;
},): Promise<string> {
  await startDetachedGuestScript({
    chosen,
    script,
    log,
    timeoutMilliseconds,
    nameCheck: 'launcher',
  },);
  return await awaitDetachedGuestScript({
    chosen,
    log,
    finishTimeoutMilliseconds,
  },);
}
