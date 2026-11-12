/**
 The retrying start: it asks the pool gate again and again until a pool has room or its own deadline passes, so a run waits for
 the slot it should fill instead of being refused once.

 A shorter `--period-seconds` takes a freed slot sooner, which is how the queue's order is kept.

 @module
 */

import type { CommandOutput, } from './run-command.ts';
import { NO_SLOT_EXIT_STATUS, } from './pool-gate.ts';

/**
 How many kibibytes of what the gate printed are kept for a reader.
 */
const GATE_OUTPUT_KIBIBYTES = 64;

/**
 How many characters one kibibyte of what the gate printed is.
 */
const CHARACTERS_PER_KIBIBYTE = 1_024;

/**
 How much of what the gate printed is kept for a reader. Its answer names the pools and why it started nothing, and it carries the
 launcher's own stack when the launcher failed, whose message hangs above frames, so a bound smaller than the batch's kept bound
 could cut the one line that says why.
 */
export const GATE_OUTPUT_TAIL_CHARACTERS: number = GATE_OUTPUT_KIBIBYTES * CHARACTERS_PER_KIBIBYTE;

/**
 What one retrying start ends with.
 */
export type RetriedStart = {
  /**
   The status the gate ended with on the last attempt, which the calling command exits with.
   */
  readonly status: number;

  /**
   What the calling command prints.
   */
  readonly text: string;
};

/**
 Asks the pool gate for a slot again and again until it starts the run or the deadline passes.

 Every wait is the shorter of the period and what the deadline leaves, so this command ends on its own even when no slot ever
 frees.

 @param attempt - one ask of the pool gate, which a test replaces

 @param now - where the wait reads the time

 @param wait - how the wait passes time

 @param deadline - when asking stops, as milliseconds since the epoch

 @param periodMilliseconds - how long to wait between asks

 @param words - the words the pool gate is asked with, which the deadline line repeats

 @returns the status to exit with, and what to print

 @example
 ```ts
 const outcome = await retryStartUntilAPoolHasRoom({ attempt, now: Date.now, wait, deadline, periodMilliseconds: 30_000, words: ['--name=a',], },);
 ```
 */
export async function retryStartUntilAPoolHasRoom({
  attempt,
  now,
  wait,
  deadline,
  periodMilliseconds,
  words,
}: {
  readonly attempt: () => Promise<CommandOutput>;
  readonly now: () => number;
  readonly wait: (milliseconds: number) => Promise<void>;
  readonly deadline: number;
  readonly periodMilliseconds: number;
  readonly words: readonly string[];
},): Promise<RetriedStart> {
  /* oxlint-disable eslint/no-await-in-loop -- one ask, then a wait, then the next ask: this command exists to keep asking while
     the runs already going fill the pools it reads. See doc/lint/sequential-awaits.md. */
  for (;;) {
    /**
     What the gate answered this time.
     */
    const asked = await attempt();
    if (asked.status !== NO_SLOT_EXIT_STATUS) {
      return {
        status: asked.status,
        text: `${asked.stdout}${asked.stderr}`.slice(-GATE_OUTPUT_TAIL_CHARACTERS,),
      };
    }
    /**
     How much of the deadline is left.
     */
    const left = deadline - now();
    if (left <= 0) {
      break;
    }
    await wait(Math.min(
      periodMilliseconds,
      left,
    ),);
  }
  /* oxlint-enable eslint/no-await-in-loop */
  return {
    status: NO_SLOT_EXIT_STATUS,
    text: `no slot freed before the deadline: ${words.join(' ',)}\n`,
  };
}
