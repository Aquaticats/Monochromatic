/**
 Answering one model error with another attempt, or ending the run on it: the delay, what Pi is sent, and how many attempts a run
 may make.

 @module
 */

import {
  FEED_PREVIEW_CHARACTERS,
  MILLISECONDS_PER_SECOND,
  MODEL_ERROR_RETRY_MAX_DELAY_MILLISECONDS,
} from './driver-constants.ts';
import type {
  CommandAnswer,
  RecordOfPiRun,
  RetryBound,
  RetryTaken,
  RunProgress,
} from './driver-types.ts';
import {
  millisecondsLeftUntil,
} from './deadline.ts';
import { fieldsOf, } from './read-pi-fields.ts';
import {
  anotherAttemptMayFix,
  continuationPromptOf,
  kindOfModelError,
  type ModelErrorKind,
} from './retries-of-a-model-error.ts';
import {
  concurrencyDelayMilliseconds,
  retryDelayMilliseconds,
  waitAskedByProvider,
} from './waits-a-model-error-names.ts';
import { preview, } from './previews-of-pi-messages.ts';
import type { WatchRequest, } from './watch-request.ts';

/**
 Where the watch stands with a model error: nothing to answer, an attempt it owes and is waiting to send, or an attempt that has
 gone and has not ended.
 */
export type RetryWaiting = {
  /**
   The attempt this watch owes: when it may go, and the failure it answers. Its absence means no attempt is waiting.
   */
  owed?: {
    /**
     When the attempt may go, as milliseconds since the epoch.
     */
    readonly at: number;

    /**
     What the model said when its message ended in the error this attempt answers, kept so the prompt names the failure the
     attempt was scheduled for rather than whichever one a later settle names.
     */
    readonly problem: string;

    /**
     How long the driver waited before this attempt, kept so the feed records the wait that was scheduled rather than one read
     again from what may by then be another failure.
     */
    readonly delayMilliseconds: number;

    /**
     What bounds the attempts the run allows for this failure.
     */
    readonly bound: RetryBound;
  };

  /**
   How many settles Pi had produced when this attempt went. The attempt's own settle is the first one after that count, and
   until one comes the error the attempt answers is neither counted again nor answered again.
   */
  settlesAtAttempt?: number;

  /**
   When a run's wait for a request slot must end, set when the first failure that names a limit on requests at once is read. A
   slot frees when another run of the provider's pool finishes, which is a wait no count of attempts names.
   */
  untilSlotsFree?: number;
};

/**
 Sends the continuation prompt that answers a model error with another attempt, and counts the attempt once it has gone.

 @param watch - watch request

 @param retry - where the watch stands with the model error this attempt answers

 @param problem - what the model said when its message ended in an error

 @param delayMilliseconds - how long the driver waited before this attempt

 @param bound - what bounds the attempts the run allows for this failure

 @returns why the run is over, or that it goes on
 */
async function answerWithAnotherAttempt(
  {
    watch,
    retry,
    problem,
    delayMilliseconds,
    bound,
  }: {
    readonly watch: WatchRequest;
    readonly retry: RetryWaiting;
    readonly problem: string;
    readonly delayMilliseconds: number;
    readonly bound: RetryBound
  },
): Promise<RunProgress> {
  /**
   How many settles Pi had produced before this attempt was sent, so the attempt's own settle is told from the one before it.
   */
  const settlesBefore = watch.record
    .settledCount();
  /**
   How many replies had succeeded before this attempt was sent. Pi's answer to the command can arrive in the same read as the
   events of the run it started, so both counts are read before the send.
   */
  const answersBefore = watch.record
    .answersTaken();
  /**
   How many attempts the run has made already.
   */
  const taken = watch.record
    .retriesTaken();
  /**
   Which attempt this is, counting from one.
   */
  const attempt = taken + 1;
  /**
   Pi's answer, waited for within the run's own deadline.
   */
  const answer = await watch.connection
    .send({
      command: {
        type: 'prompt',
        message: continuationPromptOf({ problem, },),
      },
      deadline: watch.deadline,
    },);
  // The attempt is counted here rather than when it was scheduled: a wait the wall clock, the stop file or Pi's exit cuts short
  // leaves a model error standing with no attempt behind it, and the summary must not name an attempt nobody made.
  watch.record
    .countRetry({
      attempt,
      bound,
      delayMilliseconds,
      kind: kindOfModelError(problem,),
      problem,
      answersBefore,
    },);
  /**
   What Pi did with the prompt, or what the driver's own bound named instead.
   */
  const disposition = noteRetryAnswer({
    answer,
    record: watch.record,
  },);
  delete retry.owed;
  retry.settlesAtAttempt = settlesBefore;
  if (answer.ending === 'deadline') {
    return 'wall-clock';
  }
  if (answer.ending === 'pi-exited') {
    return 'pi-exited';
  }
  /**
   Whether Pi refused the prompt itself, or the driver's own silence bound ended the wait.
   */
  if ((answer.ending === 'silence') || (answeredRefusal(answer,))) {
    // Pi refused nothing when the driver's own bound ended the wait, which the run's deadline may be what brought on.
    if (millisecondsLeftUntil(watch.deadline,) === 0) {
      return 'wall-clock';
    }
    return answeredPartlyByACompaction(answer,) ? 'compaction-failed' : 'prompt-rejected';
  }
  if (disposition === 'handled') {
    watch.record
      .countHandledPrompt();
  }
  return 'still-running';
}

/**
 Tells whether a compaction Pi reported failing is why this wait ended rather than a prompt Pi refused.

 @param answer - how waiting for Pi's answer ended

 @returns whether the wait saw a compaction fail
 */
function answeredPartlyByACompaction(answer: CommandAnswer,): boolean {
  if (answer.ending !== 'silence') {
    return false;
  }
  /**
   What Pi said of a compaction this wait saw fail.
   */
  const { compactionProblem } = answer;
  return compactionProblem !== undefined;
}

/**
 Tells whether Pi refused the prompt itself.

 @param answer - how waiting for Pi's answer ended

 @returns whether Pi answered with a refusal
 */
function answeredRefusal(answer: CommandAnswer,): boolean {
  if (answer.ending !== 'answered') {
    return false;
  }
  /**
   Pi's own record, which is what an answered wait carries.
   */
  const { record } = answer;
  return record.success === false;
}

/**
 Notes on the feed what Pi made of the retry prompt, including the bound that ended the wait when Pi answered nothing.

 @param answer - how waiting for Pi's answer ended

 @param record - run's record

 @returns what Pi said it did with the prompt, which is nothing when Pi said nothing at all
 */
function noteRetryAnswer({
  answer,
  record,
}: {
  readonly answer: CommandAnswer;
  readonly record: RecordOfPiRun
},): unknown {
  if (answer.ending !== 'answered') {
    record.note({
      kind: 'retry_prompt',
      success: false,
      error: answer.problem,
    },);
    return undefined;
  }
  /**
   Pi's own record, which is what an answered wait carries.
   */
  const { record: answered } = answer;
  /**
   What Pi did with the prompt.
   */
  const { disposition, } = fieldsOf(answered.data,);
  record.note({
    kind: 'retry_prompt',
    success: answered.success ?? false,
    disposition,
    error: answered.error ?? null,
  },);
  return disposition;
}

/**
 Answers one model error with another attempt, when another attempt may fix it, or ends the run on it.

 A failure another attempt may fix is waited out and then answered with a continuation prompt: Pi drops an assistant message
 that ended in an error when it builds the request for the model, so the attempt reads the last state the model produced
 successfully. The delay grows with each attempt and is cut to the time the run has left, so no attempt is waited for past the
 run's own wall clock. The attempt also waits for Pi to rest: a prompt handed to a Pi that is at work is queued behind that run
 or refused, and the error it answers may yet be recovered by that run's own reply. For the same reason nothing is decided on the
 failure while Pi is at work. Once the retries a run allows have gone, and Pi is at rest, the run ends on the error, as it ended
 before any of this existed.

 @param watch - watch request

 @param retry - where the watch stands with the model error it is answering

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
   How many settles Pi had produced when the last attempt went, if one has gone.
   */
  const { settlesAtAttempt, } = retry;
  if (settlesAtAttempt !== undefined) {
    /**
     How many settles Pi has produced, which tells whether the attempt that went has ended.
     */
    const settled = watch.record
      .settledCount();
    if (settled <= settlesAtAttempt) {
      // An attempt has gone and Pi has not finished with it: the attempt's own settle is what tells the driver whether the
      // failure it answered is recovered, so nothing else is scheduled or sent until one comes.
      return 'still-running';
    }
    delete retry.settlesAtAttempt;
  }
  // Nothing here decides the run while Pi is at work: its reply may recover the error or name a different one, and ending the
  // run now would abort a run whose message is already off the inbox.
  if (watch.record
    .isRunning()) {
    return 'still-running';
  }
  /**
   The error Pi settled on, when nothing after it recovered.
   */
  const failure = watch.record
    .unrecoveredModelError();
  /**
   The attempt this watch owes, when one is waiting.
   */
  const { owed, } = retry;
  if (owed !== undefined) {
    if ((!failure.failed) || (failure.problem !== owed.problem)) {
      // The failure this attempt was scheduled for is recovered or replaced, so the wait owed to it is dropped and what stands
      // now is read afresh below: its own kind, its own wait, and its own place in what the run allows.
      delete retry.owed;
    }
    else if (Date.now() < owed.at) {
      return 'still-running';
    }
    else {
      return await answerWithAnotherAttempt({
        watch,
        retry,
        problem: owed.problem,
        delayMilliseconds: owed.delayMilliseconds,
        bound: owed.bound,
      },);
    }
  }
  if (!failure.failed) {
    return 'still-running';
  }
  /**
   What kind of failure the model named.
   */
  const kind = kindOfModelError(failure.problem,);
  if (!anotherAttemptMayFix(kind,)) {
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
   What this failure leaves the run allowed to do, or why nothing follows it.
   */
  const decision = decisionAboutAnotherAttempt({
    watch,
    retry,
    kind,
  },);
  if (!decision.makeAttempt) {
    if (decision.problem !== undefined) {
      watch.record
        .note({
          kind: 'problem',
          problem: decision.problem,
        },);
    }
    return 'model-error';
  }
  /**
   How long this attempt waits, which is what the feed records for it once it goes.
   */
  const delayMilliseconds = delayBeforeAnAttempt({
    watch,
    kind,
    problem: failure.problem,
    attemptNumber: watch.record
      .retriesTaken()
      + 1,
  },);
  if (kind === 'concurrency-limit') {
    // A reader of the feed sees the wait begin, not only the attempt it ends with, because this one is expected to last about as
    // long as another run does.
    watch.record
      .note({
        kind: 'waiting-for-a-slot',
        waitMilliseconds: delayMilliseconds,
        budgetMilliseconds: concurrencyWaitMillisecondsOf(watch,),
        error: preview({
          value: failure.problem,
          length: FEED_PREVIEW_CHARACTERS.modelError,
        },),
      },);
  }
  retry.owed = {
    at: Date.now() + Math.min(
      delayMilliseconds,
      millisecondsLeftUntil(watch.deadline,),
    ),
    problem: failure.problem,
    delayMilliseconds,
    bound: decision.bound,
  };
  return 'still-running';
}

/**
 What a failure leaves the run allowed to do: make one more attempt under a bound, or end on it.
 */
type AttemptDecision =
  | {
    readonly makeAttempt: false;

  /**
   Why no attempt follows, when the driver has something to say about it beyond the failure itself.
   */
    readonly problem?: string
  }
    | {
      readonly makeAttempt: true;

    /**
     What bounds the attempts the run allows for this failure.
     */
      readonly bound: RetryBound
    };

/**
 Decides whether another attempt follows this failure, and what bounds it.

 A failure another attempt can settle is bounded by how many attempts the run allows. A limit on requests running at once is
 bounded by a total wait instead, because what it waits for is another run of the provider's pool finishing rather than a
 backoff passing, which no count of attempts names.

 @param watch - watch request

 @param retry - where the watch stands with the model error it is answering

 @param kind - what kind of failure the model named

 @returns whether another attempt follows, and what bounds it
 */
function decisionAboutAnotherAttempt({
  watch,
  retry,
  kind,
}: {
  readonly watch: WatchRequest;
  readonly retry: RetryWaiting;
  readonly kind: ModelErrorKind
},): AttemptDecision {
  /**
   How many attempts the run allows.
   */
  const { retryCap, } = watch.options;
  // A run asked for no attempts at all makes none, whatever kind of failure it met: a retry cap of zero is the run this driver
  // made before it retried anything, and the wait a full pool would get does not override a decision the caller made.
  if (retryCap === 0) {
    return { makeAttempt: false, };
  }
  if (kind === 'concurrency-limit') {
    /**
     How long the run waits for a slot, in all.
     */
    const budgetMilliseconds = concurrencyWaitMillisecondsOf(watch,);
    // The wait the run gets for slots, set by the first failure that names one so later ones share it rather than each starting
    // a new wait of their own.
    retry.untilSlotsFree ??= Date.now() + budgetMilliseconds;
    /**
     When the wait must end, which the run's own wall clock can bring forward.
     */
    const until = Math.min(
      retry.untilSlotsFree,
      watch.deadline
    );
    if (Date.now() >= until) {
      return {
        makeAttempt: false,
        problem: `the run waited ${String(budgetMilliseconds,)} ms at the provider's limit on requests running at once, and no slot freed within it, so no further attempt was made`,
      };
    }
    return {
      makeAttempt: true,
      bound: {
        kind: 'wait',
        withinMilliseconds: budgetMilliseconds,
      },
    };
  }
  /**
   How many attempts after a model error the run has made already.
   */
  const taken = watch.record
    .retriesTaken();
  return (taken
    >= watch.options
      .retryCap)
    ? { makeAttempt: false, }
    : {
      makeAttempt: true,
      bound: {
        kind: 'attempts',
        of: watch.options
          .retryCap,
      },
    };
}

/**
 How long this run waits, in all, for a provider to have a request slot free.

 @param watch - watch request

 @returns the milliseconds the wait is bounded by
 */
function concurrencyWaitMillisecondsOf(watch: WatchRequest,): number {
  return watch.options
    .concurrencyWaitSeconds
    * MILLISECONDS_PER_SECOND;
}

/**
 How long this attempt waits before it goes.

 @param watch - watch request

 @param kind - what kind of failure the attempt answers

 @param problem - what the model said when its message ended in an error

 @param attemptNumber - which attempt this is, counting from one

 @returns the milliseconds to wait
 */
function delayBeforeAnAttempt({
  watch,
  kind,
  problem,
  attemptNumber,
}: {
  readonly watch: WatchRequest;
  readonly kind: ModelErrorKind;
  readonly problem: string;
  readonly attemptNumber: number
},): number {
  return (kind === 'concurrency-limit')
    ? concurrencyDelayMilliseconds({
      problem,
      retryNumber: attemptNumber,
    },)
    : retryDelayMilliseconds({
      problem,
      retryNumber: attemptNumber,
      baseSeconds: watch.options
        .retryBaseSeconds,
    },);
}
