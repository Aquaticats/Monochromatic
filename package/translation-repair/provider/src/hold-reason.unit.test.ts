/**
 Tests for the words a hold wait gives for waiting: which providers the shortest hold belongs to, and for NeuralWatt the
 time of day its day starts again.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';
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

/**
 The account whose day starts at 19:15:08 UTC.
 */
const XINGZ_DAY = { kind: 'at', msOfDay: 69_308_000, } as const;

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: holdReasonOf.name,
      concurrency: 1,
      children: [
        it({
          name: 'NAMES NEURALWATT\'S DAY START when NeuralWatt holds the shortest hold, since that hold is its return and '
            + 'not a refusal',
          fn: async () => {
            expect(holdReasonOf({
              holds: { ...NO_HOLDS, neuralwatt: 48_000, hyper: 900_000, },
              shortest: 48_000,
              neuralwatt: XINGZ_DAY,
            },),).toBe('neuralwatt, whose day starts again at 19:15:08 UTC',);
          },
        },),

        it({
          name: 'NAMES NO DAY START when the hold that won on NeuralWatt is a refusal\'s and not its return: a 60 s '
            + 'rate-limit hold is a rate limit, whatever time the day starts',
          fn: async () => {
            expect(holdReasonOf({
              holds: { ...NO_HOLDS, neuralwatt: 60_000, },
              shortest: 60_000,
              neuralwatt: XINGZ_DAY,
              neuralwattHeldByReturn: false,
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
              neuralwatt: XINGZ_DAY,
            },),).toBe('synthetic, hyper',);
          },
        },),

        it({
          name: 'NAMES NOBODY where nothing is held',
          fn: async () => {
            expect(holdReasonOf({ holds: NO_HOLDS, shortest: 0, neuralwatt: XINGZ_DAY, },),).toBe('no provider',);
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
          name: 'SAYS THE RETURN IS PAST WHAT THE PASS HAS LEFT only when it is, and adds nothing for a '
            + 'return inside it',
          fn: async () => {
            /**
             A spent day 700 s from its return.
             */
            const dryness = { kind: 'spent-day', dayStart: '2026-10-22T19:15:08.000Z', returnsInMs: 700_000, } as const;
            expect(dryCauseClausesOf({ dryness, passLeftMs: 600_000, },),).toEqual([
              'NeuralWatt reads dry and its day starts again at 19:15:08 UTC (in 11 min 40 s), which is past '
                + 'the 10 min '
                + 'this pass has left',
            ],);
            expect(dryCauseClausesOf({ dryness, passLeftMs: 700_000, },),).toEqual([
              'NeuralWatt reads dry and its day starts again at 19:15:08 UTC (in 11 min 40 s)',
            ],);
          },
        },),

        it({
          name: 'NAMES THE BOUND THAT WON in the clause: the entry when its own cap is the smaller bound',
          fn: async () => {
            /**
             A spent day an hour from its return.
             */
            const dryness = {
              kind: 'spent-day',
              dayStart: '2026-10-22T19:15:08.000Z',
              returnsInMs: 3_600_000,
            } as const;
            expect(dryCauseClausesOf({ dryness, passLeftMs: 1_200_000, boundOwner: 'entry', },),).toEqual([
              'NeuralWatt reads dry and its day starts again at 19:15:08 UTC (in 1 h), which is past the 20 min '
                + 'the entry has left',
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
