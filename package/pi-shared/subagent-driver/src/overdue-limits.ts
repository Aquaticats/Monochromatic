/**
 What the finished runs say a running run can be expected to have done: how long one takes, and how long one tool call takes.
 Both limits are computed from the runs folder rather than written down, so they follow the work instead of the day they were
 measured.

 @module
 */

import { join, } from 'node:path';
import { errorCodeOf, } from './error-code-of.ts';
import { finishedPiRunsIn, } from './finished-pi-runs.ts';
import {
  OPEN_CALL_PERCENTILE,
  OVERDUE_PERCENTILE,
} from './overdue-constants.ts';
import { toolCallTimesOfFeed, } from './tool-call-times-of-a-feed.ts';

/**
 The code the operating system gave a missing path.
 */
const MISSING_PATH_CODE = 'ENOENT';

/**
 A sample with nothing in it, as a symbol so no measured value can be mistaken for it.
 */
export const NO_SAMPLE: unique symbol = Symbol('sample held zero measured values when a percentile was asked of it',);

/**
 A run that wrote no feed, as a symbol so no list of durations can be mistaken for it.
 */
const NO_FEED: unique symbol = Symbol('finished run wrote no feed',);

/**
 What the finished runs in one folder say.
 */
export type FinishedRunEvidence = {
  /**
   How many finished runs the folder holds whose length is evidence.
   */
  readonly finishedRuns: number;

  /**
   How long a run lasts when it goes well, as milliseconds at the overdue percentile. Absent when no finished run is there to
   learn from, because then nothing is known about how long work takes.
   */
  readonly overdueMilliseconds?: number;

  /**
   How many finished tool calls the folder holds.
   */
  readonly toolCalls: number;

  /**
   How long one call takes when it is slow, as milliseconds at the open-call percentile. Absent when the folder holds no
   finished call, and with a sample too small for that percentile it is the longest call there is.
   */
  readonly openCallMilliseconds?: number;
};

/**
 Reads one value of a sample at a percentile, by rank.

 A sample smaller than the percentile needs, such as four calls at the 99.9th percentile, answers its own longest value: the
 rank is clamped to the end of the sample rather than inventing a shorter one.

 @param sample - the measured values

 @param fraction - the share of the sample at or below the answer, above zero and at most one

 @returns the value at that rank, or {@link NO_SAMPLE} for an empty sample

 @example
 ```ts
 const slowest = percentileOf({ sample: [1, 2, 3, 4,], fraction: 0.9, },);
 ```
 */
export function percentileOf({
  sample,
  fraction,
}: {
  readonly sample: readonly number[];
  readonly fraction: number;
},): number | typeof NO_SAMPLE {
  if (sample.length === 0) {
    return NO_SAMPLE;
  }
  /**
   The sample from the smallest value to the largest.
   */
  const sorted = sample.toSorted(function bySize(
    left,
    right,
  ) {
    return left - right;
  },);
  /**
   The place of the answer in the sorted sample.
   */
  const rank = Math.max(
    0,
    Math.ceil(fraction * sorted.length,) - 1,
  );
  /**
   The value at that place, which is inside the sample because the place was clamped to it.
   */
  const atRank = sorted[rank];
  return atRank ?? NO_SAMPLE;
}

/**
 Reads how long the ended calls of one finished run took.

 @param folder - the run's folder

 @returns how long its ended calls took, or {@link NO_FEED}

 @example
 ```ts
 const times = await closedCallTimesIn('/runs/a-1',);
 ```
 */
async function closedCallTimesIn(folder: string,): Promise<readonly number[] | typeof NO_FEED> {
  try {
    /**
     The run's calls.
     */
    const times = await toolCallTimesOfFeed({ feedFile: join(
      folder,
      'feed.jsonl',
    ), },);
    return times.closedMilliseconds;
  }
  catch (error) {
    if (errorCodeOf(error,) === MISSING_PATH_CODE) {
      // A run folder without a feed is one whose feed could not be written; it says nothing about how long calls take.
      return NO_FEED;
    }
    throw error;
  }
}

/**
 Reads what a runs folder says about how long work takes.

 @param runsFolder - folder the driver writes one run folder per run into

 @returns the evidence a running run is judged against

 @example
 ```ts
 const evidence = await evidenceFromRunsFolder({ runsFolder: '/runs', },);
 ```
 */
export async function evidenceFromRunsFolder({
  runsFolder,
}: {
  readonly runsFolder: string;
}): Promise<FinishedRunEvidence> {
  /**
   Every finished run.
   */
  const finished = await finishedPiRunsIn({ runsFolder, },);
  /**
   The finished runs whose length says how long the work takes.
   */
  const runsWithEvidence = finished.filter(function countsAsWork(run,) {
    return run.lengthCountsAsWork;
  },);
  /**
   The calls each finished run made, with the runs that wrote no feed left out. A run whose length is no evidence still
   finished the calls it finished.
   */
  const perRunCalls = await Promise.all(finished.map(function callsOf(run,) {
    return closedCallTimesIn(run.folder,);
  },));
  /**
   Every call in the folder that ended.
   */
  const closedMilliseconds = perRunCalls
    .filter(function isMeasured(calls,): calls is readonly number[] {
      return (typeof calls) !== 'symbol';
    },)
    .flat();
  /**
   How long a run lasts when it goes well.
   */
  const overdueMilliseconds = percentileOf({
    sample: runsWithEvidence.map(function lengthOf(run,) {
      return run.totalMilliseconds;
    },),
    fraction: OVERDUE_PERCENTILE,
  },);
  /**
   How long one call takes when it is slow.
   */
  const openCallMilliseconds = percentileOf({
    sample: closedMilliseconds,
    fraction: OPEN_CALL_PERCENTILE,
  },);
  return {
    finishedRuns: runsWithEvidence.length,
    ...((typeof overdueMilliseconds) === 'symbol' ? {} : { overdueMilliseconds, }),
    toolCalls: closedMilliseconds.length,
    ...((typeof openCallMilliseconds) === 'symbol' ? {} : { openCallMilliseconds, }),
  };
}
