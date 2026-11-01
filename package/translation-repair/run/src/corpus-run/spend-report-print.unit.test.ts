/**
 Tests for what the spend report says about a priced tally.

 EACH SECTION IS CHECKED PRESENT AND ABSENT. The metered seats, the seats no
 price covered, the USD seats, the subscription seats and the floor of calls
 that reported no usage each print only when the tally holds them, so a case
 hands `printCost` a tally holding one section and asserts the whole text.

 A SECTION NAMES WHAT IT HOLDS. The USD seats are OpenRouter's and Bedrock's,
 and the heading and the total name the providers the seats came from, since a
 Bedrock seat printed under OpenRouter's name sends a reader to the wrong
 invoice. A report of no priced seat says why: no metered call at all, or
 metered calls on seats no row of the price table covers.

 EVERY COUNT NOUN, VERB AND PRONOUN AGREES WITH ITS COUNT, shown at one and at
 several.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  printCost,
  type SpendCost,
} from '../../dist/final/node/index.mjs';
import { divertingConsoleLog, } from '@monochromatic-dev/translation-repair-test-fixture/ts/corpus-run/console-log-capture.test-fixture.ts';
import {
  pricedSeatOf,
  seatOf,
} from './spend-seat.test-fixture.ts';

/**
 What a tally of nothing prices to.
 */
const NOTHING: SpendCost = {
  priced: [],
  unpriced: [],
  subscription: [],
  freeEnergy: [],
  openRouter: [],
  totalCredits: 0,
  totalUsd: 0,
  unreportedCalls: 0,
  pricedAsOf: '2026-09-11',
};

await describe({
  name: printCost.name,
  concurrency: 1,
  children: [
    it({
      name: 'SAYS no call went to the metered provider and totals no credits for a tally of nothing',
      fn: async (ctx) => {
        using printed = divertingConsoleLog({ sinon: ctx.sinon, },);
        printCost({ cost: NOTHING, },);
        expect(printed.lines,).toEqual([
          'metered seats, priced at rates read 2026-09-11:',
          '  none. No call in these logs went to the metered provider',
          'metered run total: 0.00 credits',
        ],);
      },
    },),

    it({
      name: 'PRINTS every priced seat with its share and the total of credits',
      fn: async (ctx) => {
        using printed = divertingConsoleLog({ sinon: ctx.sinon, },);
        printCost({
          cost: {
            ...NOTHING,
            priced: [
              pricedSeatOf({
                inputCredits: 40,
                outputCredits: 60,
                overrides: {
                  calls: 2,
                  promptTokens: 1_000_000,
                  completionTokens: 500_000,
                },
              },),
              pricedSeatOf({
                inputCredits: 1,
                outputCredits: 1,
                overrides: {
                  model: 'gemma-4-26b-a4b-it',
                  promptTokens: 400_000,
                  completionTokens: 100_000,
                },
              },),
            ],
            totalCredits: 102,
          },
        },);
        expect(printed.lines,).toEqual([
          'metered seats, priced at rates read 2026-09-11:',
          '  qwen3.8-max: 100.00 credits (98.0%) over 2 calls, in 1000000=40.00 out 500000=60.00',
          '  gemma-4-26b-a4b-it: 2.00 credits (2.0%) over 1 call, in 400000=1.00 out 100000=1.00',
          'metered run total: 102.00 credits',
        ],);
      },
    },),

    it({
      name: 'NAMES one seat the price table has no row for, in the singular, and prints its tokens',
      fn: async (ctx) => {
        using printed = divertingConsoleLog({ sinon: ctx.sinon, },);
        printCost({
          cost: {
            ...NOTHING,
            priced: [
              pricedSeatOf({
                inputCredits: 1,
                outputCredits: 1,
                overrides: {},
              },),
            ],
            unpriced: [
              seatOf({
                overrides: {
                  model: 'whisker-mini-9',
                  promptTokens: 10,
                  completionTokens: 20,
                },
              },),
            ],
            totalCredits: 2,
          },
        },);
        expect(printed.lines,).toEqual([
          'metered seats, priced at rates read 2026-09-11:',
          '  qwen3.8-max: 2.00 credits (100.0%) over 1 call, in 1000=1.00 out 500=1.00',
          'metered run total: 2.00 credits',
          'UNPRICED, and not free: 1 metered seat has no row in the price table read 2026-09-11. '
            + 'This report\'s total omits whatever cost this seat would add',
          '  whisker-mini-9: 1 call, in 10 out 20',
        ],);
      },
    },),

    it({
      name: 'NAMES several seats the price table has no row for, in the plural, and prints each one\'s tokens',
      fn: async (ctx) => {
        using printed = divertingConsoleLog({ sinon: ctx.sinon, },);
        printCost({
          cost: {
            ...NOTHING,
            priced: [
              pricedSeatOf({
                inputCredits: 1,
                outputCredits: 1,
                overrides: {},
              },),
            ],
            unpriced: [
              seatOf({
                overrides: {
                  model: 'whisker-mini-9',
                  promptTokens: 10,
                  completionTokens: 20,
                },
              },),
              seatOf({
                overrides: {
                  model: 'whisker-mini-8',
                  calls: 3,
                  promptTokens: 1,
                  completionTokens: 2,
                },
              },),
            ],
            totalCredits: 2,
          },
        },);
        expect(printed.lines,).toEqual([
          'metered seats, priced at rates read 2026-09-11:',
          '  qwen3.8-max: 2.00 credits (100.0%) over 1 call, in 1000=1.00 out 500=1.00',
          'metered run total: 2.00 credits',
          'UNPRICED, and not free: 2 metered seats have no row in the price table read 2026-09-11. '
            + 'This report\'s total omits whatever cost these seats would add',
          '  whisker-mini-9: 1 call, in 10 out 20',
          '  whisker-mini-8: 3 calls, in 1 out 2',
        ],);
      },
    },),

    it({
      name: 'SAYS every metered call was on a seat the price table has no row for, not that none was metered',
      fn: async (ctx) => {
        using printed = divertingConsoleLog({ sinon: ctx.sinon, },);
        printCost({
          cost: {
            ...NOTHING,
            unpriced: [
              seatOf({
                overrides: {
                  model: 'whisker-mini-9',
                  promptTokens: 10,
                  completionTokens: 20,
                },
              },),
            ],
          },
        },);
        expect(printed.lines,).toEqual([
          'metered seats, priced at rates read 2026-09-11:',
          '  none priced: every metered call in these logs was on a seat the price table has no row for',
          'metered run total: 0.00 credits',
          'UNPRICED, and not free: 1 metered seat has no row in the price table read 2026-09-11. '
            + 'This report\'s total omits whatever cost this seat would add',
          '  whisker-mini-9: 1 call, in 10 out 20',
        ],);
      },
    },),

    it({
      name: 'PRINTS the OpenRouter seats with their USD, a share of it, and the total never summed with credits',
      fn: async (ctx) => {
        using printed = divertingConsoleLog({ sinon: ctx.sinon, },);
        printCost({
          cost: {
            ...NOTHING,
            openRouter: [
              seatOf({
                overrides: {
                  provider: 'openrouter',
                  model: 'minimax/minimax-m3',
                  calls: 2,
                  costedCalls: 2,
                  costUsd: 0.0625,
                  promptTokens: 15,
                  completionTokens: 25,
                },
              },),
            ],
            totalUsd: 0.0625,
          },
        },);
        expect(printed.lines,).toEqual([
          'metered seats, priced at rates read 2026-09-11:',
          '  none. No call in these logs went to the metered provider',
          'metered run total: 0.00 credits',
          'OpenRouter seats, billed in USD per token, each priced from the cost= its own lines carried:',
          '  minimax/minimax-m3: 0.0625 USD (100.0%) over 2 calls, in 15 out 25',
          'OpenRouter run total: 0.0625 USD, never summed with this report\'s credits',
        ],);
      },
    },),

    it({
      name: 'NAMES Bedrock, not OpenRouter, over USD seats that are all Bedrock\'s',
      fn: async (ctx) => {
        using printed = divertingConsoleLog({ sinon: ctx.sinon, },);
        printCost({
          cost: {
            ...NOTHING,
            openRouter: [
              seatOf({
                overrides: {
                  provider: 'bedrock',
                  model: 'google.gemma-4-e2b',
                  costedCalls: 1,
                  costUsd: 0.5,
                  promptTokens: 7,
                  completionTokens: 3,
                },
              },),
            ],
            totalUsd: 0.5,
          },
        },);
        expect(printed.lines,).toEqual([
          'metered seats, priced at rates read 2026-09-11:',
          '  none. No call in these logs went to the metered provider',
          'metered run total: 0.00 credits',
          'Bedrock seats, billed in USD per token, each priced from the cost= its own lines carried:',
          '  google.gemma-4-e2b: 0.5000 USD (100.0%) over 1 call, in 7 out 3',
          'Bedrock run total: 0.5000 USD, never summed with this report\'s credits',
        ],);
      },
    },),

    it({
      name: 'NAMES both providers over USD seats from OpenRouter and from Bedrock, and totals them as one figure',
      fn: async (ctx) => {
        using printed = divertingConsoleLog({ sinon: ctx.sinon, },);
        printCost({
          cost: {
            ...NOTHING,
            openRouter: [
              seatOf({
                overrides: {
                  provider: 'bedrock',
                  model: 'google.gemma-4-e2b',
                  costedCalls: 1,
                  costUsd: 0.5,
                  promptTokens: 7,
                  completionTokens: 3,
                },
              },),
              seatOf({
                overrides: {
                  provider: 'openrouter',
                  model: 'minimax/minimax-m3',
                  costedCalls: 1,
                  costUsd: 0.0625,
                  promptTokens: 15,
                  completionTokens: 25,
                },
              },),
            ],
            totalUsd: 0.5625,
          },
        },);
        expect(printed.lines,).toEqual([
          'metered seats, priced at rates read 2026-09-11:',
          '  none. No call in these logs went to the metered provider',
          'metered run total: 0.00 credits',
          'OpenRouter and Bedrock seats, billed in USD per token, each priced from the cost= its own lines carried:',
          '  google.gemma-4-e2b: 0.5000 USD (88.9%) over 1 call, in 7 out 3',
          '  minimax/minimax-m3: 0.0625 USD (11.1%) over 1 call, in 15 out 25',
          'OpenRouter and Bedrock run total: 0.5625 USD, never summed with this report\'s credits',
        ],);
      },
    },),

    it({
      name: 'PRINTS the subscription seats, which bill no credits, with their tokens only',
      fn: async (ctx) => {
        using printed = divertingConsoleLog({ sinon: ctx.sinon, },);
        printCost({
          cost: {
            ...NOTHING,
            subscription: [
              seatOf({
                overrides: {
                  provider: 'synthetic',
                  model: 'hf:zai-org/GLM-5.3-Flash',
                  promptTokens: 300,
                  completionTokens: 40,
                },
              },),
            ],
          },
        },);
        expect(printed.lines,).toEqual([
          'metered seats, priced at rates read 2026-09-11:',
          '  none. No call in these logs went to the metered provider',
          'metered run total: 0.00 credits',
          'subscription seats, which bill no credits and are metered as a percentage of a weekly allowance on the '
            + 'METERS line:',
          '  hf:zai-org/GLM-5.3-Flash: 1 call, in 300 out 40',
        ],);
      },
    },),

    it({
      name: 'PRINTS the NeuralWatt seats as free daily energy metered in watt-hours, with their tokens only and '
        + 'no cost line while no call reported a cost',
      fn: async (ctx) => {
        using printed = divertingConsoleLog({ sinon: ctx.sinon, },);
        printCost({
          cost: {
            ...NOTHING,
            freeEnergy: [
              seatOf({
                overrides: {
                  provider: 'neuralwatt',
                  model: 'deepseek-v4.1-flash',
                  promptTokens: 3_000,
                  completionTokens: 400,
                },
              },),
            ],
          },
        },);
        expect(printed.lines,).toEqual([
          'metered seats, priced at rates read 2026-09-11:',
          '  none. No call in these logs went to the metered provider',
          'metered run total: 0.00 credits',
          'NeuralWatt seats, on the free daily energy, which bills no credits and is metered in watt-hours by the '
            + 'energy ledger (neuralwattWh on the METERS line):',
          '  deepseek-v4.1-flash: 1 call, in 3000 out 400',
        ],);
      },
    },),

    it({
      name: 'SAYS THE NOMINAL COST NEURALWATT REPORTED IS NOT PLAN SPEND, with the USD its lines reported, since the '
        + 'provider reports a cost on free-energy requests too and the provider stays wet',
      fn: async (ctx) => {
        using printed = divertingConsoleLog({ sinon: ctx.sinon, },);
        printCost({
          cost: {
            ...NOTHING,
            freeEnergy: [
              seatOf({
                overrides: {
                  provider: 'neuralwatt',
                  model: 'glm-5.3-flash',
                  calls: 2,
                  costUsd: 0.0004,
                  costedCalls: 2,
                },
              },),
            ],
          },
        },);
        expect(printed.lines.at(-1,),).toBe(
          'NeuralWatt nominal cost: 0.0004 USD reported across the calls of these logs; NeuralWatt reports a cost on '
            + 'free-energy requests too, so this is not plan spend, which only the plan energy line at the end of a run '
            + 'measures',
        );
      },
    },),

    it({
      name: 'SAYS one call reported no usage block, with its pronoun in the singular',
      fn: async (ctx) => {
        using printed = divertingConsoleLog({ sinon: ctx.sinon, },);
        printCost({
          cost: {
            ...NOTHING,
            unreportedCalls: 1,
          },
        },);
        expect(printed.lines,).toEqual([
          'metered seats, priced at rates read 2026-09-11:',
          '  none. No call in these logs went to the metered provider',
          'metered run total: 0.00 credits',
          'FLOOR, NOT A TOTAL: 1 call reported no usage block, so its tokens are in no figure of this report',
        ],);
      },
    },),

    it({
      name: 'SAYS several calls reported no usage block, with their pronoun in the plural',
      fn: async (ctx) => {
        using printed = divertingConsoleLog({ sinon: ctx.sinon, },);
        printCost({
          cost: {
            ...NOTHING,
            unreportedCalls: 3,
          },
        },);
        expect(printed.lines,).toEqual([
          'metered seats, priced at rates read 2026-09-11:',
          '  none. No call in these logs went to the metered provider',
          'metered run total: 0.00 credits',
          'FLOOR, NOT A TOTAL: 3 calls reported no usage block, so their tokens are in no figure of this report',
        ],);
      },
    },),
  ],
},);
