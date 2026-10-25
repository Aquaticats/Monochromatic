/**
 Tests for the pricing of one DeepSeek or Xiaomi call from the usage it reported and the card's rates.

 Rates in the expected figures are the cards' own, in USD per million tokens: DeepSeek at peak 0.006 (hit), 0.3 (miss)
 and 1.2 (output), and half of each when the call started outside a weekday peak window (UTC); Xiaomi 0.0028 (cached),
 0.14 and 0.28. Instants are written in UTC; 2026-10-21 is a Wednesday, 2026-10-23 a Friday, 2026-10-24 a Saturday and
 2026-10-25 a Sunday. Fixtures are cat-themed invention.

 @module
 */

import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import {
  DEEPSEEK_MODELS,
  DEEPSEEK_OFF_PEAK_MULTIPLIER,
  deepSeekCostOf,
  DIRECT_COST_UNREPORTED,
  isDeepSeekPeakAt,
  XIAOMI_MODELS,
  xiaomiCostOf,
} from '../dist/final/node/index.mjs';

/**
 DeepSeek's card, whose rates price a call.
 */
const DEEPSEEK_CARD = DEEPSEEK_MODELS['deepseek-flash'];

/**
 Xiaomi's card, whose rates price a call.
 */
const XIAOMI_CARD = XIAOMI_MODELS['mimo-v2.6-flash'];

/**
 A completion whose usage block reported 100 prompt and 10 completion tokens.
 */
const HUNDRED_AND_TEN = { text: '', usage: { prompt_tokens: 100, completion_tokens: 10, total_tokens: 110, }, };

/**
 A completion whose usage block reported no tokens at all.
 */
const NO_TOKENS = { text: '', usage: { prompt_tokens: 0, completion_tokens: 0, total_tokens: 0, }, };

/**
 A Wednesday at 02:00 UTC, inside a peak window, so the cases of the splits and the clamps are priced at the card's own
 rates.
 */
const WEDNESDAY_PEAK = new Date('2026-10-21T02:00:00.000Z',);

/**
 USD of the call that read 40 hits, 60 misses and 10 output tokens, at the card's peak rates.
 */
const PEAK_USD = ((40 * 0.006) + (60 * 0.3) + (10 * 1.2)) / 1_000_000;

/**
 USD of the same call at half the peak rates, the off-peak price.
 */
const OFF_PEAK_USD = PEAK_USD / 2;

/**
 Prices the 40-hit, 60-miss, 10-output call as if it started at an instant.

 @param startedAt - ISO 8601 instant, in UTC, the request started at

 @returns USD the call cost, or that nothing was reported

 @example
 ```ts
 const usd = costStartedAt({ startedAt: '2026-10-24T02:00:00.000Z', },); // a Saturday
 ```
 */
function costStartedAt({ startedAt, }: { readonly startedAt: string; },): number | typeof DIRECT_COST_UNREPORTED {
  return deepSeekCostOf({
    card: DEEPSEEK_CARD,
    extracted: HUNDRED_AND_TEN,
    split: { hitTokens: 40, missTokens: 60, },
    startedAt: new Date(startedAt,),
  },);
}

await describe({
  name: 'direct cost',
  concurrency: DEFAULT_CONCURRENCY,
  children: [
    describe({
      name: deepSeekCostOf.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'PRICES DeepSeek at peak with the cache split, all misses when the block split nothing, and reports '
            + 'nothing when no usage arrived',
          fn: async () => {
            const card = DEEPSEEK_CARD;
            const extracted = HUNDRED_AND_TEN;
            const startedAt = WEDNESDAY_PEAK;
            expect(deepSeekCostOf({ card, extracted, split: { hitTokens: 40, missTokens: 60, }, startedAt, },),)
              .toBeCloseTo(((40 * 0.006) + (60 * 0.3) + (10 * 1.2)) / 1_000_000, 12,);
            expect(deepSeekCostOf({ card, extracted, split: {}, startedAt, },),)
              .toBeCloseTo(((100 * 0.3) + (10 * 1.2)) / 1_000_000, 12,);
            expect(deepSeekCostOf({ card, extracted, split: { hitTokens: 40, }, startedAt, },),)
              .toBeCloseTo(((40 * 0.006) + (60 * 0.3) + (10 * 1.2)) / 1_000_000, 12,);
            expect(deepSeekCostOf({ card, extracted: { text: '', }, split: {}, startedAt, },),)
              .toBe(DIRECT_COST_UNREPORTED,);
          },
        },),
        it({
          name: 'CLAMPS a hit count above the prompt count to the prompt count, since no more tokens can be served '
            + 'from '
            + 'the cache than the prompt held',
          fn: async () => {
            expect(
              deepSeekCostOf({
                card: DEEPSEEK_CARD,
                extracted: HUNDRED_AND_TEN,
                split: { hitTokens: 150, },
                startedAt: WEDNESDAY_PEAK,
              },),
            )
              .toBeCloseTo(((100 * 0.006) + (10 * 1.2)) / 1_000_000, 12,);
          },
        },),
        it({
          name: 'CLAMPS a miss count to what is left of the prompt after the hits, so hit and miss together never '
            + 'price '
            + 'more tokens than the prompt held',
          fn: async () => {
            expect(deepSeekCostOf({
              card: DEEPSEEK_CARD,
              extracted: HUNDRED_AND_TEN,
              split: { hitTokens: 150, missTokens: 20, },
              startedAt: WEDNESDAY_PEAK,
            },),)
              .toBeCloseTo(((100 * 0.006) + (10 * 1.2)) / 1_000_000, 12,);
            expect(deepSeekCostOf({
              card: DEEPSEEK_CARD,
              extracted: HUNDRED_AND_TEN,
              split: { hitTokens: 40, missTokens: 150, },
              startedAt: WEDNESDAY_PEAK,
            },),)
              .toBeCloseTo(((40 * 0.006) + (60 * 0.3) + (10 * 1.2)) / 1_000_000, 12,);
            expect(
              deepSeekCostOf({
                card: DEEPSEEK_CARD,
                extracted: HUNDRED_AND_TEN,
                split: { missTokens: 150, },
                startedAt: WEDNESDAY_PEAK,
              },),
            )
              .toBeCloseTo(((100 * 0.3) + (10 * 1.2)) / 1_000_000, 12,);
          },
        },),
        it({
          name: 'PRICES a block that names only a miss count as that many misses and no hits, and the output either '
            + 'way',
          fn: async () => {
            expect(
              deepSeekCostOf({
                card: DEEPSEEK_CARD,
                extracted: HUNDRED_AND_TEN,
                split: { missTokens: 60, },
                startedAt: WEDNESDAY_PEAK,
              },),
            )
              .toBeCloseTo(((60 * 0.3) + (10 * 1.2)) / 1_000_000, 12,);
          },
        },),
        it({
          name: 'PRICES a zero-token usage block at zero, which is a reported cost and not an unreported one',
          fn: async () => {
            expect(
              deepSeekCostOf({ card: DEEPSEEK_CARD, extracted: NO_TOKENS, split: {}, startedAt: WEDNESDAY_PEAK, },),
            ).toBe(0,);
          },
        },),
        it({
          name: 'BOOKS a weekday call at peak, 02:00 UTC on a Wednesday, and an off-peak Saturday call at half of the '
            + 'peak rates, the same call at 02:00 UTC on a Saturday',
          fn: async () => {
            expect(costStartedAt({ startedAt: '2026-10-21T02:00:00.000Z', },),).toBeCloseTo(PEAK_USD, 12,);
            expect(costStartedAt({ startedAt: '2026-10-24T02:00:00.000Z', },),).toBeCloseTo(OFF_PEAK_USD, 12,);
            expect(costStartedAt({ startedAt: '2026-10-24T07:30:00.000Z', },),).toBeCloseTo(OFF_PEAK_USD, 12,);
          },
        },),
        it({
          name: 'BOOKS a call at 23:59 on a Sunday at the off-peak rates, and one on a Sunday morning inside the hours '
            + 'a weekday would be peak',
          fn: async () => {
            expect(costStartedAt({ startedAt: '2026-10-25T23:59:00.000Z', },),).toBeCloseTo(OFF_PEAK_USD, 12,);
            expect(costStartedAt({ startedAt: '2026-10-25T01:00:00.000Z', },),).toBeCloseTo(OFF_PEAK_USD, 12,);
            expect(costStartedAt({ startedAt: '2026-10-25T06:00:00.000Z', },),).toBeCloseTo(OFF_PEAK_USD, 12,);
          },
        },),
        it({
          name: 'BOOKS the first window from 01:00:00 at peak, to 03:59:59.999 at peak, and from 04:00:00 at '
            + 'off-peak, the end being half-open',
          fn: async () => {
            expect(costStartedAt({ startedAt: '2026-10-21T00:59:59.999Z', },),).toBeCloseTo(OFF_PEAK_USD, 12,);
            expect(costStartedAt({ startedAt: '2026-10-21T01:00:00.000Z', },),).toBeCloseTo(PEAK_USD, 12,);
            expect(costStartedAt({ startedAt: '2026-10-21T03:59:59.999Z', },),).toBeCloseTo(PEAK_USD, 12,);
            expect(costStartedAt({ startedAt: '2026-10-21T04:00:00.000Z', },),).toBeCloseTo(OFF_PEAK_USD, 12,);
          },
        },),
        it({
          name: 'BOOKS the second window from 06:00:00 at peak, to 09:59:59.999 at peak, and from 10:00:00 at '
            + 'off-peak, the end being half-open',
          fn: async () => {
            expect(costStartedAt({ startedAt: '2026-10-21T05:59:59.999Z', },),).toBeCloseTo(OFF_PEAK_USD, 12,);
            expect(costStartedAt({ startedAt: '2026-10-21T06:00:00.000Z', },),).toBeCloseTo(PEAK_USD, 12,);
            expect(costStartedAt({ startedAt: '2026-10-21T09:59:59.999Z', },),).toBeCloseTo(PEAK_USD, 12,);
            expect(costStartedAt({ startedAt: '2026-10-21T10:00:00.000Z', },),).toBeCloseTo(OFF_PEAK_USD, 12,);
          },
        },),
        it({
          name: 'BOOKS the edges of the week by the UTC weekday: Friday 09:59 at peak, Friday 10:00 and 23:59 off-peak, '
            + 'Monday 00:59 off-peak and Monday 01:00 at peak',
          fn: async () => {
            expect(costStartedAt({ startedAt: '2026-10-23T09:59:59.999Z', },),).toBeCloseTo(PEAK_USD, 12,);
            expect(costStartedAt({ startedAt: '2026-10-23T10:00:00.000Z', },),).toBeCloseTo(OFF_PEAK_USD, 12,);
            expect(costStartedAt({ startedAt: '2026-10-23T23:59:00.000Z', },),).toBeCloseTo(OFF_PEAK_USD, 12,);
            expect(costStartedAt({ startedAt: '2026-10-26T00:59:59.999Z', },),).toBeCloseTo(OFF_PEAK_USD, 12,);
            expect(costStartedAt({ startedAt: '2026-10-26T01:00:00.000Z', },),).toBeCloseTo(PEAK_USD, 12,);
          },
        },),
      ],
    },),
    describe({
      name: isDeepSeekPeakAt.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'READS the page\'s two windows as half-open on the hour: 01:00:00 and 06:00:00 are peak, 04:00:00 and '
            + '10:00:00 are off peak, and so is every other hour of a weekday',
          fn: async () => {
            /**
             Instants of a Wednesday paired with whether they are peak.
             */
            const wednesday: readonly (readonly [string, boolean])[] = [
              ['00:00:00.000', false,],
              ['00:59:59.999', false,],
              ['01:00:00.000', true,],
              ['02:30:00.000', true,],
              ['03:59:59.999', true,],
              ['04:00:00.000', false,],
              ['05:59:59.999', false,],
              ['06:00:00.000', true,],
              ['09:59:59.999', true,],
              ['10:00:00.000', false,],
              ['23:59:59.999', false,],
            ];
            for (const [time, peak,] of wednesday)
              expect(isDeepSeekPeakAt({ at: new Date(`2026-10-21T${time}Z`,), },),).toBe(peak,);
          },
        },),
        it({
          name: 'READS Saturday and Sunday as off peak at every hour, the page naming Monday through Friday only',
          fn: async () => {
            for (const day of ['2026-10-24', '2026-10-25',]) {
              for (let hour = 0; hour < 24; hour += 1) {
                expect(
                  isDeepSeekPeakAt({ at: new Date(`${day}T${String(hour,).padStart(2, '0',)}:30:00.000Z`,), },),
                ).toBe(false,);
              }
            }
          },
        },),
        it({
          name: 'READS the weekday in UTC: a Monday reaches peak at 01:00 UTC and a Friday leaves it at 10:00 UTC',
          fn: async () => {
            expect(isDeepSeekPeakAt({ at: new Date('2026-10-26T00:59:59.999Z',), },),).toBe(false,);
            expect(isDeepSeekPeakAt({ at: new Date('2026-10-26T01:00:00.000Z',), },),).toBe(true,);
            expect(isDeepSeekPeakAt({ at: new Date('2026-10-23T09:59:59.999Z',), },),).toBe(true,);
            expect(isDeepSeekPeakAt({ at: new Date('2026-10-23T10:00:00.000Z',), },),).toBe(false,);
          },
        },),
        it({
          name: 'READS an instant that is not a time as peak, the dearer rate, so a failing clock never books less than '
            + 'the account may have been charged',
          fn: async () => {
            expect(isDeepSeekPeakAt({ at: new Date(Number.NaN,), },),).toBe(true,);
          },
        },),
        it({
          name: 'HALVES the card\'s rates off peak: the multiplier is 0.5, as the page states its off-peak rates',
          fn: async () => {
            expect(DEEPSEEK_OFF_PEAK_MULTIPLIER,).toBe(0.5,);
          },
        },),
      ],
    },),
    describe({
      name: xiaomiCostOf.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'PRICES Xiaomi with the cached count and reports nothing without usage',
          fn: async () => {
            const card = XIAOMI_CARD;
            const extracted = HUNDRED_AND_TEN;
            expect(xiaomiCostOf({ card, extracted, cachedTokens: 50, },),)
              .toBeCloseTo(((50 * 0.0028) + (50 * 0.14) + (10 * 0.28)) / 1_000_000, 12,);
            expect(xiaomiCostOf({ card, extracted, cachedTokens: 0, },),)
              .toBeCloseTo(((100 * 0.14) + (10 * 0.28)) / 1_000_000, 12,);
            expect(xiaomiCostOf({ card, extracted: { text: '', }, cachedTokens: 0, },),).toBe(DIRECT_COST_UNREPORTED,);
          },
        },),
        it({
          name: 'CLAMPS a cached count above the prompt count to the prompt count, so the figure never exceeds what '
            + 'the '
            + 'prompt can support',
          fn: async () => {
            expect(xiaomiCostOf({ card: XIAOMI_CARD, extracted: HUNDRED_AND_TEN, cachedTokens: 500, },),)
              .toBeCloseTo(((100 * 0.0028) + (10 * 0.28)) / 1_000_000, 12,);
          },
        },),
        it({
          name: 'PRICES a zero-token usage block at zero, which is a reported cost and not an unreported one',
          fn: async () => {
            expect(xiaomiCostOf({ card: XIAOMI_CARD, extracted: NO_TOKENS, cachedTokens: 0, },),).toBe(0,);
          },
        },),
      ],
    },),
  ],
},);
