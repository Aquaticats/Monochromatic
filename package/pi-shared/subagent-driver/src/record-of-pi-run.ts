/**
 What the driver knows about a run: the feed it writes as events arrive, and the counts that decide when the run is over.

 @module
 */

import type {
  EventObservation,
  PiRecord,
  RecordOfPiRun,
} from './driver-types.ts';
import {
  type Feed,
  startFeed,
} from './feed-of-a-run.ts';
import {
  type ObservedPiRun,
  startObservingPiRun,
} from './observe-pi-events.ts';

/**
 The model error the summary carries, when Pi settled on one nothing recovered from.

 @param run - what the run's events said

 @returns the summary's field, or nothing
 */
function modelErrorOf(run: ObservedPiRun,): { readonly modelError?: string } {
  /**
   What the run ended with.
   */
  const failure = run.unrecoveredModelError();
  return failure.failed ? { modelError: failure.problem, } : {};
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
  const state: { driverError?: string } = {};

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
        ...modelErrorOf(observed,),
        ...feedProblemOf(feed,),
        ...(state.driverError === undefined ? {} : { driverError: state.driverError, }),
      };
    },
    close: feed.close,
  };
}
