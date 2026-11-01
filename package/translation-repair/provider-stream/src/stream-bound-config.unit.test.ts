/**
 Tests that a configuration whose stream bounds cannot fire before the per-call
 deadline is refused when it is built, and that the shipped bounds clear what
 the pass logs show a healthy stream needing.

 WHY: `STREAM_FIRST_BYTE_MS` (now `STREAM_FIRST_CONTENT_MS`) and `STREAM_IDLE_MS` were 600,000 ms and the
 per-call deadline 360,000 ms, so neither guard could fire, and the
 `zheermao22` artifact stamped `perCallTimeoutMs: 360000, streamFirstByteMs:
 600000, streamIdleMs: 600000` without anything having noticed (ledger B804).

 WHAT THESE PIN: a bound at or above the deadline, not a number, not positive or
 not finite is refused by name with the values and the remedy; a consistent
 pair is accepted; and the shipped windows sit above the largest healthy first
 byte and the largest healthy silence in the pass logs from `CuspariaKLSY20` to
 `zheermao22` (19542 completed streams that delivered text: first byte 115725
 ms, silence between two chunks 93736 ms), the first-content window by 56
 percent because the first byte is only a lower bound on the wait for text.

 Fixtures are cat-themed invention.

 @module
 */

import {
  caught,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  requireStreamBoundsInsideCallTimeout,
  STREAM_FIRST_CONTENT_MS,
  STREAM_IDLE_MS,
  StreamBoundsOutsideCallTimeoutError,
} from '../dist/final/node/index.mjs';

/**
 Largest time to first byte of a completed stream that delivered text in the
 pass logs, in milliseconds, a `minimax-m3` call on Hyper. A lower bound on its
 time to first text, which the logs do not record.
 */
const LARGEST_HEALTHY_FIRST_BYTE_MS = 115_725;

/**
 Largest silence between two chunks of such a stream, in milliseconds, a
 `minimax-m3` call on Hyper.
 */
const LARGEST_HEALTHY_SILENCE_MS = 93_736;

/**
 A per-call deadline for the cases, the shipped one in size.
 */
const DEADLINE_MS = 360_000;

await describe({
  name: '',
  children: [
    describe({
      name: requireStreamBoundsInsideCallTimeout.name,
      concurrency: 1,
      children: [
        it({
          name: 'ACCEPTS BOUNDS SHORTER THAN THE PER-CALL DEADLINE',
          fn: async () => {
            await Promise.resolve();
            // A refusal would end the case here, by throwing.
            expect(
              requireStreamBoundsInsideCallTimeout({
                perCallTimeoutMs: DEADLINE_MS,
                streamFirstContentMs: 180_000,
                streamIdleMs: 120_000,
              },),
            ).toBeUndefined();
          },
        },),

        it({
          name: 'REFUSES A FIRST-CONTENT BOUND EQUAL TO THE DEADLINE, since a guard that fires with the deadline ends '
            + 'nothing the deadline would not',
          fn: async () => {
            await Promise.resolve();
            /**
             What the check raised.
             */
            const refusal = caught(function equal(): unknown {
              return requireStreamBoundsInsideCallTimeout({
                perCallTimeoutMs: DEADLINE_MS,
                streamFirstContentMs: DEADLINE_MS,
                streamIdleMs: 120_000,
              },);
            },);
            expect(refusal,).toBeInstanceOf(StreamBoundsOutsideCallTimeoutError,);
            expect((refusal as Error).message,).toContain('streamFirstContentMs',);
            expect((refusal as Error).message,).toContain('360000',);
            expect((refusal as Error).message,).not.toContain('streamIdleMs',);
          },
        },),

        it({
          name: 'REFUSES BOTH BOUNDS AT THE OLD 600000, the pair that shipped, and names both with the deadline and '
            + 'what to do',
          fn: async () => {
            await Promise.resolve();
            /**
             What the check raised.
             */
            const refusal = caught(function shipped(): unknown {
              return requireStreamBoundsInsideCallTimeout({
                perCallTimeoutMs: DEADLINE_MS,
                streamFirstContentMs: 600_000,
                streamIdleMs: 600_000,
              },);
            },);
            expect(refusal,).toBeInstanceOf(StreamBoundsOutsideCallTimeoutError,);
            /**
             The refusal's words.
             */
            const { message, } = refusal as Error;
            expect(message,).toContain('streamFirstContentMs',);
            expect(message,).toContain('streamIdleMs',);
            expect(message,).toContain('600000',);
            expect(message,).toContain('360000',);
            expect(message,).toContain('lower',);
            expect(message,).toContain('raise',);
          },
        },),

        it({
          name: 'REFUSES AN IDLE BOUND ABOVE THE DEADLINE while the first-content bound is fine',
          fn: async () => {
            await Promise.resolve();
            /**
             What the check raised.
             */
            const refusal = caught(function idleAbove(): unknown {
              return requireStreamBoundsInsideCallTimeout({
                perCallTimeoutMs: DEADLINE_MS,
                streamFirstContentMs: 120_000,
                streamIdleMs: DEADLINE_MS + 1,
              },);
            },);
            expect(refusal,).toBeInstanceOf(StreamBoundsOutsideCallTimeoutError,);
            expect((refusal as Error).message,).toContain('streamIdleMs',);
            expect((refusal as Error).message,).not.toContain('streamFirstContentMs',);
          },
        },),

        it({
          name: 'REFUSES A BOUND THAT IS NOT A POSITIVE FINITE NUMBER, which a comparison with the deadline would pass '
            + 'or never trip on: not-a-number, zero, a negative, infinity',
          fn: async () => {
            await Promise.resolve();
            for (const bad of [Number.NaN, 0, -1, Number.POSITIVE_INFINITY,]) {
              expect(
                caught(function badFirstContent(): unknown {
                  return requireStreamBoundsInsideCallTimeout({
                    perCallTimeoutMs: DEADLINE_MS,
                    streamFirstContentMs: bad,
                    streamIdleMs: 120_000,
                  },);
                },),
              ).toBeInstanceOf(StreamBoundsOutsideCallTimeoutError,);
              expect(
                caught(function badIdle(): unknown {
                  return requireStreamBoundsInsideCallTimeout({
                    perCallTimeoutMs: DEADLINE_MS,
                    streamFirstContentMs: 120_000,
                    streamIdleMs: bad,
                  },);
                },),
              ).toBeInstanceOf(StreamBoundsOutsideCallTimeoutError,);
            }
          },
        },),

        it({
          name: 'REFUSES A DEADLINE THAT IS NOT A POSITIVE FINITE NUMBER, since no bound is shorter than it',
          fn: async () => {
            await Promise.resolve();
            for (const bad of [Number.NaN, 0, -1, Number.POSITIVE_INFINITY,]) {
              expect(
                caught(function badDeadline(): unknown {
                  return requireStreamBoundsInsideCallTimeout({
                    perCallTimeoutMs: bad,
                    streamFirstContentMs: 120_000,
                    streamIdleMs: 120_000,
                  },);
                },),
              ).toBeInstanceOf(StreamBoundsOutsideCallTimeoutError,);
            }
          },
        },),
      ],
    },),

    describe({
      name: 'shipped stream bounds',
      concurrency: 1,
      children: [
        it({
          name: 'KEEPS BOTH BOUNDS ABOVE THE LARGEST HEALTHY WAIT AND SILENCE THE PASS LOGS SHOW, so no completed stream '
            + 'in them would have been cut',
          fn: async () => {
            await Promise.resolve();
            expect(STREAM_FIRST_CONTENT_MS,).toBeGreaterThan(LARGEST_HEALTHY_FIRST_BYTE_MS,);
            expect(STREAM_IDLE_MS,).toBeGreaterThan(LARGEST_HEALTHY_SILENCE_MS,);
          },
        },),

        it({
          name: 'KEEPS THE FIRST-CONTENT BOUND HALF AGAIN OVER THE LARGEST FIRST BYTE, since the first byte is only a '
            + 'lower bound on the wait for the first text and a margin of 3.7 percent could have been negative',
          fn: async () => {
            await Promise.resolve();
            expect(STREAM_FIRST_CONTENT_MS,).toBeGreaterThanOrEqual(LARGEST_HEALTHY_FIRST_BYTE_MS * 1.5,);
          },
        },),

        it({
          name: 'KEEPS BOTH BOUNDS SHORTER THAN THE SHIPPED PER-CALL DEADLINE of 360000 ms, so each can fire first',
          fn: async () => {
            await Promise.resolve();
            // A refusal would end the case here, by throwing.
            expect(
              requireStreamBoundsInsideCallTimeout({
                perCallTimeoutMs: DEADLINE_MS,
                streamFirstContentMs: STREAM_FIRST_CONTENT_MS,
                streamIdleMs: STREAM_IDLE_MS,
              },),
            ).toBeUndefined();
          },
        },),
      ],
    },),
  ],
},);
