/**
 The command's one wait: looks at the running Pi runs until one is past a limit, no run is left, the caller asked for one look,
 or the deadline passes, then ends. Whatever happens, it ends on its own: every wait is bounded by what the deadline leaves.

 It prints nothing itself. It answers with the lines to act on, and the command that called it prints those and nothing else.

 @module
 */

import type { OverdueLook, } from './look-at-pi-runs.ts';
import {
  comparedAgainstTextOf,
  deadlineTextOf,
} from './overdue-line-text.ts';

/**
 What the wait needs.
 */
export type OverdueWatchRequest = {
  /**
   One look at the runs going now, which also keeps what it reported.
   */
  readonly look: () => Promise<OverdueLook>;

  /**
   Where the wait reads the time, as milliseconds since the epoch.
   */
  readonly now: () => number;

  /**
   How the wait passes time, which a test replaces so it neither waits nor needs a clock.
   */
  readonly wait: (milliseconds: number) => Promise<void>;

  /**
   When the wait must be over, as milliseconds since the epoch.
   */
  readonly deadline: number;

  /**
   How long to wait between looks.
   */
  readonly intervalMilliseconds: number;

  /**
   Whether to look once and end, which a caller deciding what to do next asks for.
   */
  readonly once: boolean;
};

/**
 Waits for a Pi run to be past a limit.

 @param look - one look at the runs going now, which keeps what it reported

 @param now - where the wait reads the time

 @param wait - how the wait passes time

 @param deadline - when the wait must be over

 @param intervalMilliseconds - how long to wait between looks

 @param once - whether to look once and end

 @returns the lines to print: the findings, or the one line saying how the wait ended

 @example
 ```ts
 const lines = await watchForOverdueRuns({ look, now: Date.now, wait, deadline, intervalMilliseconds, once: false, },);
 ```
 */
export async function watchForOverdueRuns({
  look,
  now,
  wait,
  deadline,
  intervalMilliseconds,
  once,
}: OverdueWatchRequest,): Promise<readonly string[]> {
  /* oxlint-disable eslint/no-await-in-loop -- a look, then a wait, then the next look: the wait is what keeps one command up
     while the runs work, and a promise per look would put every look at once. See doc/lint/sequential-awaits.md. */
  for (;;) {
    /**
     What the runs going now look like.
     */
    const found = await look();
    if (found.lines
      .length
      > 0) {
      return found.lines;
    }
    if (!found.anyRunning) {
      return ['no Pi agent is running',];
    }
    if (once) {
      return [comparedAgainstTextOf(found.evidence,),];
    }
    /**
     How much of the deadline is left.
     */
    const left = deadline - now();
    if (left <= 0) {
      return [deadlineTextOf(found.evidence,),];
    }
    await wait(Math.min(
      intervalMilliseconds,
      left,
    ),);
  }
  /* oxlint-enable eslint/no-await-in-loop */
}
