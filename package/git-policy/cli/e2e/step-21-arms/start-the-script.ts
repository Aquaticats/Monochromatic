/**
 Start one detached guest script, trying again when the shared helper's own bound on the push expires.

 The helpers in `bin/mvm-guest-detached-script.ts` push the script and its launcher into the guest's shared folder before
 they start anything, and each push carries `HOST_COMMAND_TIMEOUT_MS`. On a host that is out of memory a 700-byte push
 answered in 0.12 s and the same push was still running after 120 s, so a run that gave up at the first expiry would
 report the machine's state as a defect of the arms. Nothing else changes: the script, its bound and its log are the
 shared helper's own.

 @example
 ```ts
 await startTheScript({ chosen, script: 'id', log: 'guest-id.log', timeoutMilliseconds: 900_000 });
 ```
 */

import {
  caughtValueText,
} from '@monochromatic-dev/caught-value-core/ts';

import {
  startDetachedGuestScript,
} from '../../bin/mvm-guest-detached-script.ts';
import type {
  GuestFacts,
} from '../../bin/mvm-guest-guest-facts.ts';

/**
 How many times one script's start is tried, and why each retry is a fresh push rather than a longer wait.
 */
const STARTS = 3;

/**
 Start one detached script, trying again when the shared helper's push expired.

 @param chosen - the run's guest and log folder

 @param script - the shell script to run inside the guest

 @param log - the log's name inside the guest's shared folder

 @param timeoutMilliseconds - milliseconds the script may take inside the guest

 @throws GuestDriverError naming the last refusal, when every start expired

 @example
 ```ts
 await startTheScript({ chosen, script: 'id', log: 'guest-id.log', timeoutMilliseconds: 900_000 });
 ```
 */
export async function startTheScript({
  chosen,
  script,
  log,
  timeoutMilliseconds,
}: {
  readonly chosen: GuestFacts;
  readonly script: string;
  readonly log: string;
  readonly timeoutMilliseconds: number;
},): Promise<void> {
  await startAnAttempt({
    chosen,
    script,
    log,
    timeoutMilliseconds,
    attemptsLeft: STARTS,
  },);
}

/**
 One attempt to start the script, and the next one when this attempt's push expired.

 @param chosen - the run's guest and log folder

 @param script - the shell script to run inside the guest

 @param log - the log's name inside the guest's shared folder

 @param timeoutMilliseconds - milliseconds the script may take inside the guest

 @param attemptsLeft - how many starts are still allowed, this one included

 @throws Error naming the last refusal, when no attempt is left

 @example
 ```ts
 await startAnAttempt({ chosen, script: 'id', log: 'guest-id.log', timeoutMilliseconds: 900_000, attemptsLeft: 3 });
 ```
 */
async function startAnAttempt({
  chosen,
  script,
  log,
  timeoutMilliseconds,
  attemptsLeft,
}: {
  readonly chosen: GuestFacts;
  readonly script: string;
  readonly log: string;
  readonly timeoutMilliseconds: number;
  readonly attemptsLeft: number;
},): Promise<void> {
  try {
    await startDetachedGuestScript({
      chosen,
      script,
      log,
      timeoutMilliseconds,
      nameCheck: 'step',
    },);
  }
  catch (caught) {
    /**
     What this attempt refused with, which the last attempt's refusal is reported with.
     */
    const refusal = caughtValueText(caught,);
    if (attemptsLeft <= 1) {
      throw new Error(
        `the script ${log} was never started: ${refusal}`,
        { cause: caught, },
      );
    }
    await startAnAttempt({
      chosen,
      script,
      log,
      timeoutMilliseconds,
      attemptsLeft: attemptsLeft - 1,
    },);
  }
}
