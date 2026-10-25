/**
 Tests for what a caller is told before the Claude client sends an attempt: the admit becomes the retry ladder's own
 `beforeAttempt`, at the figure the attempt's abandoned line would write, and with no admit the ladder is handed nothing.

 Fixtures are cat-themed invention.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import { SONNET_5_5_PRICES, } from '@monochromatic-dev/translation-repair-shared-contract';
import { rejectionOf, } from '@monochromatic-dev/translation-repair-test-fixture/ts/rejecting-call.test-fixture.ts';
import {
  admissionOf,
  claudeAttemptBound,
} from '../dist/final/node/index.mjs';

await describe({
  name: admissionOf.name,
  children: [
    it({
      name: 'HANDS THE LADDER NOTHING when the caller asked to be told of nothing, so the ladder runs as it ran before',
      fn: async () => {
        expect(
          admissionOf({ label: 'claude-sonnet-5-5-effort-low', prices: SONNET_5_5_PRICES, requestBodyBytes: 1_000, maxTokens: 32_000, },),
        ).toStrictEqual({},);
      },
    },),
    it({
      name: 'TELLS THE CALLER THE CARD AND THE ATTEMPT\'S BOUND, the figure the abandoned line writes for the same body: '
        + '1,000 bytes and 32,000 tokens on Sonnet are (1000 x 2.5 + 32000 x 10) / 1e6 USD, once for each telling',
      fn: async () => {
        /**
         What the caller was told.
         */
        const told: { readonly label: string; readonly usd: number; }[] = [];
        const { beforeAttempt, } = admissionOf({
          admitAttempt: function noteAsk({ label, bound, },): void {
            told.push({ label, usd: bound.usd, },);
          },
          label: 'claude-sonnet-5-5-effort-low',
          prices: SONNET_5_5_PRICES,
          requestBodyBytes: 1_000,
          maxTokens: 32_000,
        },);
        await beforeAttempt?.();
        await beforeAttempt?.();
        /**
         The bound the abandoned line would write for the same body.
         */
        const { usd, } = claudeAttemptBound({ prices: SONNET_5_5_PRICES, requestBodyBytes: 1_000, maxTokens: 32_000, },);
        expect(usd,).toBeCloseTo(0.3225, 10,);
        expect(told,).toStrictEqual([
          { label: 'claude-sonnet-5-5-effort-low', usd, },
          { label: 'claude-sonnet-5-5-effort-low', usd, },
        ],);
      },
    },),
    it({
      name: 'LETS WHAT THE CALLER THROWS OUT, so the ladder ends the exchange at the refused attempt',
      fn: async () => {
        /**
         The refusal the caller throws.
         */
        const refusal = new Error('the attempt would pass the amount',);
        const { beforeAttempt, } = admissionOf({
          admitAttempt: function refuses(): void {
            throw refusal;
          },
          label: 'claude-sonnet-5-5-effort-low',
          prices: SONNET_5_5_PRICES,
          requestBodyBytes: 1_000,
          maxTokens: 32_000,
        },);
        expect(await rejectionOf(async () => await beforeAttempt?.(),),).toBe(refusal,);
      },
    },),
  ],
},);
