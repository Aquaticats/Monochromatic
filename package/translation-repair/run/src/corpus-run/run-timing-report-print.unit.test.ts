/**
 Tests for what the run timing report prints, one printer at a time.

 EACH PRINTER IS HANDED THE READING IT PRINTS, so a case states the whole
 reading and asserts the whole text it produced. The span is checked in each of
 its three units, since the same report reads a six-hour pass and a
 thirty-second probe and a probe printed in hours reads as a run that did
 nothing.

 THE ROUND TIME SHARE IS CHECKED AT ZERO. Rounds that all took no time leave
 nothing to divide the grace by, and a percentage of nothing reads as a
 measurement.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  asSpan,
  type InFlight,
  printInFlight,
  printRounds,
  type RoundTiming,
  type RunTiming,
} from '../../dist/final/node/index.mjs';
import { divertingConsoleLog, } from '@monochromatic-dev/translation-repair-test-fixture/ts/corpus-run/console-log-capture.test-fixture.ts';

/**
 A round whose quorum stood, with thirty seconds spent in grace.
 */
const STOOD_ROUND: RoundTiming = {
  stage: 'editor',
  heard: 6,
  asked: 7,
  totalMs: 91_402,
  quorum: {
    kind: 'stood',
    toQuorumMs: 61_401,
    inGraceMs: 30_001,
  },
};

/**
 A round whose quorum never stood.
 */
const NEVER_ROUND: RoundTiming = {
  stage: 'editor',
  heard: 1,
  asked: 7,
  totalMs: 120_000,
  quorum: {
    kind: 'never',
    needed: 4,
  },
};

/**
 A round that took no time.
 */
const INSTANT_ROUND: RoundTiming = {
  stage: 'editor',
  heard: 3,
  asked: 3,
  totalMs: 0,
  quorum: {
    kind: 'stood',
    toQuorumMs: 0,
    inGraceMs: 0,
  },
};

/**
 Timing a log read when it held the given rounds and no call.

 @param rounds - rounds the log reported

 @returns Reading

 @example
 ```ts
 const reading = readingOf({ rounds: [STOOD_ROUND,], },);
 ```
 */
function readingOf({ rounds, }: { readonly rounds: readonly RoundTiming[]; },): RunTiming {
  return {
    rounds,
    calls: [],
    callsWithoutDuration: 0,
    callsWithoutStamp: 0,
  };
}

await describe({
  name: 'run timing report printers',
  concurrency: 1,
  children: [
    describe({
      name: asSpan.name,
      concurrency: 1,
      children: [
        it({
          name: 'RENDERS a span of an hour or more in hours',
          fn: async () => {
            expect(asSpan({ ms: 22_140_000, },),).toBe('6.15h',);
            expect(asSpan({ ms: 3_600_000, },),).toBe('1.00h',);
          },
        },),

        it({
          name: 'RENDERS a span of a minute up to an hour in minutes',
          fn: async () => {
            expect(asSpan({ ms: 60_000, },),).toBe('1.00min',);
            expect(asSpan({ ms: 90_000, },),).toBe('1.50min',);
            expect(asSpan({ ms: 3_540_000, },),).toBe('59.00min',);
          },
        },),

        it({
          name: 'RENDERS a span under a minute in seconds, and none as no seconds',
          fn: async () => {
            expect(asSpan({ ms: 59_000, },),).toBe('59.00s',);
            expect(asSpan({ ms: 30_001, },),).toBe('30.00s',);
            expect(asSpan({ ms: 0, },),).toBe('0.00s',);
          },
        },),
      ],
    },),

    describe({
      name: printRounds.name,
      concurrency: 1,
      children: [
        it({
          name: 'SAYS NO ROUND LINE and that it is not the same as a run that never waited when the log held none',
          fn: async (ctx) => {
            using printed = divertingConsoleLog({ sinon: ctx.sinon, },);
            printRounds({ reading: readingOf({ rounds: [], },), },);
            expect(printed.lines,).toEqual([
              'NO ROUND LINE. This log predates round lines and call durations, so how long each fan-out took and '
                + 'how much of that was spent waiting after quorum are both unrecorded. That is not the same as a '
                + 'run that never waited.',
            ],);
          },
        },),

        it({
          name: 'PRINTS the rounds counted, their time, the share spent waiting and the voices never heard',
          fn: async (ctx) => {
            using printed = divertingConsoleLog({ sinon: ctx.sinon, },);
            printRounds({
              reading: readingOf({
                rounds: [
                  STOOD_ROUND,
                  NEVER_ROUND,
                ],
              },),
            },);
            expect(printed.lines,).toEqual([
              'rounds                 2, 3.52min in total',
              '  without quorum       1',
              '  waiting after quorum 30.00s, 14.2% of round time',
              '  voices never heard   7',
            ],);
          },
        },),

        it({
          name: 'PRINTS one round, whose quorum stood, with no round that lacked quorum',
          fn: async (ctx) => {
            using printed = divertingConsoleLog({ sinon: ctx.sinon, },);
            printRounds({ reading: readingOf({ rounds: [STOOD_ROUND,], },), },);
            expect(printed.lines,).toEqual([
              'rounds                 1, 1.52min in total',
              '  without quorum       0',
              '  waiting after quorum 30.00s, 32.8% of round time',
              '  voices never heard   1',
            ],);
          },
        },),

        it({
          name: 'SAYS there is no round time to share when every round took no time, instead of a percentage',
          fn: async (ctx) => {
            using printed = divertingConsoleLog({ sinon: ctx.sinon, },);
            printRounds({ reading: readingOf({ rounds: [INSTANT_ROUND,], },), },);
            expect(printed.lines,).toEqual([
              'rounds                 1, 0.00s in total',
              '  without quorum       0',
              '  waiting after quorum 0.00s, no round time to share',
              '  voices never heard   0',
            ],);
          },
        },),
      ],
    },),

    describe({
      name: printInFlight.name,
      concurrency: 1,
      children: [
        it({
          name: 'PRINTS the mean and peak in flight and the busy time against the span of the run',
          fn: async (ctx) => {
            using printed = divertingConsoleLog({ sinon: ctx.sinon, },);
            /**
             Two calls overlapping for ten of fifteen seconds.
             */
            const flight: InFlight = {
              spanMs: 15_000,
              busyMs: 20_000,
              meanInFlight: 20 / 15,
              peakInFlight: 2,
            };
            printInFlight({ flight, },);
            expect(printed.lines,).toEqual([
              'calls in flight        mean 1.33, peak 2',
              '  busy against span    20.00s of calls across 15.00s of run',
            ],);
          },
        },),

        it({
          name: 'PRINTS a long run\'s busy time and span in the unit each fills',
          fn: async (ctx) => {
            using printed = divertingConsoleLog({ sinon: ctx.sinon, },);
            printInFlight({
              flight: {
                spanMs: 7_200_000,
                busyMs: 90_000,
                meanInFlight: 0.012,
                peakInFlight: 1,
              },
            },);
            expect(printed.lines,).toEqual([
              'calls in flight        mean 0.01, peak 1',
              '  busy against span    1.50min of calls across 2.00h of run',
            ],);
          },
        },),
      ],
    },),
  ],
},);
