/**
 What the driver knows about a run: the feed it writes as events arrive, and the counts that decide when the run is over.

 @module
 */

import type {
  EventObservation,
  PiRecord,
  RecordOfPiRun,
  RetryTaken,
} from './driver-types.ts';
import {
  FEED_PREVIEW_CHARACTERS,
} from './driver-constants.ts';
import {
  type Feed,
  startFeed,
} from './feed-of-a-run.ts';
import {
  type ObservedPiRun,
  startObservingPiRun,
} from './observe-pi-events.ts';
import { preview, } from './previews-of-pi-messages.ts';
import {
  isRateLimit,
  kindOfModelError,
} from './retries-of-a-model-error.ts';

/**
 The model error the summary carries, when Pi settled on one nothing recovered from.

 @param run - what the run's events said

 @param retries - how many attempts after a model error the run made before settling on this one

 @returns the summary's fields, or nothing
 */
function modelErrorOf({
  run,
  retries,
}: {
  readonly run: ObservedPiRun;
  readonly retries: number
},): {
  readonly modelError?: string;
  readonly modelErrorAttempts?: number
} {
  /**
   What the run ended with.
   */
  const failure = run.unrecoveredModelError();
  return failure.failed ? {
    modelError: failure.problem,
    modelErrorAttempts: retries + 1,
  } : {};
}

/**
 The feed's failure as the summary carries it, when the feed could not be written.

 @param runFeed - the run's feed

 @returns the summary's field, or nothing
 */
function feedProblemOf(runFeed: Feed,): { readonly feedProblem?: string } {
  /**
   What the feed ended with.
   */
  const failure = runFeed.problem();
  return failure.failed ? { feedProblem: failure.problem, } : {};
}

/**
 What starting the record of a run needs.
 */
export type RecordOfPiRunRequest = {
  /**
   File that receives one JSON line per noted event.
   */
  readonly feedFile: string;

  /**
   Where the record reads the time. A test gives its own, so a silence is measured instead of waited for.
   */
  readonly now?: () => number;
};

/**
 Starts the record of a run.

 @param feedFile - file that receives one JSON line per noted event

 @param now - where the record reads the time

 @returns the record

 @example
 ```ts
 const record = startRecordOfPiRun({ feedFile: 'feed.jsonl', },);
 ```
 */
export function startRecordOfPiRun({
  feedFile,
  now = Date.now,
}: RecordOfPiRunRequest,): RecordOfPiRun {
  /**
   When the run started.
   */
  const startedAt = now();
  /**
   The run's feed, which also counts as activity.
   */
  const feed = startFeed({
    feedFile,
    startedAt,
    now,
  },);
  /**
   What changes about the driver's own part of the run.
   */
  const state: {
    driverError?: string;

    /**
     How many attempts after a model error the run made.
     */
    retries: number;

    /**
     How many attempts the run made at the error it now stands on, which a reply that succeeded ends.
     */
    retriesSinceAnswer: number;

    /**
     How many assistant messages have replied without an error.
     */
    answers: number;

    /**
     How many model errors named a rate limit or a quota.
     */
    rateLimitErrors: number;
  } = {
    retries: 0,
    retriesSinceAnswer: 0,
    answers: 0,
    rateLimitErrors: 0,
  };

  /**
   When something was last heard or noted.
   */
  const activity: { at: number } = { at: startedAt };

  /**
   How long the run has lasted, for the records the summary keeps.

   @returns milliseconds since it started
   */
  function millisecondsSoFar(): number {
    return now() - startedAt;
  }

  /**
   Writes one feed line, which also counts as activity.

   @param entry - fields of the line
   */
  function note(entry: PiRecord,): void {
    activity.at = now();
    feed.note(entry,);
  }

  /**
   What the run's events say about it.
   */
  const observed = startObservingPiRun({
    note,
    millisecondsSoFar,
    countModelError: function countModelError(problem,) {
      if (isRateLimit(kindOfModelError(problem,),)) {
        state.rateLimitErrors += 1;
      }
    },
    noteAnswer: function noteAnswer() {
      state.answers += 1;
      state.retriesSinceAnswer = 0;
    },
  },);

  return {
    isRunning: observed.isRunning,
    settledCount: observed.settledCount,
    turnCount: observed.turnCount,
    silentMilliseconds: function silentMilliseconds() {
      return now() - activity.at;
    },
    note,
    countHandledPrompt: observed.countPromptHandled,
    feedProblem: feed.problem,
    unrecoveredModelError: observed.unrecoveredModelError,
    retriesTaken: function retriesTaken() {
      return state.retries;
    },
    answersTaken: function answersTaken() {
      return state.answers;
    },
    countRetry: function countRetry(retry: RetryTaken,) {
      state.retries += 1;
      // A reply that arrived while this attempt's command was being answered belongs to this attempt, so the attempt is
      // credited to the failures before it rather than to the ones after it.
      if (state.answers === retry.answersBefore) {
        state.retriesSinceAnswer += 1;
      }
      note({
        kind: 'retry',
        attempt: retry.attempt,
        of: retry.of,
        delayMilliseconds: retry.delayMilliseconds,
        errorKind: retry.kind,
        error: preview({
          value: retry.problem,
          length: FEED_PREVIEW_CHARACTERS.modelError,
        },),
      },);
    },
    noteDriverFailure: function noteDriverFailure(failure,) {
      state.driverError = failure;
      note({
        kind: 'driver-failed',
        failure,
      },);
    },
    observe: function observe(observation,) {
      activity.at = now();
      observed.observe(observation,);
    },
    summarize: function summarize({
      endReason,
      piExit,
      piHasExited,
      sessionStats,
      compactionProblem,
    },) {
      return {
        endReason,
        ...(piHasExited ? { piExit, } : {}),
        totalMilliseconds: now() - startedAt,
        turns: observed.turnCount(),
        settled: observed.settledCount(),
        toolHistogram: observed.toolHistogram(),
        toolErrors: observed.toolErrors(),
        dialogs: observed.dialogs(),
        sessionStats,
        lastAssistantText: observed.lastAssistantText(),
        retries: state.retries,
        rateLimitErrors: state.rateLimitErrors,
        ...modelErrorOf({
          run: observed,
          retries: state.retriesSinceAnswer,
        },),
        ...feedProblemOf(feed,),
        ...(state.driverError === undefined ? {} : { driverError: state.driverError, }),
        ...(compactionProblem === undefined ? {} : { compactionProblem, }),
      };
    },
    close: feed.close,
  };
}
