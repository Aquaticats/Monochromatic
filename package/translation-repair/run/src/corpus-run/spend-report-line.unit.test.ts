/**
 Tests for the lines of the spend report that stand for one seat.

 EACH LINE IS HANDED THE SEAT IT PRINTS, so a case states the whole seat and
 asserts the whole line. A count noun is shown at one and at several, the
 share of a bill at nothing billed and at some, and the note of a floor or a
 reckoning both present and absent, since each is an arm of its own.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  asCredits,
  asUsd,
  pricedLine,
  tokensOnlyLine,
  usdLine,
} from '../../dist/final/node/index.mjs';
import {
  pricedSeatOf,
  seatOf,
} from './spend-seat.test-fixture.ts';

await describe({
  name: 'spend report lines',
  concurrency: 1,
  children: [
    describe({
      name: asCredits.name,
      concurrency: 1,
      children: [
        it({
          name: 'RENDERS credits to two places',
          fn: async () => {
            expect(asCredits({ credits: 9.5, },),).toBe('9.50',);
            expect(asCredits({ credits: 0, },),).toBe('0.00',);
          },
        },),
      ],
    },),

    describe({
      name: asUsd.name,
      concurrency: 1,
      children: [
        it({
          name: 'RENDERS USD to four places, since a call costs tenths of a cent',
          fn: async () => {
            expect(asUsd({ usd: 0.0842, },),).toBe('0.0842',);
            expect(asUsd({ usd: 0.00015646, },),).toBe('0.0002',);
          },
        },),
      ],
    },),

    describe({
      name: pricedLine.name,
      concurrency: 1,
      children: [
        it({
          name: 'PRINTS a priced seat with its share of the bill, its calls, and the credits of each half',
          fn: async () => {
            expect(pricedLine({
              seat: pricedSeatOf({
                inputCredits: 40,
                outputCredits: 60,
                overrides: {
                  calls: 3,
                  promptTokens: 1_000_000,
                  completionTokens: 500_000,
                },
              },),
              totalCredits: 400,
            },),).toBe(
              '  qwen3.8-max: 100.00 credits (25.0%) over 3 calls, in 1000000=40.00 out 500000=60.00',
            );
          },
        },),

        it({
          name: 'PRINTS the singular of a call and the note of the calls reckoned rather than reported',
          fn: async () => {
            expect(pricedLine({
              seat: pricedSeatOf({
                inputCredits: 1,
                outputCredits: 1,
                overrides: {
                  calls: 1,
                  reckonedCalls: 1,
                },
              },),
              totalCredits: 2,
            },),).toBe(
              '  qwen3.8-max: 2.00 credits (100.0%) over 1 call, in 1000=1.00 out 500=1.00, '
                + '1 of them reckoned rather than reported',
            );
          },
        },),

        it({
          name: 'PRINTS n/a for the share when nothing was billed at all',
          fn: async () => {
            expect(pricedLine({
              seat: pricedSeatOf({
                inputCredits: 0,
                outputCredits: 0,
                overrides: {
                  promptTokens: 0,
                  completionTokens: 0,
                },
              },),
              totalCredits: 0,
            },),).toBe(
              '  qwen3.8-max: 0.00 credits (n/a) over 1 call, in 0=0.00 out 0=0.00',
            );
          },
        },),
      ],
    },),

    describe({
      name: usdLine.name,
      concurrency: 1,
      children: [
        it({
          name: 'PRINTS a USD seat with its share of the USD billed and no floor when every call was costed',
          fn: async () => {
            expect(usdLine({
              seat: seatOf({
                overrides: {
                  provider: 'openrouter',
                  model: 'minimax/minimax-m3',
                  calls: 2,
                  costedCalls: 2,
                  costUsd: 0.25,
                  promptTokens: 15,
                  completionTokens: 25,
                },
              },),
              totalUsd: 1,
            },),).toBe(
              '  minimax/minimax-m3: 0.2500 USD (25.0%) over 2 calls, in 15 out 25',
            );
          },
        },),

        it({
          name: 'PRINTS the floor in the singular when one call carried no cost, and the reckoned calls',
          fn: async () => {
            expect(usdLine({
              seat: seatOf({
                overrides: {
                  provider: 'openrouter',
                  model: 'minimax/minimax-m3',
                  calls: 2,
                  costedCalls: 1,
                  costUsd: 0.0625,
                  promptTokens: 15,
                  completionTokens: 25,
                  reckonedCalls: 1,
                },
              },),
              totalUsd: 0.0625,
            },),).toBe(
              '  minimax/minimax-m3: 0.0625 USD (100.0%) over 2 calls, in 15 out 25, '
                + 'a floor: 1 call carried no cost, 1 of them reckoned rather than reported',
            );
          },
        },),

        it({
          name: 'PRINTS the floor in the plural and n/a for the share when no call carried a cost',
          fn: async () => {
            expect(usdLine({
              seat: seatOf({
                overrides: {
                  provider: 'bedrock',
                  model: 'google.gemma-4-e2b',
                  calls: 3,
                  costedCalls: 0,
                  costUsd: 0,
                  promptTokens: 7,
                  completionTokens: 3,
                },
              },),
              totalUsd: 0,
            },),).toBe(
              '  google.gemma-4-e2b: 0.0000 USD (n/a) over 3 calls, in 7 out 3, a floor: 3 calls carried no cost',
            );
          },
        },),
      ],
    },),

    describe({
      name: tokensOnlyLine.name,
      concurrency: 1,
      children: [
        it({
          name: 'PRINTS a seat with tokens and no credits, in the singular',
          fn: async () => {
            expect(tokensOnlyLine({
              seat: seatOf({
                overrides: {
                  provider: 'synthetic',
                  model: 'hf:zai-org/GLM-5.3-Flash',
                  promptTokens: 300,
                  completionTokens: 40,
                },
              },),
            },),).toBe(
              '  hf:zai-org/GLM-5.3-Flash: 1 call, in 300 out 40',
            );
          },
        },),

        it({
          name: 'PRINTS the plural of a call and the note of the calls reckoned rather than reported',
          fn: async () => {
            expect(tokensOnlyLine({
              seat: seatOf({
                overrides: {
                  provider: 'synthetic',
                  model: 'hf:zai-org/GLM-5.3-Flash',
                  calls: 4,
                  reckonedCalls: 2,
                  promptTokens: 300,
                  completionTokens: 40,
                },
              },),
            },),).toBe(
              '  hf:zai-org/GLM-5.3-Flash: 4 calls, in 300 out 40, 2 of them reckoned rather than reported',
            );
          },
        },),
      ],
    },),
  ],
},);
