/**
 Tests for the dated Charm Hyper credit table.

 THE CASE THAT EARNS ITS KEEP is the catalog join: every model the pipeline
 can seat on the metered provider must have a row here. Without it, adding a
 seat to `hyper-catalog.ts` drops it silently into the unpriced bucket, and a
 run total reads as complete while missing a whole model's bill.

 THE PROTOTYPE CASES ARE NOT DECORATION. Model ids arrive from log lines, and
 an object lookup answers `__proto__` and `constructor` with something that is
 not a rate. `spend-read.ts` had exactly this hole in its field table and it
 was found by writing the reader, not by reading the writer.

 RATES ARE ASSERTED AGAINST THE PROVIDER'S OWN QUOTED FIGURES, at round token
 counts chosen so the arithmetic is checkable by eye: a million tokens costs
 exactly the quoted per-million rate.

 Model identifiers come from the catalog. No corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { HYPER_MODELS, } from '@monochromatic-dev/translation-repair-shared-contract';
import {
  creditsFor,
  HYPER_PRICE_READ_ON,
  ratesFor,
} from '../../dist/final/node/index.mjs';
import {
  SEAT_HYPER_TEXT_BEDROCK,
  SEAT_HYPER_VISION,
  SEAT_SYNTHETIC_VISION_WITHHELD,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';

/**
 One million, the unit the provider quotes rates in.
 */
const ONE_MILLION = 1_000_000;

/**
 Half a million, for the case proving the rate scales rather than steps.
 */
const HALF_MILLION = 500_000;

await describe({
  name: 'hyper price table',
  children: [
    it({
      name: 'COVERS every model the catalog can seat on the metered provider, '
        + 'so adding a seat cannot silently drop it into the unpriced bucket',
      fn: async () => {
        expect(
          Object
            .keys(HYPER_MODELS,)
            .filter(function unpriced(model,): boolean {
              return ratesFor({ model, },) === 'unpriced';
            },),
        )
          .toEqual([],);
      },
    },),

    it({
      name: 'CARRIES the date its rates were read, so a report can say how old '
        + 'the figures it prints are',
      fn: async () => {
        expect(HYPER_PRICE_READ_ON,)
          .toBe('2026-10-11',);
      },
    },),

    it({
      name: 'PRICES a million prompt tokens at exactly the quoted input rate',
      fn: async () => {
        expect(creditsFor({
          model: 'qwen3.8-max',
          promptTokens: ONE_MILLION,
          completionTokens: 0,
        },),)
          .toEqual({
            inputCredits: 40,
            outputCredits: 0,
          },);
      },
    },),

    it({
      name: 'PRICES a million completion tokens at the quoted output rate, '
        + 'which is the expensive half and the one thinking lands in',
      fn: async () => {
        expect(creditsFor({
          model: 'qwen3.8-max',
          promptTokens: 0,
          completionTokens: ONE_MILLION,
        },),)
          .toEqual({
            inputCredits: 0,
            outputCredits: 120,
          },);
      },
    },),

    it({
      name: 'PRICES the roster\'s dearest seat at its own rate rather than a '
        + 'shared one, since the spread across this roster is two orders of '
        + 'magnitude',
      fn: async () => {
        expect(creditsFor({
          model: 'kimi-k3',
          promptTokens: ONE_MILLION,
          completionTokens: ONE_MILLION,
        },),)
          .toEqual({
            // Exact provider rate times 20 since the 2026-09-01 regeneration;
            // the earlier 65.33 was a hand-rounded paste.
            inputCredits: 65.328,
            outputCredits: 326.64,
          },);
      },
    },),

    it({
      name: 'RETAINS RETIRED GLM-5.2 PRICING so historical artifacts remain accountable',
      fn: async () => {
        expect(creditsFor({
          model: 'glm-5.2',
          promptTokens: ONE_MILLION,
          completionTokens: ONE_MILLION,
        },),)
          .toEqual({
            inputCredits: 30.4864,
            outputCredits: 95.8144,
          },);
      },
    },),

    it({
      name: 'SCALES with the token count rather than stepping per call',
      fn: async () => {
        expect(creditsFor({
          model: SEAT_HYPER_TEXT_BEDROCK,
          promptTokens: HALF_MILLION,
          completionTokens: 0,
        },),)
          .toEqual({
            // Half of the 1.96 credits per million read on 2026-10-11.
            inputCredits: 0.98,
            outputCredits: 0,
          },);
      },
    },),

    it({
      name: 'REPORTS zero for a seat that reported no tokens, which is a real '
        + 'zero and not the unpriced answer',
      fn: async () => {
        expect(creditsFor({
          model: SEAT_HYPER_VISION,
          promptTokens: 0,
          completionTokens: 0,
        },),)
          .toEqual({
            inputCredits: 0,
            outputCredits: 0,
          },);
      },
    },),

    it({
      name: 'REFUSES a model the table has never heard of, rather than billing '
        + 'it at nothing and reading as a cheaper run',
      fn: async () => {
        expect(ratesFor({ model: 'cat-nap-9000', },),)
          .toBe('unpriced',);
      },
    },),

    it({
      name: 'REFUSES `__proto__`, which an object lookup would answer with the '
        + 'prototype instead of a rate',
      fn: async () => {
        expect(ratesFor({ model: '__proto__', },),)
          .toBe('unpriced',);
      },
    },),

    it({
      name: 'REFUSES `constructor`, for the same reason as `__proto__`',
      fn: async () => {
        expect(ratesFor({ model: 'constructor', },),)
          .toBe('unpriced',);
      },
    },),

    it({
      name: 'REFUSES an empty model id, which a truncated record can carry',
      fn: async () => {
        expect(ratesFor({ model: '', },),)
          .toBe('unpriced',);
      },
    },),

    it({
      name: 'REFUSES a synthetic spelling of a model it prices under the '
        + 'metered id, since the two bill differently and only one bills credits',
      fn: async () => {
        expect(ratesFor({ model: SEAT_SYNTHETIC_VISION_WITHHELD, },),)
          .toBe('unpriced',);
      },
    },),

    it({
      name: 'PRICES an unpriced model as unpriced through `creditsFor` too, '
        + 'rather than only through the lookup beneath it',
      fn: async () => {
        expect(creditsFor({
          model: 'cat-nap-9000',
          promptTokens: ONE_MILLION,
          completionTokens: ONE_MILLION,
        },),)
          .toBe('unpriced',);
      },
    },),
  ],
},);
