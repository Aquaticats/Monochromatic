/**
 The part of a run after the prompt is sent: watches the limits, delivers the inbox, and decides when the run is over.

 @module
 */

import { readdir, } from 'node:fs/promises';
import { setTimeout as wait, } from 'node:timers/promises';
import {
  POLL_MILLISECONDS,
  STOP_FILE_NAME,
} from './driver-constants.ts';
import type {
  EndReason,
  RunProgress,
} from './driver-types.ts';
import {
  answerModelError,
  type RetryWaiting,
} from './answer-a-model-error.ts';
import type { WorkWaitedFor, } from './kept-while-work-runs.ts';
import { endingOfARestingRun, } from './ending-of-a-resting-run.ts';
import {
  deliverHeldMessage,
  deliverReadyMessage,
  type HeldMessage,
  holdMessagesTheRunOwes,
  readyMessages,
} from './messages-from-the-inbox.ts';
import type { WatchRequest, } from './watch-request.ts';
import { mayEndIdle, } from './when-a-run-may-end.ts';

/**
 Looks once at every limit that could end a run.

 @param watch - watch request

 @param wallDeadline - when the run must be over

 @returns why the run is over, or that it goes on
 */
async function progressAgainstLimits(
  {
    watch,
    wallDeadline,
  }: {
    readonly watch: WatchRequest;
    readonly wallDeadline: number
  },
): Promise<RunProgress> {
  if (watch.connection
    .hasExited()) {
    return 'pi-exited';
  }
  if (Date.now() >= wallDeadline) {
    return 'wall-clock';
  }
  if ((await readdir(watch.options
    .runFolder,)).includes(STOP_FILE_NAME,)) {
    return 'stop-file';
  }
  return watch.record
    .turnCount()
    >= watch.options
      .turnCap ? 'turn-cap' : 'still-running';
}

/**
 Checks the limits and delivers the next inbox message, on every tick, until something ends the run. One message per tick keeps
 them in order, each after Pi has answered the one before. A message is waited for only as long as the run has left, so a
 stalled answer cannot keep the stop file or the wall clock from being seen.

 A message Pi refused because it had begun a turn is held and sent again once Pi rests, ahead of the inbox, so no message is
 dropped by a race. A message the run ends before Pi takes is kept aside and named in the summary.

 @param watch - watch request

 @returns why the run ended

 @example
 ```ts
 const endReason = await watchUntilEnded(watch,);
 ```
 */
export async function watchUntilEnded(watch: WatchRequest,): Promise<EndReason> {
  /**
   The inbox messages claimed and not yet taken: Pi refused one because it had begun a turn, so the same message goes again once
   Pi rests, within the run's deadline. Disposing it keeps and names every message still owed when the run ends, or throws.
   */
  await using owed = holdMessagesTheRunOwes(watch,);
  return await watchWhileItGoesOn({
    watch,
    held: owed.claimed,
  },);
}

/**
 Watches one run until something ends it, holding the messages Pi was not ready to take.

 @param watch - watch request

 @param held - where a message Pi refused at work is held, which this fills and empties

 @returns why the run ended
 */
async function watchWhileItGoesOn({
  watch,
  held,
}: {
  readonly watch: WatchRequest;
  readonly held: HeldMessage[]
},): Promise<EndReason> {
  /**
   Where the watch stands with the model error it last answered, so one error is answered by one attempt.
   */
  const retry: RetryWaiting = {};
  /**
   The work this run left running, once the driver has seen it and is waiting for it to end; nothing until then.
   */
  const waited: WorkWaitedFor = {};
  /* oxlint-disable eslint/no-await-in-loop -- a look at the limits, then one message and Pi's answer before the next look; two
     deliveries at once would reorder the inbox. See doc/lint/sequential-awaits.md. */
  for (;;) {
    // The wait is what keeps the driver up between looks: it is the only handle a run has while Pi says nothing.
    await wait(POLL_MILLISECONDS,);
    /**
     Where the run stands against its limits.
     */
    const progress = await progressAgainstLimits({
      watch,
      wallDeadline: watch.deadline,
    },);
    if (progress !== 'still-running') {
      // A run the wall clock ended while it waited for work it left running did not end with nothing to do, and its summary
      // must not read as one that did.
      if ((progress === 'wall-clock') && (waited.work !== undefined)) {
        watch.record
          .note({
            kind: 'problem',
            problem: `the run ended on its wall clock while it waited for the work it left running: ${waited.work
              .join(', ',)}`,
          },);
      }
      return progress;
    }
    /**
     Where the run stands with the model error Pi settled on, if it settled on one.
     */
    const afterModelError = await answerModelError({
      watch,
      retry,
    },);
    if (afterModelError !== 'still-running') {
      return afterModelError;
    }
    // A message the run still owes is delivered before the inbox, and out of turn only when Pi rests: a message it refused while
    // Pi worked is no answer about the message, so it waits for the rest it needs rather than being dropped or reordered.
    if (retry.owed === undefined) {
      /**
       The message the run still owes Pi, when it owes one.
       */
      const [heldMessage,] = held;
      if (heldMessage !== undefined) {
        if (!watch.record
          .isRunning()) {
          /**
           What became of the message this look sent again.
           */
          const delivered = await deliverHeldMessage({
            watch,
            held: heldMessage,
          },);
          if (delivered.ending !== 'at-work') {
            held.shift();
          }
        }
      }
      else {
        /**
         The next message in the inbox, which waits out a retry that is about to go: a prompt meant for a resting Pi must not
         start a run the retry prompt then lands in the middle of.
         */
        const [nextMessage,] = await readyMessages(watch.inbox,);
        if (nextMessage !== undefined) {
          /**
           What became of the message this look took.
           */
          const delivered = await deliverReadyMessage({
            watch,
            name: nextMessage,
          },);
          if ((delivered !== 'could-not-take') && (delivered.ending === 'at-work')) {
            held.push(delivered.held,);
          }
        }
        // A run waiting out a delay before another attempt is not idle: it is about to answer the error it settled on. A ready
        // message keeps the run up whether or not it could be taken: it is still the inbox's, and it is not this look's to end.
        else if (mayEndIdle({
          watch,
          ...(waited.wokenAtSettle === undefined ? {} : { wokenAtSettle: waited.wokenAtSettle, }),
        },)) {
          /**
           What the run came to: the work it left running, then what its last answer said. Both steps await inside, so the decision
           that the run may end is taken again on the state they leave, and a turn Pi began in that window keeps the run up.
           */
          const ending = await endingOfARestingRun({
            watch,
            waited,
          },);
          if (ending === 'still-running') {
            continue;
          }
          return ending;
        }
      }
    }
  }
  /* oxlint-enable eslint/no-await-in-loop */
}
