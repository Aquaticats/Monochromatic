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
  MASKING_NOTHING,
  type TextMasking,
} from './secret-value-masking.ts';
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
 What Pi reported of its own automatic retry giving up on the model error the run stands on, as the summary carries it.

 @param run - what the run's events said

 @returns the summary's fields, or nothing when Pi reported giving up on nothing
 */
function piRetryOf(run: ObservedPiRun,): {
  readonly piRetryGaveUp?: {
    readonly attempts: number;
    readonly problem: string;
  };
} {
  /**
   What Pi reported when it stopped retrying.
   */
  const stopped = run.piRetryGaveUp();
  return stopped.gaveUp ? {
    piRetryGaveUp: {
      attempts: stopped.attempts,
      problem: stopped.problem,
    },
  } : {};
}

/**
 What Pi reported of a compaction, when that is why the run could not go on, as the summary carries it.

 @param run - what the run's events said

 @returns the summary's field, or nothing when Pi reported nothing of a compaction it could not finish
 */
function compactionProblemOf(run: ObservedPiRun,): { readonly compactionProblem?: string } {
  /**
   What Pi said of the compaction the run ended on, if any.
   */
  const failure = run.compactionProblem();
  return failure.failed ? { compactionProblem: failure.problem, } : {};
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

  /**
   The masking every feed line goes through, which is one boundary for every record a run writes: a tool's arguments, a tool's
   result, an inbox preview, a problem, and Pi's own event detail all pass it.

   The field has no default: a caller that writes a run's feed has to say what masks it, so a line nobody thought about cannot be
   written as it came. A caller that holds no values passes {@link MASKING_NOTHING} and says so.
   */
  readonly mask: TextMasking;
};

/**
 Starts the record of a run.

 @param feedFile - file that receives one JSON line per noted event

 @param now - where the record reads the time

 @param mask - the masking every feed line goes through

 @returns the record

 @example
 ```ts
 const record = startRecordOfPiRun({ feedFile: 'feed.jsonl', },);
 ```
 */
export function startRecordOfPiRun({
  feedFile,
  now = Date.now,
  mask,
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
     What the driver failed to read when it asked what the run left running.
     */
    workProblem?: string;

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

    /**
     Whether Pi took the run's first prompt, which is where work may begin.
     */
    tookPrompt: boolean;
  } = {
    retries: 0,
    retriesSinceAnswer: 0,
    answers: 0,
    rateLimitErrors: 0,
    tookPrompt: false,
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
    // One boundary for every line: the entry is masked as the data it serialises to, so a field nobody thought of, a key, and a
    // method an object carries are all covered by the same answer.
    /**
     The entry as data, or why it could not be masked.
     */
    const masked = mask.maskData(entry,);
    feed.note(
      ('value' in masked)
        // oxlint-disable-next-line typescript/no-unsafe-type-assertion -- the line's own fields are what the round trip proved.
        ? masked.value as PiRecord
        : {
          kind: 'withheld',
          cause: masked.cause,
        },
    );
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
    countPromptTaken: function countPromptTaken() {
      state.tookPrompt = true;
    },
    promptTaken: function promptTaken() {
      return state.tookPrompt;
    },
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
    noteWorkProblem: function noteWorkProblem(failure,) {
      state.workProblem = failure;
      note({
        kind: 'work-unreadable',
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
      piStartAttempts,
      piStartFailures,
      piStartStoppedBy,
      sessionAtRest,
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
        piStartAttempts: piStartAttempts ?? 1,
        ...(piStartFailures === undefined ? {} : { piStartFailures, }),
        ...(piStartStoppedBy === undefined ? {} : { piStartStoppedBy, }),
        ...(sessionAtRest === undefined ? {} : { sessionAtRest, }),
        ...modelErrorOf({
          run: observed,
          retries: state.retriesSinceAnswer,
        },),
        ...piRetryOf(observed,),
        ...feedProblemOf(feed,),
        ...(state.driverError === undefined ? {} : { driverError: state.driverError, }),
        ...(state.workProblem === undefined ? {} : { workProblem: state.workProblem, }),
        ...compactionProblemOf(observed,),
      };
    },
    close: feed.close,
  };
}
