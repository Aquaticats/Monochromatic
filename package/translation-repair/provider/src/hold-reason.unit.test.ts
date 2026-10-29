/**
 Tests for the words a hold wait and a refusal give: which providers the shortest hold belongs to, and why NeuralWatt
 reads dry, with the time of day its day starts again for the operator to relaunch by.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  dryCauseClausesOf,
  holdReasonOf,
} from '../dist/final/node/index.mjs';

/**
 Nobody held.
 */
const NO_HOLDS = {
  neuralwatt: 0,
  synthetic: 0,
  bedrock: 0,
  deepseek: 0,
  xiaomi: 0,
  anthropic: 0,
  hyper: 0,
  openrouter: 0,
} as const;

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: holdReasonOf.name,
      concurrency: 1,
      children: [
        it({
          name: 'NAMES NEURALWATT AS A PLAIN NAME when a refusal holds it for the shortest hold, with no word of its '
            + 'day start: a 60 s rate-limit hold is a rate limit, whatever time the day starts',
          fn: async () => {
            expect(holdReasonOf({
              holds: { ...NO_HOLDS, neuralwatt: 60_000, },
              shortest: 60_000,
            },),).toBe('neuralwatt',);
          },
        },),

        it({
          name: 'NAMES THE PROVIDERS THAT HOLD THE SHORTEST HOLD in spending order, and the refusals they stand for as '
            + 'plain names',
          fn: async () => {
            expect(holdReasonOf({
              holds: { ...NO_HOLDS, synthetic: 5_000, hyper: 5_000, openrouter: 9_000, },
              shortest: 5_000,
            },),).toBe('synthetic, hyper',);
          },
        },),

        it({
          name: 'NAMES NOBODY where nothing is held',
          fn: async () => {
            expect(holdReasonOf({ holds: NO_HOLDS, shortest: 0, },),).toBe('no provider',);
          },
        },),
      ],
    },),

    describe({
      name: dryCauseClausesOf.name,
      concurrency: 1,
      children: [
        it({
          name: 'STATES THE KEY\'S OWN TIME for a blocked key, the calls in flight for held energy, and the learned '
            + 'day start for an unanchored run: only the true cause',
          fn: async () => {
            expect(dryCauseClausesOf({
              dryness: { kind: 'key-blocked', until: '2026-10-23T05:00:00.000Z', },
            },),).toEqual([
              'NeuralWatt reads dry because its key allowance is blocked until 2026-10-23T05:00:00.000Z',
            ],);
            expect(dryCauseClausesOf({ dryness: { kind: 'reserved', }, },),).toEqual([
              'NeuralWatt reads dry because calls in flight hold the energy the settled calls leave',
            ],);
            expect(dryCauseClausesOf({ dryness: { kind: 'no-day-start', }, },),).toEqual([
              'NeuralWatt reads dry because this run learned no day start for it',
            ],);
          },
        },),

        it({
          name: 'STATES THE DAY START AND THE DISTANCE for a spent day with a return, and that the account still shows '
            + 'the day spent for one without',
          fn: async () => {
            expect(dryCauseClausesOf({
              dryness: { kind: 'spent-day', dayStart: '2026-10-22T19:15:08.000Z', returnsInMs: 301_000, },
            },),).toEqual(['NeuralWatt reads dry and its day starts again at 19:15:08 UTC (in 5 min 1 s)',],);
            expect(dryCauseClausesOf({
              dryness: { kind: 'spent-day', dayStart: '2026-10-22T19:15:08.000Z', },
            },),).toEqual([
              'NeuralWatt reads dry because its account still shows the day spent past the day start at 19:15:08 UTC',
            ],);
          },
        },),

        it({
          name: 'STATES THE SAME TIME OF DAY AND DISTANCE AT ANY DISTANCE and says nothing of a bound, since nothing '
            + 'waits for the return: an hour away reads as 5 minutes away does',
          fn: async () => {
            /**
             A spent day an hour from its return.
             */
            const dryness = {
              kind: 'spent-day',
              dayStart: '2026-10-22T19:15:08.000Z',
              returnsInMs: 3_600_000,
            } as const;
            expect(dryCauseClausesOf({ dryness, },),).toEqual([
              'NeuralWatt reads dry and its day starts again at 19:15:08 UTC (in 1 h)',
            ],);
          },
        },),

        it({
          name: 'SAYS NOTHING where NeuralWatt is not dry',
          fn: async () => {
            expect(dryCauseClausesOf({ dryness: { kind: 'wet', }, },),).toEqual([],);
          },
        },),
      ],
    },),
  ],
},);
