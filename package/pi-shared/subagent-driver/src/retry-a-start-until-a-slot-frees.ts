/**
 Waiting for a slot: asks a start again and again until it is no longer refused for want of a pool slot, and gives up at a
 deadline.

 A shorter period takes a freed slot sooner, which is how the queue's order is kept, so the period is the caller's. Every wait
 is bounded by the deadline, so no queue waits for ever.

 @module
 */

import { NO_SLOT_EXIT_STATUS, } from './pools-of-pi-runs.ts';

/**
 What one attempt at the start answered with: the status it exited with, and what it printed.
 */
export type StartAttempt = {
  /**
   The status the start exited with.
   */
  readonly status: number;

  /**
   What the start printed, standard error included, which the caller prints when the attempt ended otherwise.
   */
  readonly output: string;
};

/**
 What one queue answers with.
 */
export type RetryOutcome = {
  /**
   The status to exit with: the start's own, or {@link NO_SLOT_EXIT_STATUS} when the deadline passed.
   */
  readonly status: number;

  /**
   What to print.
   */
  readonly output: string;
};

/**
 Everything the queue needs.
 */
export type RetryAStartRequest = {
  /**
   One attempt at the start.
   */
  readonly startOnce: () => Promise<StartAttempt>;

  /**
   Where the queue reads the time, which a test replaces with its own clock.
   */
  readonly now: () => number;

  /**
   How the queue passes time, which a test replaces so it neither waits nor needs a clock.
   */
  readonly wait: (milliseconds: number) => Promise<void>;

  /**
   When the queue must be over, as milliseconds since the epoch.
   */
  readonly deadlineMilliseconds: number;

  /**
   How long to wait before asking again.
   */
  readonly periodMilliseconds: number;

  /**
   What the queue is waiting to start, for the line that says the deadline passed.
   */
  readonly describedAs: string;
};

/**
 Reads the last line a start printed, which is the line a queue reports.

 @param output - what the start printed

 @returns the last line that holds anything, or empty text when it printed nothing

 @example
 ```ts
 const line = lastLineOf({ output: 'a\nb\n', });
 ```
 */
function lastLineOf({
  output,
}: {
  readonly output: string;
},): string {
  return output.split('\n',)
    .filter(function holdsSomething(line,) {
      return line.trim() !== '';
    },)
    .at(-1,)
    ?.trim()
    ?? '';
}

/**
 Asks a start for a slot until it gets one or the deadline passes.

 @param request - the attempt, the clock, the wait and the deadline

 @returns the start's own answer, or the line that says no slot freed in time

 @example
 ```ts
 const outcome = await retryAStartUntilASlotFrees({ startOnce, now: Date.now, wait, deadlineMilliseconds, periodMilliseconds, describedAs: '--name=a --branch=b', });
 ```
 */
export async function retryAStartUntilASlotFrees(
  request: RetryAStartRequest,
): Promise<RetryOutcome> {
  /**
   What the queue has seen: the last refusal, so the line the deadline answers with says what the pools looked like.
   */
  const seen: { refusal: string } = { refusal: '', };
  while (request.now() < request.deadlineMilliseconds) {
    /* oxlint-disable eslint/no-await-in-loop -- one attempt, one wait, then the next attempt, so a freed slot is taken in the order the queue was formed; see doc/lint/sequential-awaits.md */
    /**
     What this attempt at the start answered.
     */
    const attempt = await request.startOnce();
    /* oxlint-enable eslint/no-await-in-loop */
    if (attempt.status !== NO_SLOT_EXIT_STATUS) {
      return attempt;
    }
    seen.refusal = lastLineOf({ output: attempt.output, },);
    // oxlint-disable-next-line eslint/no-await-in-loop -- the wait between two attempts keeps a queued run from asking the process table and systemd as fast as it can; see doc/lint/sequential-awaits.md
    await request.wait(request.periodMilliseconds,);
  }
  return {
    status: NO_SLOT_EXIT_STATUS,
    output: `${(seen.refusal === '') ? '' : `${seen.refusal}\n`}no slot freed before the deadline: ${request.describedAs}`,
  };
}
