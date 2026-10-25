/**
 Tests for the bound an abandoned Claude attempt is written at.

 A Claude stream that ends without its terminator is retried inside one call, and each attempt that delivered something
 was billed for what it generated. Until this unit the Claude client told nobody, so a call's SPEND line read the last
 attempt alone and the run meter read a floor. These cases pin the bound (no more prompt tokens than the body had bytes,
 no more completion tokens than the `max_tokens` it sent, the prompt at the higher of the input and cache-write rates),
 the line's mark, and the run meter moving by the same figure.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import {
  HAIKU_5_5_PRICES,
  resetRunSpend,
  runSpendUsd,
  SONNET_5_5_PRICES,
} from '@monochromatic-dev/translation-repair-shared-contract';
import { infoLinesDuring, } from '@monochromatic-dev/translation-repair-typography/ts/console-warn-lines.test-fixture.ts';
import {
  claudeAttemptBound,
  reportClaudeAbandonedSpend,
} from '../dist/final/node/index.mjs';

await describe({
  name: 'the bound of an abandoned Claude attempt',
  // SEQUENTIAL: the run spend meter is process-wide, so cases that read its change must not overlap.
  concurrency: 1,
  children: [
    describe({
      name: claudeAttemptBound.name,
      children: [
        it({
          name: 'PRICES THE PROMPT AS MANY TOKENS AS THE BODY HAS BYTES AND THE COMPLETION AS THE CEILING THE CALL SENT: '
            + '10,000 bytes and 32,000 tokens on Sonnet are (10,000 x 2.5 + 32,000 x 10) / 1e6 = 0.345 USD, the prompt at '
            + 'the cache-write rate, which is above the input rate',
          fn: async () => {
            const bound = claudeAttemptBound({
              prices: SONNET_5_5_PRICES,
              requestBodyBytes: 10_000,
              maxTokens: 32_000,
            },);

            expect(bound.promptTokens,).toBe(10_000,);
            expect(bound.completionTokens,).toBe(32_000,);
            expect(bound.usd,).toBeCloseTo(0.345, 10,);
          },
        },),
        it({
          name: 'PRICES THE WHOLE ATTEMPT AT THE LONGER TIER once the body has bytes past the prompt size that picks it: '
            + '150,000 bytes and 1,000 tokens on Haiku are (150,000 x 0.625 + 1,000 x 2.5) / 1e6 = 0.0962500 USD',
          fn: async () => {
            const bound = claudeAttemptBound({
              prices: HAIKU_5_5_PRICES,
              requestBodyBytes: 150_000,
              maxTokens: 1_000,
            },);

            expect(bound.usd,).toBeCloseTo(0.09625, 10,);
          },
        },),
        it({
          name: 'KEEPS THE BASE TIER at or under the size that picks the longer one: 100,000 bytes and 1,000 tokens on Haiku '
            + 'are (100,000 x 0.125 + 1,000 x 0.5) / 1e6 = 0.013 USD',
          fn: async () => {
            const bound = claudeAttemptBound({
              prices: HAIKU_5_5_PRICES,
              requestBodyBytes: 100_000,
              maxTokens: 1_000,
            },);

            expect(bound.usd,).toBeCloseTo(0.013, 10,);
          },
        },),
      ],
    },),
    describe({
      name: reportClaudeAbandonedSpend.name,
      children: [
        it({
          name: 'WRITES ONE SPEND LINE NAMED BY THE CARD with the bound\'s counts and cost and the abandoned-bound mark, '
            + 'and moves the run meter by the same figure',
          fn: async () => {
            resetRunSpend();

            const { logged, result: line, } = await infoLinesDuring({
              run: () =>
                Promise.resolve(reportClaudeAbandonedSpend({
                  label: 'claude-sonnet-5-5-effort-low',
                  prices: SONNET_5_5_PRICES,
                  requestBodyBytes: 10_000,
                  maxTokens: 32_000,
                },),),
            },);

            expect(line,).toBe(
              'SPEND provider=anthropic model=claude-sonnet-5-5-effort-low prompt=10000 completion=32000 cost=0.345 '
                + 'estimated=abandoned-bound',
            );
            expect(logged.filter(function isSpend(entry,): boolean {
              return entry.includes('SPEND ',);
            },).length,).toBe(1,);
            expect(runSpendUsd({ provider: 'anthropic', },),).toBeCloseTo(0.345, 10,);
          },
        },),
      ],
    },),
  ],
},);
