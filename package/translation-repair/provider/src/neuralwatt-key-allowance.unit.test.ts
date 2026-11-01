/**
 Tests the reading of the per-key cap in a quota reply (`key.allowance`, the `KeyAllowance` schema of
 `https://docs.neuralwatt.com/api/quota`, read 2026-10-20) and the instant a blocked cap lets requests through.

 Fixtures are cat-themed invention.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  blockedUntilOf,
  keyAllowanceOf,
} from '../dist/final/node/index.mjs';

await describe({
  name: 'NeuralWatt key allowance',
  concurrency: 1,
  children: [
    describe({
      name: keyAllowanceOf.name,
      children: [
        it({
          name: 'READS NO CAP, and no problem, from a key with none, however the reply says so',
          fn: async () => {
            expect(keyAllowanceOf({ key: undefined, },),).toEqual({},);
            expect(keyAllowanceOf({ key: null, },),).toEqual({},);
            expect(keyAllowanceOf({ key: { allowance: null, }, },),).toEqual({},);
            expect(keyAllowanceOf({ key: {}, },),).toEqual({},);
          },
        },),
        it({
          name: 'READS WHAT IS LEFT, WHETHER IT IS BLOCKED, and the reset spelled as toISOString spells it',
          fn: async () => {
            expect(keyAllowanceOf({
              key: {
                allowance: {
                  limit_usd: 5,
                  period: 'daily',
                  spent_usd: 1.75,
                  remaining_usd: 3.25,
                  blocked: false,
                  reset_at: '2026-10-21T19:15:08+00:00',
                },
              },
            },),).toEqual({
              allowance: {
                remainingUsd: 3.25,
                blocked: false,
                resetAt: '2026-10-21T19:15:08.000Z',
              },
            },);
            expect(keyAllowanceOf({
              key: {
                allowance: {
                  remaining_usd: 0,
                  blocked: true,
                  reset_at: null,
                },
              },
            },),).toEqual({
              allowance: {
                remainingUsd: 0,
                blocked: true,
              },
            },);
          },
        },),
        it({
          name: 'NAMES WHAT WILL NOT READ AND BELIEVES NONE OF IT: a key or cap that is no object, an amount that is '
            + 'negative or text, a blocked that is no boolean, a period that is no text, a reset that is no instant',
          fn: async () => {
            /**
             Each broken reply and the phrase naming what is wrong with it.
             */
            const broken: readonly (readonly [unknown, string])[] = [
              ['cat', 'key is neither an object nor null',],
              [{ allowance: 'cat', }, 'key.allowance is neither an object nor null',],
              [{ allowance: { remaining_usd: -1, blocked: false, }, }, 'key.allowance.remaining_usd is not a non-negative number',],
              [{ allowance: { remaining_usd: '3', blocked: false, }, }, 'key.allowance.remaining_usd is not a non-negative number',],
              [{ allowance: { remaining_usd: 3, blocked: 'no', }, }, 'key.allowance.blocked is not a boolean',],
              [{ allowance: { remaining_usd: 3, blocked: false, limit_usd: 'x', }, }, 'key.allowance.limit_usd is not a non-negative number',],
              [{ allowance: { remaining_usd: 3, blocked: false, spent_usd: -2, }, }, 'key.allowance.spent_usd is not a non-negative number',],
              [{ allowance: { remaining_usd: 3, blocked: false, period: 7, }, }, 'key.allowance.period is not a string',],
              [{ allowance: { remaining_usd: 3, blocked: false, reset_at: 'soon', }, }, 'key.allowance.reset_at is neither an ISO 8601 instant nor null',],
              [{ allowance: { remaining_usd: 3, blocked: false, reset_at: 7, }, }, 'key.allowance.reset_at is neither an ISO 8601 instant nor null',],
            ];
            for (const [key, problem,] of broken)
              expect(keyAllowanceOf({ key, },),).toEqual({ problem, },);
          },
        },),
      ],
    },),
    describe({
      name: blockedUntilOf.name,
      children: [
        it({
          name: 'LETS REQUESTS THROUGH AT THE CAP\'S OWN RESET, else at the next day\'s anchor, a day after this one\'s',
          fn: async () => {
            /**
             First instant of the current day, 15:15 EDT in America/New_York being 19:15 UTC.
             */
            const start = new Date('2026-10-20T19:15:08.000Z',);
            expect(blockedUntilOf({
              allowance: {
                remainingUsd: 0,
                blocked: true,
                resetAt: '2026-10-20T23:00:00.000Z',
              },
              start,
            },).toISOString(),).toBe('2026-10-20T23:00:00.000Z',);
            expect(blockedUntilOf({
              allowance: {
                remainingUsd: 0,
                blocked: true,
              },
              start,
            },).toISOString(),).toBe('2026-10-21T19:15:08.000Z',);
          },
        },),
      ],
    },),
  ],
},);
