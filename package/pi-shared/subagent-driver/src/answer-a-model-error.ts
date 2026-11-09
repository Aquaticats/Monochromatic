/**
 Answering one model error with another attempt, or ending the run on it: the delay, what Pi is sent, and how many attempts a run
 may make.

 @module
 */

import type { RunProgress, } from './driver-types.ts';
import {
  answerMillisecondsBefore,
  millisecondsLeftUntil,
} from './deadline.ts';
import { fieldsOf, } from './read-pi-fields.ts';
import {
  anotherAttemptMayFix,
  continuationPromptOf,
  kindOfModelError,
  retryDelayMilliseconds,
} from './retries-of-a-model-error.ts';
import type { WatchRequest, } from './watch-request.ts';

/**
 Where the watch stands with a model error: nothing to answer, or a delay it is waiting out before the next attempt.
 */
export type RetryWaiting = {
  /**
   When the next attempt may go, as milliseconds since the epoch. Its absence means no attempt is waiting.
   */
  at?: number;

  /**
   What the model said when its message ended in the error this attempt answers, kept because counting the attempt takes the
   error as answered and Pi reports no new one until the attempt ends.
   */
  problem?: string;
};

/**
 Sends the continuation prompt that answers a model error with another attempt.

 @param watch - watch request

 @param problem - what the model said when its message ended in an error

 @returns why the run is over, or that it goes on
 */
async function answerWithAnotherAttempt(
  {
    watch,
    problem,
  }: {
    readonly watch: WatchRequest;
    readonly problem: string
  },
): Promise<RunProgress> {
  /**
   Pi's answer, waited for only as long as the run has left.
   */
  const answer = await watch.connection
    .send({
      command: {
        type: 'prompt',
        message: continuationPromptOf({ problem, },),
      },
      answerMilliseconds: answerMillisecondsBefore(watch.deadline,),
    },);
  /**
   What Pi did with the prompt.
   */
  const { disposition, } = fieldsOf(answer.data,);
  watch.record
    .note({
      kind: 'retry_prompt',
      success: answer.success,
      disposition,
      error: answer.error ?? null,
    },);
  if (answer.success === false) {
    return watch.connection
      .hasExited() ? 'pi-exited' : 'prompt-rejected';
  }
  if (disposition === 'handled') {
    watch.record
      .countHandledPrompt();
  }
  return 'still-running';
}

/**
 Answers one model error with another attempt, when another attempt may fix it, or ends the run on it.

 A failure another attempt may fix is waited out and then answered with a continuation prompt: Pi drops an assistant message
 that ended in an error when it builds the request for the model, so the attempt reads the last state the model produced
 successfully. The delay grows with each attempt and is cut to the time the run has left, so no attempt is waited for past the
 run's own wall clock. Once the retries a run allows have gone, the run ends on the error, as it ended before any of this
 existed.

 @param watch - watch request

 @param retry - where the watch stands with the model error it last counted

 @returns why the run is over, or that it goes on

 @example
 ```ts
 const progress = await answerModelError({ watch, retry, },);
 ```
 */
export async function answerModelError(
  {
    watch,
    retry,
  }: {
    readonly watch: WatchRequest;
    readonly retry: RetryWaiting
  },
): Promise<RunProgress> {
  /**
   Where a delay already counted for this error stands, which is the attempt it owes.
   */
  const {
    at,
    problem: awaited,
  } = retry;
  if ((at !== undefined) && (awaited !== undefined)) {
    if (Date.now() < at) {
      return 'still-running';
    }
    delete retry.at;
    delete retry.problem;
    return await answerWithAnotherAttempt({
      watch,
      problem: awaited,
    },);
  }
  /**
   The error Pi settled on, when nothing after it recovered.
   */
  const failure = watch.record
    .unrecoveredModelError();
  if (!failure.failed) {
    delete retry.at;
    return 'still-running';
  }
  /**
   What kind of failure the model named.
   */
  const kind = kindOfModelError(failure.problem,);
  /**
   How many attempts after a model error the run has made already.
   */
  const taken = watch.record
    .retriesTaken();
  if ((!anotherAttemptMayFix(kind,)) || (taken
    >= watch.options
      .retryCap)) {
    return 'model-error';
  }
  /**
   How long this attempt waits.
   */
  const delayMilliseconds = retryDelayMilliseconds({
    problem: failure.problem,
    retryNumber: taken + 1,
    baseSeconds: watch.options
      .retryBaseSeconds,
  },);
  watch.record
    .countRetry({
      attempt: taken + 1,
      of: watch.options
        .retryCap,
      delayMilliseconds,
      kind,
      problem: failure.problem,
    },);
  retry.at = Date.now() + Math.min(
    delayMilliseconds,
    millisecondsLeftUntil(watch.deadline,),
  );
  retry.problem = failure.problem;
  return 'still-running';
}
