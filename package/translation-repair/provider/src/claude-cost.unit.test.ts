/**
 Tests for what one Claude call cost in USD, from the four counts its usage block reports (`claude-cost.ts`), at the
 prices of the Anthropic pricing page as the owner's brief of 2026-10-22 carries them: Sonnet 5.5 at 2 and 10 USD a
 million with a 2.50 five-minute cache write and a 0.10 cache hit; Haiku 5.5 at 0.10, 0.50, 0.125 and 0.01 up to 100,000
 prompt tokens, and 0.50, 2.50, 0.625 and 0.05 past them. Expected figures are worked by hand in each case.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import { MODEL_CARDS, } from '@monochromatic-dev/translation-repair-shared-contract';
import { claudeCostOf, } from '../dist/final/node/index.mjs';

/**
 Prices of the Sonnet 5.5 cards.
 */
const SONNET = MODEL_CARDS['claude-sonnet-5-5'].anthropic?.prices;

/**
 Prices of the Haiku 5.5 cards.
 */
const HAIKU = MODEL_CARDS['claude-haiku-5-5'].anthropic?.prices;

await describe({
  name: claudeCostOf.name,
  children: [
    it({
      name: 'PRICES SONNET\'S FOUR COUNTS EACH AT ITS OWN RATE: 1,000 plain prompt tokens at 2, 2,000 cache writes at '
        + '2.50, 3,000 cache hits at 0.10 and 500 completion tokens at 10 USD a million come to 0.0123',
      fn: async () => {
        if (SONNET === undefined)
          throw new Error('the Sonnet card carries no prices',);
        expect(claudeCostOf({
          prices: SONNET,
          usage: { input: 1_000, cacheWrite: 2_000, cacheRead: 3_000, output: 500, },
        },),).toBeCloseTo(0.0123, 10,);
      },
    },),

    it({
      name: 'PRICES HAIKU AT THE BASE TIER UP TO AND AT 100,000 PROMPT TOKENS, where 100,000 plain tokens and 1,000 '
        + 'completion tokens come to 0.0105',
      fn: async () => {
        if (HAIKU === undefined)
          throw new Error('the Haiku card carries no prices',);
        expect(claudeCostOf({
          prices: HAIKU,
          usage: { input: 100_000, cacheWrite: 0, cacheRead: 0, output: 1_000, },
        },),).toBeCloseTo(0.0105, 10,);
      },
    },),

    it({
      name: 'PRICES HAIKU AT THE LONG TIER ONCE THE PROMPT PASSES 100,000, the whole call at the dearer rates: 100,001 '
        + 'plain tokens and 1,000 completion tokens come to 0.0525005',
      fn: async () => {
        if (HAIKU === undefined)
          throw new Error('the Haiku card carries no prices',);
        expect(claudeCostOf({
          prices: HAIKU,
          usage: { input: 100_001, cacheWrite: 0, cacheRead: 0, output: 1_000, },
        },),).toBeCloseTo(0.0525005, 10,);
      },
    },),

    it({
      name: 'COUNTS THE CACHE TOKENS INTO THE PROMPT THAT PICKS THE TIER: 10 plain, 60,000 written and 50,000 read '
        + 'tokens make a 110,010-token prompt, so 100 completion tokens come to 0.040255 at the long tier',
      fn: async () => {
        if (HAIKU === undefined)
          throw new Error('the Haiku card carries no prices',);
        expect(claudeCostOf({
          prices: HAIKU,
          usage: { input: 10, cacheWrite: 60_000, cacheRead: 50_000, output: 100, },
        },),).toBeCloseTo(0.040255, 10,);
      },
    },),

    it({
      name: 'PRICES A CALL OF NOTHING AT NOTHING',
      fn: async () => {
        if (SONNET === undefined)
          throw new Error('the Sonnet card carries no prices',);
        expect(claudeCostOf({
          prices: SONNET,
          usage: { input: 0, cacheWrite: 0, cacheRead: 0, output: 0, },
        },),).toBe(0,);
      },
    },),
  ],
},);
