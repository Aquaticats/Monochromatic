/**
 Tests for the OpenRouter credits reader.

 THE LIVE SHAPE is the case that matters: `/api/v1/credits` answered the
 ordinary key with `{"data":{"total_credits":1913,"total_usage":1855.383100082}}`
 on 2026-09-03, against a page saying a management key is required.

 @module
 */

import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  OpenRouterCreditsShapeError,
  openRouterIsDry,
  openRouterMeterLevel,
  parseOpenRouterCredits,
} from '../dist/final/node/index.mjs';

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: parseOpenRouterCredits.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'READS THE LIVE SHAPE OF 2026-09-03 into purchased, used and what is left (the live '
            + 'usage carried nine decimals; two are kept here)',
          fn: async () => {
            expect(parseOpenRouterCredits({
              bodyText: '{"data":{"total_credits":1913,"total_usage":1855.38}}',
            },),).toEqual({
              purchasedUsd: 1_913,
              usedUsd: 1_855.38,
              remainingUsd: 1_913 - 1_855.38,
            },);
          },
        },),

        it({
          name: 'REFUSES a body that is not JSON, lacks its envelope, or carries a non-finite figure, '
            + 'since a non-finite balance would read as an unlimited budget',
          fn: async () => {
            for (const bodyText of [
              'not json',
              '[]',
              '{"total_credits":1,"total_usage":0}',
              '{"data":{"total_credits":"1913","total_usage":0}}',
              '{"data":{"total_credits":1913}}',
              '{"data":{"total_credits":1e999,"total_usage":0}}',
            ]) {
              expect(function parse() {
                parseOpenRouterCredits({ bodyText, },);
              },).toThrow(OpenRouterCreditsShapeError,);
            }
          },
        },),
      ],
    },),

    describe({
      name: openRouterIsDry.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'READS any positive remainder as not dry and zero or below as dry, the same rule as Hyper\'s',
          fn: async () => {
            expect(openRouterIsDry({ credits: { purchasedUsd: 1_913, usedUsd: 1_855.38, remainingUsd: 57.62, }, },),)
              .toBe(false,);
            expect(openRouterIsDry({ credits: { purchasedUsd: 10, usedUsd: 10, remainingUsd: 0, }, },),).toBe(true,);
            expect(openRouterIsDry({ credits: { purchasedUsd: 10, usedUsd: 11, remainingUsd: -1, }, },),).toBe(true,);
          },
        },),
      ],
    },),

    describe({
      name: openRouterMeterLevel.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'WRITES what is left in USD to two places, as one space-free field',
          fn: async () => {
            expect(openRouterMeterLevel({
              credits: { purchasedUsd: 1_913, usedUsd: 1_855.384, remainingUsd: 57.616, },
            },),).toEqual(['openrouterUsd=57.62',],);
          },
        },),
      ],
    },),
  ],
},);
