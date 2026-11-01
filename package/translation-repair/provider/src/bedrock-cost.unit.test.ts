/**
 Tests for the Bedrock cost: usage times the catalog's prices, and the named
 absence where no usage arrived.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  BEDROCK_COST_UNREPORTED,
  BEDROCK_MODELS,
  bedrockCostOf,
} from '../dist/final/node/index.mjs';
import {
  SEAT_BEDROCK_ONLY_TEXT,
  SEAT_BEDROCK_ONLY_VISION_UNSEATED,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';

await describe({
  name: bedrockCostOf.name,
  children: [
    it({
      name: 'PRICES prompt and completion tokens each at their own rate per million',
      fn: async () => {
        /**
         Row the call is priced by.
         */
        const info = BEDROCK_MODELS[SEAT_BEDROCK_ONLY_VISION_UNSEATED];
        expect(bedrockCostOf({
          servedId: SEAT_BEDROCK_ONLY_VISION_UNSEATED,
          extracted: {
            text: '{"spot":"sunbeam"}',
            usage: {
              prompt_tokens: 1_000_000,
              completion_tokens: 500_000,
              total_tokens: 1_500_000,
            },
          },
        },),).toBeCloseTo(
          info.promptUsdPerMillion + (info.completionUsdPerMillion / 2),
          9,
        );
      },
    },),

    it({
      name: 'PRICES a small call to a fraction of a cent rather than rounding it away',
      fn: async () => {
        /**
         What the probe of 2026-09-07 cost on Gemma 4 E2B: 90 prompt and 10
         completion tokens.
         */
        const cost = bedrockCostOf({
          servedId: SEAT_BEDROCK_ONLY_TEXT,
          extracted: {
            text: '{"spot":"on the sunny windowsill"}',
            usage: {
              prompt_tokens: 90,
              completion_tokens: 10,
              total_tokens: 100,
            },
          },
        },);
        expect(cost,).toBeGreaterThan(0,);
        expect(cost,).toBeLessThan(0.0001,);
      },
    },),

    it({
      name: 'NAMES the absence when the stream carried no usage, so nothing is priced or written',
      fn: async () => {
        expect(bedrockCostOf({
          servedId: 'openai.gpt-oss-120b',
          extracted: { text: 'hello', },
        },),).toBe(BEDROCK_COST_UNREPORTED,);
      },
    },),
  ],
},);
