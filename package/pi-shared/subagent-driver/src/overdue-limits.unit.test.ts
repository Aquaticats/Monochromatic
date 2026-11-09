/**
 What the finished runs say a running run can be expected to have done, read from a runs folder a test writes and throws away.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  NO_SAMPLE,
  evidenceFromRunsFolder,
  percentileOf,
  standInRunsFolder,
} from '../dist/final/node/index.mjs';


/**
 A finished run of a minute, as a summary reports it.
 */
const MINUTE = 60_000;

/**
 One call that ended after thirty seconds.
 */
const THIRTY_SECONDS = 30_000;

/**
 One call that ended after ten minutes.
 */
const TEN_MINUTES = 600_000;

/**
 A run of twelve minutes, long enough for the ten-minute call inside it.
 */
const TWELVE_MINUTES = 12 * MINUTE;

/**
 Reads the percentiles this test expects without going through the module under test. Every sample it is asked about holds a
 value, so the answer is always one of them.

 @param sample - the measured values

 @param fraction - the share of the sample at or below the answer

 @returns the value at that rank

 @example
 ```ts
 const expected = rankOf([1, 2, 3,], 0.9,);
 ```
 */
function rankOf(sample: readonly number[], fraction: number,): number {
  /**
   The sample from the smallest value to the largest.
   */
  const sorted = sample.toSorted(function bySize(left, right,) {
    return left - right;
  },);
  return sorted[Math.ceil(fraction * sorted.length,) - 1] ?? 0;
}

await describe({
  name: percentileOf.name,
  children: [
    it({
      name: 'takes the value at the given rank of a sample it does not change',
      fn: async () => {
        /**
         The sample, in an order the answer must not depend on.
         */
        const sample = [40, 10, 30, 20,];
        expect(percentileOf({
          sample,
          fraction: 0.9,
        },),).toBe(40,);
        expect(sample,).toEqual([40, 10, 30, 20,],);
      },
    },),
    it({
      name: 'keeps a value below the largest when the sample is big enough for that rank',
      fn: async () => {
        /**
         A hundred values, one to a hundred.
         */
        const sample = Array.from(
          { length: 100, },
          function count(_unused, index,) {
            return index + 1;
          },
        );
        expect(percentileOf({
          sample,
          fraction: 0.9,
        },),).toBe(rankOf(sample, 0.9,),);
        expect(percentileOf({
          sample,
          fraction: 0.9,
        },),).toBe(90,);
      },
    },),
    it({
      name: 'answers its own largest value for a sample smaller than the rank it was asked for',
      fn: async () => {
        expect(percentileOf({
          sample: [5, 1,],
          fraction: 0.999,
        },),).toBe(5,);
      },
    },),
    it({
      name: 'answers a symbol rather than a number for an empty sample',
      fn: async () => {
        expect(percentileOf({
          sample: [],
          fraction: 0.9,
        },),).toBe(NO_SAMPLE,);
      },
    },),
  ],
},);

await describe({
  name: evidenceFromRunsFolder.name,
  children: [
    it({
      name: 'takes both limits from the finished runs, and leaves out the runs whose length is no evidence',
      fn: async () => {
        await using runs = await standInRunsFolder({
          runs: [
            {
              name: 'idle-1',
              summary: {
                endReason: 'idle',
                totalMilliseconds: MINUTE,
              },
              feedLines: [
                {
                  t: 0,
                  kind: 'tool_start',
                  tool: 'bash',
                },
                {
                  t: THIRTY_SECONDS,
                  kind: 'tool_end',
                  tool: 'bash',
                },
              ],
            },
            {
              name: 'wall-clock-1',
              summary: {
                endReason: 'wall-clock',
                totalMilliseconds: TWELVE_MINUTES,
              },
              feedLines: [
                {
                  t: 0,
                  kind: 'tool_start',
                  tool: 'bash',
                },
                {
                  t: TEN_MINUTES,
                  kind: 'tool_end',
                  tool: 'bash',
                },
              ],
            },
            {
              name: 'model-error-1',
              summary: {
                endReason: 'model-error',
                totalMilliseconds: 3 * MINUTE,
              },
            },
            {
              name: 'prompt-rejected-1',
              summary: {
                endReason: 'prompt-rejected',
                totalMilliseconds: 4 * MINUTE,
              },
            },
            {
              name: 'driver-failed-1',
              summary: {
                endReason: 'driver-failed',
                totalMilliseconds: 5 * MINUTE,
              },
            },
            {
              name: 'still-going-1',
              feedLines: [],
            },
            {
              name: 'no-total-1',
              summary: { endReason: 'idle', },
            },
            {
              name: 'a-plain-file',
              isAPlainFile: true,
            },
          ],
        },);
        /**
         What the folder says.
         */
        const evidence = await evidenceFromRunsFolder({ runsFolder: runs.path, },);
        expect(evidence.finishedRuns,).toBe(2,);
        expect(evidence.overdueMilliseconds,).toBe(rankOf([MINUTE, TWELVE_MINUTES,], 0.9,),);
        expect(evidence.toolCalls,).toBe(2,);
        expect(evidence.openCallMilliseconds,).toBe(rankOf([THIRTY_SECONDS, TEN_MINUTES,], 0.999,),);
      },
    },),
    it({
      name: 'leaves a run without a feed out of the call sample rather than counting its silence as calls',
      fn: async () => {
        await using runs = await standInRunsFolder({
          runs: [
            {
              name: 'no-feed-1',
              summary: {
                endReason: 'idle',
                totalMilliseconds: MINUTE,
              },
            },
          ],
        },);
        /**
         What the folder says.
         */
        const evidence = await evidenceFromRunsFolder({ runsFolder: runs.path, },);
        expect(evidence.finishedRuns,).toBe(1,);
        expect(evidence.toolCalls,).toBe(0,);
        expect(Object.keys(evidence,).includes('openCallMilliseconds',),).toBe(false,);
      },
    },),
    it({
      name: 'answers no limit at all for a runs folder holding no finished run',
      fn: async () => {
        await using runs = await standInRunsFolder({
          runs: [
            {
              name: 'still-going-1',
              feedLines: [],
            },
          ],
        },);
        /**
         What the folder says.
         */
        const evidence = await evidenceFromRunsFolder({ runsFolder: runs.path, },);
        expect(evidence,).toEqual({
          finishedRuns: 0,
          toolCalls: 0,
        },);
      },
    },),
  ],
},);
