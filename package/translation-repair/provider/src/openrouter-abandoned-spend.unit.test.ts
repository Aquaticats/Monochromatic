/**
 Tests for the reckoned spend line an abandoned OpenRouter stream writes.

 Between the top-up of 2026-09-08 and the refusal of 2026-09-09 the log
 summed 141.62 USD where the meter moved 199.92, the difference being
 streams the rounds abandoned and no line recorded. These cases pin the
 reckoning, the line's mark, the run meter moving, and silence where the
 error says nothing about what was delivered.

 ONE UNIT (ledger P7, 2026-09-28). The reckoning divides raw wire characters
 by a raw-characters-per-token ratio, and a cut stream carried raw
 characters while an overrun or a degenerate ending carried one channel's
 decoded count, so those two read about a hundredth of what they cost
 (`completion=5` for 1,633 content characters on shihai4h2). And no call is
 billed past the `max_tokens` it sent, so neither is the reckoning.

 @module
 */

import { nonNullishOrThrow, } from '@monochromatic-dev/or-throw-core/ts';

import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  MODEL_CARDS,
  resetRunSpend,
  runSpendUsd,
} from '@monochromatic-dev/translation-repair-shared-contract';
import {
  deliveredCharsOf,
  StreamCutShortError,
  StreamDegenerateError,
  StreamOverrunError,
} from '@monochromatic-dev/translation-repair-provider-stream';
import {
  estimateAbandonedSpend,
  OPENROUTER_MODELS,
  rawCharsPerCompletionTokenOf,
  reportAbandonedSpend,
} from '../dist/final/node/index.mjs';

/**
 Raw characters the reckoning divides by for MiniMax M3, as its card
 measured them.
 */
const MINIMAX_RAW_CHARS_PER_TOKEN = (function measuredRatio(): number {
  /**
   Ratio the card carries.
   */
  const ratio = MODEL_CARDS['minimax-m3'].openrouter?.rawCharsPerToken;
  if ((typeof ratio) !== 'number')
    throw new Error('the MiniMax M3 card carries a measured ratio',);
  return ratio as number;
})();

/**
 Raw characters a cut stream delivered, ten tokens at MiniMax M3's ratio.
 */
const TEN_TOKENS_RAW = 10 * MINIMAX_RAW_CHARS_PER_TOKEN;

/**
 A cut stream that delivered ten tokens' worth of raw characters.
 */
const CUT = new StreamCutShortError({
  label: 'minimax/minimax-m3',
  partialText: 'x'.repeat(TEN_TOKENS_RAW,),
  progress: {
    firstByteMs: 500,
    firstContentMs: 500,
    maxGapMs: 100,
    elapsedMs: 4_000,
    chars: TEN_TOKENS_RAW,
  },
  cause: new Error('abandoned',),
},);

/**
 Ceiling far above anything these fixtures deliver, for the cases that are
 not about the ceiling.
 */
const ROOMY_MAX_TOKENS = 100_000;

/**
 Tokens in one million, the listing's unit.
 */
const MILLION = 1_000_000;

/**
 Median of ascending numbers: the middle one where the count is odd and the
 mean of the two around the middle where it is even.

 @param ascending - numbers least first

 @returns Middle of the ascending numbers

 @throws When the list holds no number
 */
function medianOf(ascending: readonly number[],): number {
  /**
   Position halfway through the list, an index where the count is odd.
   */
  const middle = (ascending.length - 1) / 2;
  /**
   Index of the value below the middle.
   */
  const lowerAt = Math.floor(middle,);
  /**
   Index of the value above the middle, the same index where the count is
   odd.
   */
  const upperAt = Math.ceil(middle,);
  /**
   Value below the middle.
   */
  const lower = nonNullishOrThrow(ascending.at(lowerAt,),);
  /**
   Value above the middle.
   */
  const upper = nonNullishOrThrow(ascending.at(upperAt,),);
  return (lower + upper) / 2;
}

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: deliveredCharsOf.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'READS RAW WIRE CHARACTERS off a cut stream and off both streams this pipeline ends, never one '
            + 'channel\'s decoded count, and answers nothing-known for any other error',
          fn: async () => {
            expect({
              cut: deliveredCharsOf({ error: CUT, },),
              overrun: deliveredCharsOf({
                error: new StreamOverrunError({
                  label: 'minimax/minimax-m3',
                  channel: 'reasoning',
                  charsSeen: 274,
                  cap: 200,
                  rawChars: 27_400,
                },),
              },),
              degenerate: deliveredCharsOf({
                error: new StreamDegenerateError({
                  label: 'minimax/minimax-m3',
                  channel: 'content',
                  distinctRatio: 0.02,
                  charsSeen: 500,
                  rawChars: 50_000,
                },),
              },),
              other: deliveredCharsOf({ error: new Error('HTTP 502',), },),
            },).toEqual({
              cut: TEN_TOKENS_RAW,
              overrun: 27_400,
              degenerate: 50_000,
              other: 'nothing-known',
            },);
          },
        },),
      ],
    },),

    describe({
      name: estimateAbandonedSpend.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'RECKONS completion tokens from raw characters at the measured ratio, prompt tokens from '
            + 'body bytes, and prices both at the listing\'s rates',
          fn: async () => {
            /**
             Listing prices for MiniMax M3.
             */
            const info = OPENROUTER_MODELS['minimax/minimax-m3'];
            /**
             Reckoning for ten tokens delivered on a four-thousand-byte body.
             */
            const estimate = estimateAbandonedSpend({
              servedId: 'minimax/minimax-m3',
              deliveredChars: TEN_TOKENS_RAW,
              requestBodyBytes: 4_000,
              maxTokens: ROOMY_MAX_TOKENS,
            },);
            expect(estimate.completionTokens,).toBe(10,);
            expect(estimate.promptTokens,).toBe(1_000,);
            expect(estimate.usd,).toBeCloseTo(
              ((1_000 * info.promptUsdPerMillion) + (10 * info.completionUsdPerMillion)) / MILLION,
              12,
            );
          },
        },),
        it({
          name: 'NEVER RECKONS MORE COMPLETION TOKENS THAN THE CALL SENT AS max_tokens, since the endpoint '
            + 'bills none past it',
          fn: async () => {
            /**
             Ceiling the call sent, below what the delivered characters stand for.
             */
            const maxTokens = 4;
            expect(estimateAbandonedSpend({
              servedId: 'minimax/minimax-m3',
              deliveredChars: TEN_TOKENS_RAW,
              requestBodyBytes: 4_000,
              maxTokens,
            },).completionTokens,).toBe(maxTokens,);
          },
        },),
      ],
    },),

    describe({
      name: rawCharsPerCompletionTokenOf.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'READS a measured ratio as itself and FALLS BACK to the median of the measured card '
            + 'ratios where a card carries none',
          fn: async () => {
            /**
             Measured ratios the roster's cards carry, least first.
             */
            const measured = Object.values(MODEL_CARDS,).flatMap(function carried(card,): number[] {
              const ratio = card.openrouter?.rawCharsPerToken;
              return ((typeof ratio) === 'number') ? [ratio,] : [];
            },).toSorted(function ascending(a: number, b: number,): number {
              return a - b;
            },);
            expect(rawCharsPerCompletionTokenOf('unmeasured',),).toBe(medianOf(measured,),);
            expect(rawCharsPerCompletionTokenOf(3,),).toBe(3,);
          },
        },),
      ],
    },),

    describe({
      name: reportAbandonedSpend.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'WRITES a SPEND line marked estimated=abandoned for one abandoned attempt, in the grammar the '
            + 'reader already parses, and moves the run meter by the reckoned USD',
          fn: async () => {
            resetRunSpend();
            /**
             Line the report logged.
             */
            const line = reportAbandonedSpend({
              servedId: 'minimax/minimax-m3',
              deliveredChars: TEN_TOKENS_RAW,
              requestBodyBytes: 4_000,
              maxTokens: ROOMY_MAX_TOKENS,
            },);
            expect({
              opens: line.startsWith('SPEND provider=openrouter model=minimax/minimax-m3 prompt=1000 completion=10 cost=',),
              marked: line.endsWith(' estimated=abandoned',),
              metered: runSpendUsd({ provider: 'openrouter', },) > 0,
            },).toEqual({
              opens: true,
              marked: true,
              metered: true,
            },);
          },
        },),
      ],
    },),
  ],
},);
