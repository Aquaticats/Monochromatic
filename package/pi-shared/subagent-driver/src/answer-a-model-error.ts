/**
 Answering one model error with another attempt, or ending the run on it: the delay, what Pi is sent, and how many attempts a run
 may make.

 @module
 */

import { MODEL_ERROR_RETRY_MAX_DELAY_MILLISECONDS, } from './driver-constants.ts';
import type {
  RetryTaken,
  RunProgress,
} from './driver-types.ts';
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
  waitAskedByProvider,
} from './retries-of-a-model-error.ts';
import type { WatchRequest, } from './watch-request.ts';

/**
 Where the watch stands with a model error: nothing to answer, a delay it is waiting out before the next attempt, or an attempt
 that has gone and has not ended.
 */
export type RetryWaiting = {
  /**
   When the next attempt may go, as milliseconds since the epoch. Its absence means no attempt is waiting.
   */
  at?: number;

  /**
   What the model said when its message ended in the error this attempt answers, kept so the prompt names the failure the
   attempt was scheduled for rather than whichever one a later settle names.
   */
  problem?: string;

  /**
   How many settles Pi had produced when this attempt went. The attempt's own settle is the first one after that count, and
   until one comes the error the attempt answers is neither counted again nor answered again.
   */
  settlesAtAttempt?: number;
};

/**
 What the attempt after a model error is: which one this is, how long it waited, and the failure it answers.

 Read where the attempt is sent, so the attempt a run counts is one it made, and the wait it was scheduled for is the wait the
 feed records.

 @param watch - watch request

 @param problem - what the model said when its message ended in the error this attempt answers

 @returns the attempt as the feed and the summary carry it
 */
function attemptAfterTheError({
  watch,
  problem,
}: {
  readonly watch: WatchRequest;
  readonly problem: string
},): RetryTaken {
  /**
   Which attempt this is, counting from one.
   */
  const attempt = watch.record
    .retriesTaken() + 1;
  return {
    attempt,
    of: watch.options
      .retryCap,
    delayMilliseconds: retryDelayMilliseconds({
      problem,
      retryNumber: attempt,
      baseSeconds: watch.options
        .retryBaseSeconds,
    },),
    kind: kindOfModelError(problem,),
    problem,
  };
}

/**
 Sends the continuation prompt that answers a model error with another attempt, and counts the attempt once it has gone.

 @param watch - watch request

 @param retry - where the watch stands with the model error this attempt answers

 @param problem - what the model said when its message ended in an error

 @returns why the run is over, or that it goes on
 */
async function answerWithAnotherAttempt(
  {
    watch,
    retry,
    problem,
  }: {
    readonly watch: WatchRequest;
    readonly retry: RetryWaiting;
    readonly problem: string
  },
): Promise<RunProgress> {
  /**
   How many settles Pi had produced before this attempt, so the attempt's own settle is told from the one before it.
   */
  const settlesBefore = watch.record
    .settledCount();
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
  // The attempt is counted here rather than when it was scheduled: a wait the wall clock, the stop file or Pi's exit cuts short
  // leaves a model error standing with no attempt behind it, and the summary must not name an attempt nobody made.
  watch.record
    .countRetry(attemptAfterTheError({
      watch,
      problem,
    },),);
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
  delete retry.at;
  delete retry.problem;
  retry.settlesAtAttempt = settlesBefore;
  if (answer.success === false) {
    if (millisecondsLeftUntil(watch.deadline,) === 0) {
      return 'wall-clock';
    }
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
 run's own wall clock. The attempt also waits for Pi to rest: a prompt handed to a Pi that is at work is queued behind that run
 or refused, and the error it answers may yet be recovered by that run's own reply. Once the retries a run allows have gone, the
 run ends on the error, as it ended before any of this existed.

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
   Where a delay already counted for this error stands, which is the attempt it owes, or the attempt that has gone.
   */
  const {
    at,
    problem: awaited,
    settlesAtAttempt,
  } = retry;
  if (settlesAtAttempt !== undefined) {
    if (watch.record
      .settledCount() <= settlesAtAttempt) {
      // An attempt has gone and Pi has not finished with it: the attempt's own settle is what tells the driver whether the
      // failure it answered is recovered, so nothing else is scheduled or sent until one comes.
      return 'still-running';
    }
    delete retry.settlesAtAttempt;
  }
  if ((at !== undefined) && (awaited !== undefined)) {
    if (Date.now() < at) {
      return 'still-running';
    }
    if (watch.record
      .isRunning()) {
      // A prompt sent to a Pi that is at work is queued behind that run or refused, and either way this attempt lands in a run
      // it did not start, whose reply may still recover the error. The attempt stays owed until Pi rests.
      return 'still-running';
    }
    return await answerWithAnotherAttempt({
      watch,
      retry,
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
   The wait the provider asked for, when it named one.
   */
  const asked = waitAskedByProvider(failure.problem,);
  if (asked.asked && (asked.milliseconds > MODEL_ERROR_RETRY_MAX_DELAY_MILLISECONDS)) {
    // A provider that asks for longer than the largest delay is not retried inside its window: an attempt early in it reaches
    // the same answer, so the run ends on the error and a later run takes it up once the window has passed.
    watch.record
      .note({
        kind: 'problem',
        problem: `the provider asked for a wait of ${String(asked.milliseconds,)} ms, longer than the ${String(MODEL_ERROR_RETRY_MAX_DELAY_MILLISECONDS,)} ms a retry waits, so no further attempt was made`,
      },);
    return 'model-error';
  }
  /**
   How long this attempt waits.
   */
  const delayMilliseconds = attemptAfterTheError({
    watch,
    problem: failure.problem,
  },).delayMilliseconds;
  retry.at = Date.now() + Math.min(
    delayMilliseconds,
    millisecondsLeftUntil(watch.deadline,),
  );
  retry.problem = failure.problem;
  return 'still-running';
}
