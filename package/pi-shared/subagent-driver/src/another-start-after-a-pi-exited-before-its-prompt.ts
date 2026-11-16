/**
 When a start that ended before its Pi took the prompt is started again, how long the driver waits first, and when it stops.

 Pi writes why it will not start to its standard error and nowhere else, and it exits there before the prompt is read, so such a
 run did no work at all. Of the 1,936 finished runs in `${HOME}/temp/agent/pi-runs` read on 2026-11-14, 18 ended that way: 13 on
 an extension that could not load a module, each start failing in 2,096 to 20,491 ms, one on a logger with no space left on the
 device (6,097 ms), and four on Pi's own retry of a model-store lock another process held, each start spending 127,176 to
 247,556 ms. Those are conditions that pass as often as they stand, so the driver rides them out with a bounded number of starts,
 and when they do not pass it ends the run saying what Pi printed.

 A Pi that took its prompt is never started again, whatever it does afterwards: the prompt is where work may begin, and a second
 Pi would begin it a second time.

 @module
 */

import {
  START_ATTEMPTS,
} from './driver-constants.ts';
import type {
  EndReason,
  NoFurtherStart,
  PiExit,
} from './driver-types.ts';

/**
 What the driver does after a start ended before its Pi took the prompt: start Pi again after a pause, or stop and say why.
 */
export type AnotherStart =
  | {
    readonly startAgain: false;

    /**
     Which bound stopped the starts: the number of them, the time one start-up may take, or the run's own wall clock.
     */
    readonly why: NoFurtherStart;
  }
    | {
      readonly startAgain: true;

      /**
       Milliseconds to wait before starting Pi again.
       */
      readonly pauseMilliseconds: number;
    };

/**
 A start that ended before its Pi took the prompt, or the reason a start was not one: absence travels as a value, as everywhere
 else in this package, so no caller reads a nullish member.
 */
export type StartThatTookNoPrompt =
  | { readonly tookNoPrompt: false }
    | {
      readonly tookNoPrompt: true;

      /**
       How that Pi ended.
       */
      readonly exit: PiExit;
    };

/**
 Reads a start that ended before its Pi took the prompt, which is the only ending the driver starts Pi again after.

 A Pi the operating system refused to start is not one of them: nothing about the machine changes between two such tries, and a
 pause would only make the run end later with the same answer. Nor is a Pi that took the prompt or started a turn, because work
 may have begun.

 @param endReason - why the start left the run

 @param tookPrompt - whether Pi answered the run's first prompt

 @param turnsStarted - how many turns Pi started before it left

 @param exit - how Pi ended

 @returns the exit for a start that ended before it took the prompt, or that this was no such start

 @example
 ```ts
 const start = startThatTookNoPromptOf({ endReason: 'pi-exited', tookPrompt: false, turnsStarted: 0, exit: { code: 1, }, },);
 ```
 */
export function startThatTookNoPromptOf({
  endReason,
  tookPrompt,
  turnsStarted,
  exit,
}: {
  readonly endReason: EndReason;
  readonly tookPrompt: boolean;
  readonly turnsStarted: number;
  readonly exit: PiExit;
},): StartThatTookNoPrompt {
  if (endReason !== 'pi-exited') {
    return { tookNoPrompt: false, };
  }
  if (tookPrompt || (turnsStarted > 0)) {
    return { tookNoPrompt: false, };
  }
  if (exit.processError !== undefined) {
    return { tookNoPrompt: false, };
  }
  if ((exit.code === undefined) && (exit.signal === undefined)) {
    return { tookNoPrompt: false, };
  }
  return {
    tookNoPrompt: true,
    exit,
  };
}

/**
 Says whether Pi is started again after a start that ended before it took the prompt, and how long the driver waits first.

 Three bounds answer: the starts a run allows, the time its whole sequence of starts may take, and the run's own wall clock. A
 pause that would leave no time in one of the first two is not made, so the driver never spends a pause on a start that has no
 room to answer inside.

 @param startsTaken - how many times Pi has been started so far

 @param pauseMilliseconds - how long the driver waits before it starts Pi again

 @param now - where the driver reads the time, as milliseconds since the epoch

 @param wallDeadline - when the run must be over

 @param startSequenceDeadline - when the run's whole sequence of starts must be over

 @returns to start again after the pause, or to stop and why

 @example
 ```ts
 const another = anotherStartOf({ startsTaken: 1, pauseMilliseconds: 10_000, now: Date.now(), wallDeadline: Date.now() + 600_000, startSequenceDeadline: Date.now() + 120_000, },);
 ```
 */
export function anotherStartOf({
  startsTaken,
  pauseMilliseconds,
  now,
  wallDeadline,
  startSequenceDeadline,
}: {
  readonly startsTaken: number;
  readonly pauseMilliseconds: number;
  readonly now: number;
  readonly wallDeadline: number;
  readonly startSequenceDeadline: number;
},): AnotherStart {
  if (startsTaken >= START_ATTEMPTS) {
    return {
      startAgain: false,
      why: 'tries-spent',
    };
  }
  /**
   When the pause would end, which is when the next start could begin.
   */
  const startsAt = now + pauseMilliseconds;
  if (startsAt >= Math.min(
    wallDeadline,
    startSequenceDeadline,
  )) {
    return {
      startAgain: false,
      why: (startsAt >= wallDeadline) ? 'wall-clock-too-near' : 'start-sequence-spent',
    };
  }
  return {
    startAgain: true,
    pauseMilliseconds,
  };
}

/**
 Waits before starting Pi again, within the bound the starts share, and says whether there is still time to start.

 @param pauseMilliseconds - how long the driver waits before it starts Pi again

 @param deadline - when the pause must be over, which is the nearer of the run's wall clock and the sequence of starts

 @param now - where the driver reads the time

 @param wait - how a wait passes time, which a test replaces

 @returns whether the deadline still leaves time for another start

 @example
 ```ts
 const inTime = await waitForAnotherStart({ pauseMilliseconds: 10_000, deadline, now: Date.now, wait, },);
 ```
 */
export async function waitForAnotherStart({
  pauseMilliseconds,
  deadline,
  now,
  wait,
}: {
  readonly pauseMilliseconds: number;
  readonly deadline: number;
  readonly now: () => number;
  readonly wait: (milliseconds: number) => Promise<void>;
},): Promise<boolean> {
  /**
   How much of the bound is left when the pause is due.
   */
  const left = deadline - now();
  if (left <= 0) {
    return false;
  }
  await wait(Math.min(
    pauseMilliseconds,
    left,
  ),);
  return now() < deadline;
}
