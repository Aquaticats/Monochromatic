import { wait, } from '@monochromatic-dev/async-time-core/ts';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  armIdleGuard,
  drainBody,
  STREAM_FIRST_CONTENT_MS,
  STREAM_IDLE_MS,
  StreamCutShortError,
  StreamStalledError,
} from '../dist/final/node/index.mjs';

import {
  HOUR_MS,
  stubWallClock,
  WALL_START_MS,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/wall-clock-stub.test-fixture.ts';
import { HANG_STOP_MS, } from '@monochromatic-dev/translation-repair-test-fixture/ts/hang-stop.test-fixture.ts';

//region Fixtures
// Cat-themed invention throughout: no corpus text ever reaches a committed
// fixture, because the corpus is unlicensed.

/**
 Tiny window so a stall test finishes in milliseconds rather than minutes.
 */
const TINY_MS = 20;

/**
 Waits for a stall to trip, by polling the guard's own signal rather than
 sleeping a fixed span, so the test does not race a slow machine.

 @param signal - guard signal expected to abort

 @returns Nothing; resolves once aborted
 */
async function untilAborted(signal: AbortSignal,): Promise<void> {
  const spin = { done: signal.aborted, };
  while (!spin.done) {
    // oxlint-disable-next-line no-await-in-loop -- polling is inherently sequential
    await new Promise(function tick(resolve,) {
      setTimeout(resolve, TINY_MS,);
    },);
    spin.done = signal.aborted;
  }
}

/**
 Builds a Response whose body emits the given chunks, so the drain sees a
 real ReadableStream rather than a whole-body string.

 @param chunks - byte chunks to emit in order

 @returns Response carrying those chunks as its body
 */
function streamingResponse(chunks: readonly Uint8Array[],): Response {
  return new Response(
    new ReadableStream({
      start(controller,) {
        for (const chunk of chunks)
          controller.enqueue(chunk,);
        controller.close();
      },
    },),
  );
}

//endregion Fixtures

await describe({
  name: '',
  children: [
    describe({
      name: 'idle window constants',
      children: [
        it({
          name: 'keeps both windows shorter than the per-call deadline so the guard can end a stalled call before the deadline does (ledger B804)',
          fn: async () => {
            // 360_000 is RUN_PER_CALL_TIMEOUT_MS in
            // corpus-run/run-config.ts, not imported here because it is not
            // re-exported through the package barrel this test builds
            // against. A window at or past it can never fire first.
            expect(STREAM_FIRST_CONTENT_MS,).toBeLessThan(360_000,);
            expect(STREAM_IDLE_MS,).toBeLessThan(360_000,);
          },
        },),

        it({
          name: 'keeps both windows above the highest first-byte wait and mid-stream gap of a completed stream measured in production so far',
          fn: async () => {
            // 115_725 is the largest first byte and 93_736 the largest gap
            // among 19_542 completed streams that carried text (ledger B804).
            // A window below its own history would start cutting healthy calls.
            expect(STREAM_FIRST_CONTENT_MS,).toBeGreaterThan(115_725,);
            expect(STREAM_IDLE_MS,).toBeGreaterThan(93_736,);
          },
        },),

        it({
          name: 'SETS THE FIRST-CONTENT WINDOW AT 180000 ms, 56 percent over the largest first byte, because the logged '
            + 'first byte is only a lower bound on the wait for the first text (ledger B804)',
          fn: async () => {
            expect(STREAM_FIRST_CONTENT_MS,).toBe(180_000,);
            // The idle window is a gap between pieces of text, which the logs measure directly, and stays as it was.
            expect(STREAM_IDLE_MS,).toBe(120_000,);
          },
        },),

        it({
          name: 'STILL CUTS the 183755 ms first byte of a retired roster, which is the outer edge of what the older '
            + 'logs hold and was set aside as evidence rather than cleared (ledger B804)',
          fn: async () => {
            expect(STREAM_FIRST_CONTENT_MS,).toBeLessThan(183_755,);
          },
        },),
      ],
    },),

    describe({
      name: StreamStalledError.name,
      children: [
        it({
          name: 'names the call, the window, and which phase went silent',
          fn: async () => {
            const error = new StreamStalledError({
              label: 'hf:whiskers',
              idleMs: 1_500,
              phase: 'body',
            },);
            expect(error.name,).toBe('StreamStalledError',);
            expect(error.message,).toContain('hf:whiskers',);
            expect(error.message,).toContain('1500',);
            expect(error.message,).toContain('body',);
          },
        },),
      ],
    },),

    describe({
      name: armIdleGuard.name,
      children: [
        //region Tripping

        it({
          name: 'trips on first-content silence and carries the stall as the abort reason',
          fn: async () => {
            using guard = armIdleGuard({
              label: 'hf:whiskers',
              firstContentMs: TINY_MS,
              idleMs: HANG_STOP_MS,
            },);
            await untilAborted(guard.signal,);
            expect(guard.signal.aborted,).toBe(true,);
            expect(guard.signal.reason,).toBeInstanceOf(StreamStalledError,);
            expect((guard.signal.reason as Error).message,).toContain(
              'first-content',
            );
          },
        },),

        it({
          name: 'trips on body silence once the stream has started',
          fn: async () => {
            using guard = armIdleGuard({
              label: 'hf:mittens',
              firstContentMs: HANG_STOP_MS,
              idleMs: TINY_MS,
            },);
            guard.notify(5,);
            guard.notifyGenerated({ content: 5, reasoning: 0, },);
            await untilAborted(guard.signal,);
            expect((guard.signal.reason as Error).message,).toContain('body',);
          },
        },),

        it({
          name: 'does not trip while chunks keep arriving',
          fn: async () => {
            using guard = armIdleGuard({
              label: 'hf:mittens',
              firstContentMs: HANG_STOP_MS,
              idleMs: HANG_STOP_MS,
            },);
            guard.notify(3,);
            guard.notify(4,);
            expect(guard.signal.aborted,).toBe(false,);
          },
        },),

        //endregion Tripping

        //region Progress

        it({
          name: 'reports no first byte before anything arrives',
          fn: async () => {
            using guard = armIdleGuard({
              label: 'hf:mittens',
              firstContentMs: HANG_STOP_MS,
              idleMs: HANG_STOP_MS,
            },);
            expect(guard.progress().firstByteMs,).toBeLessThan(0,);
            expect(guard.progress().chars,).toBe(0,);
          },
        },),

        it({
          name: 'accumulates characters and times the first byte',
          fn: async () => {
            using guard = armIdleGuard({
              label: 'hf:mittens',
              firstContentMs: HANG_STOP_MS,
              idleMs: HANG_STOP_MS,
            },);
            guard.notify(7,);
            guard.notify(11,);
            const progress = guard.progress();
            expect(progress.chars,).toBe(18,);
            expect(progress.firstByteMs,).toBeGreaterThanOrEqual(0,);
            expect(progress.maxGapMs,).toBeGreaterThanOrEqual(0,);
          },
        },),

        it({
          name: 'reports no first content before generated text arrives, though a byte has, and none for an empty '
            + 'total (ledger B804)',
          fn: async () => {
            using guard = armIdleGuard({
              label: 'hf:mittens',
              firstContentMs: HANG_STOP_MS,
              idleMs: HANG_STOP_MS,
            },);
            guard.notify(25,);
            guard.notifyGenerated({ content: 0, reasoning: 0, },);
            expect(guard.progress().firstByteMs,).toBeGreaterThanOrEqual(0,);
            expect(guard.progress().firstContentMs,).toBeLessThan(0,);
          },
        },),

        it({
          name: 'TIMES THE FIRST TEXT LATER THAN THE FIRST BYTE when the byte came first, on a reasoning delta as on a '
            + 'content delta, and keeps the first reading when more text follows (ledger B804)',
          fn: async () => {
            using guard = armIdleGuard({
              label: 'hf:mittens',
              firstContentMs: HANG_STOP_MS,
              idleMs: HANG_STOP_MS,
            },);
            guard.notify(25,);
            await wait(TINY_MS,);
            guard.notify(60,);
            guard.notifyGenerated({ content: 0, reasoning: 4, },);

            /**
             What the guard measured at the first text.
             */
            const first = guard.progress();
            expect(first.firstContentMs,).toBeGreaterThan(first.firstByteMs,);

            await wait(TINY_MS,);
            guard.notify(60,);
            guard.notifyGenerated({ content: 9, reasoning: 4, },);
            expect(guard.progress().firstContentMs,).toBe(first.firstContentMs,);
          },
        },),

        it({
          name: 'CARRIES THE TIME SINCE ARMING, which is the only number that says where a '
            + "run's own hours went: first-byte and gap measure phases inside a call, and no sum "
            + 'of them is the call',
          fn: async () => {
            using guard = armIdleGuard({
              label: 'hf:mittens',
              firstContentMs: HANG_STOP_MS,
              idleMs: HANG_STOP_MS,
            },);

            // A real span, so first byte lands at a measurable offset rather
            // than at zero, where a constant would be indistinguishable from a
            // measurement.
            await wait(TINY_MS,);
            guard.notify(9,);

            /**
             What the guard measured across the whole armed window.
             */
            const progress = guard.progress();

            expect(progress.firstByteMs,).toBeGreaterThan(0,);
            // The call cannot be shorter than the wait for its own first byte.
            expect(progress.elapsedMs,).toBeGreaterThanOrEqual(progress.firstByteMs,);
            expect(Number.isFinite(progress.elapsedMs,),).toBe(true,);
          },
        },),

        it({
          name: 'MEASURES ON A CLOCK THE SYSTEM TIME CANNOT MOVE (ledger B78): with the system clock set back an '
            + 'hour before the first byte and forward two before the next, the first byte, the elapsed time and '
            + 'the largest gap stay the moments between, where they read an hour negative and an hour long',
          fn: async ctx => {
            const wall = stubWallClock({ sinon: ctx.sinon, atMs: WALL_START_MS, },);
            using guard = armIdleGuard({
              label: 'hf:mittens',
              firstContentMs: HANG_STOP_MS,
              idleMs: HANG_STOP_MS,
            },);
            wall.step({ byMs: -HOUR_MS, },);
            guard.notify(3,);

            /**
             What the guard measured at the first byte.
             */
            const early = guard.progress();
            wall.step({ byMs: 2 * HOUR_MS, },);
            guard.notify(4,);

            /**
             What it measured at the second chunk.
             */
            const late = guard.progress();
            expect(early.firstByteMs,).toBeGreaterThanOrEqual(0,);
            expect(early.elapsedMs,).toBeGreaterThanOrEqual(0,);
            expect(late.maxGapMs,).toBeLessThan(HOUR_MS,);
            expect(late.elapsedMs,).toBeLessThan(HOUR_MS,);
          },
        },),

        //endregion Progress

        //region Disposal

        it({
          name: 'stops tripping once disposed',
          fn: async () => {
            const guard = armIdleGuard({
              label: 'hf:mittens',
              firstContentMs: TINY_MS,
              idleMs: TINY_MS,
            },);
            guard[Symbol.dispose]();
            await new Promise(function settle(resolve,) {
              setTimeout(resolve, TINY_MS * 5,);
            },);
            expect(guard.signal.aborted,).toBe(false,);
          },
        },),

        //endregion Disposal
      ],
    },),

    describe({
      name: drainBody.name,
      children: [
        it({
          name: 'concatenates every chunk into the whole body',
          fn: async () => {
            using guard = armIdleGuard({
              label: 'hf:whiskers',
              firstContentMs: HANG_STOP_MS,
              idleMs: HANG_STOP_MS,
            },);
            const encoder = new TextEncoder();
            const bodyText = await drainBody({
              credentials: [],
              response: streamingResponse([
                encoder.encode('data: one\n',),
                encoder.encode('data: two\n',),
              ],),
              guard,
              label: 'hf:whiskers',
              callerSignal: new AbortController().signal,
            },);
            expect(bodyText,).toBe('data: one\ndata: two\n',);
            expect(guard.progress().chars,).toBe(20,);
          },
        },),

        it({
          name: 'decodes a multi-byte character split across two chunks',
          fn: async () => {
            using guard = armIdleGuard({
              label: 'hf:whiskers',
              firstContentMs: HANG_STOP_MS,
              idleMs: HANG_STOP_MS,
            },);
            // U+732B in UTF-8 is e7 8c ab; splitting it proves the decoder is
            // incremental rather than per-chunk.
            const bodyText = await drainBody({
              credentials: [],
              response: streamingResponse([
                new Uint8Array([0xE7, 0x8C,],),
                new Uint8Array([0xAB,],),
              ],),
              guard,
              label: 'hf:whiskers',
              callerSignal: new AbortController().signal,
            },);
            expect(bodyText,).toBe('猫',);
          },
        },),

        it({
          name: 'reads an empty body as the empty string',
          fn: async () => {
            using guard = armIdleGuard({
              label: 'hf:whiskers',
              firstContentMs: HANG_STOP_MS,
              idleMs: HANG_STOP_MS,
            },);
            const bodyText = await drainBody({
              credentials: [],
              response: new Response(null,),
              guard,
              label: 'hf:whiskers',
              callerSignal: new AbortController().signal,
            },);
            expect(bodyText,).toBe('',);
          },
        },),

        it({
          name: 'surfaces the stall rather than the platform abort when the guard trips',
          fn: async () => {
            using guard = armIdleGuard({
              label: 'hf:whiskers',
              firstContentMs: TINY_MS,
              idleMs: TINY_MS,
            },);

            /**
             Body that never emits, so only the guard can end the read.
             */
            const stalled = new Response(
              new ReadableStream({
                start(controller,) {
                  guard.signal
                    .addEventListener(
                    'abort',
                    function tearDown() {
                      controller.error(guard.signal.reason,);
                    },
                    { once: true, },
                    );
                },
              },),
            );

            let caught: unknown;
            try {
              await drainBody({
                credentials: [],
                response: stalled,
                guard,
                label: 'hf:whiskers',
                callerSignal: new AbortController().signal,
              },);
            }
            catch (error) {
              caught = error;
            }
            // The stall is now the CAUSE of a cut rather than the thrown value
            // itself, because the drain also has to hand back whatever the stream
            // delivered before it stopped. The distinction this case exists for is
            // unchanged: a stall must still be identifiable as a stall rather than
            // as the platform abort that carried it out.
            expect(caught,).toBeInstanceOf(StreamCutShortError,);
            expect((caught as StreamCutShortError).cause,).toBeInstanceOf(StreamStalledError,);
          },
        },),

        it({
          name: 'reports caller steering as itself, not as a stall',
          fn: async () => {
            using guard = armIdleGuard({
              label: 'hf:whiskers',
              firstContentMs: HANG_STOP_MS,
              idleMs: HANG_STOP_MS,
            },);

            /**
             Caller's own controller, aborted mid-read to imitate steering.
             */
            const caller = new AbortController();

            /**
             Failure the caller's abort raises, distinct from a stall.
             */
            const steering = new Error('caller stopped the run',);

            const steered = new Response(
              new ReadableStream({
                start(controller,) {
                  caller.signal
                    .addEventListener(
                    'abort',
                    function tearDown() {
                      controller.error(steering,);
                    },
                    { once: true, },
                    );
                },
              },),
            );
            caller.abort(steering,);

            let caught: unknown;
            try {
              await drainBody({
                credentials: [],
                response: steered,
                guard,
                label: 'hf:whiskers',
                callerSignal: caller.signal,
              },);
            }
            catch (error) {
              caught = error;
            }
            // THE STEERING IS THE CAUSE, BY IDENTITY, and the cut's message
            // names it by class alone: the message is repeated at every
            // refusal printer, and a caller's abort reason is any value at
            // all, so its own words stay on the cause.
            expect(caught,).toBeInstanceOf(StreamCutShortError,);
            expect((caught as StreamCutShortError).cause,).toBe(steering,);
            expect(String(caught,),).toBe(
              'StreamCutShortError: hf:whiskers: stream cut after 0 characters (refused by Error)',
            );
          },
        },),
      ],
    },),
  ],
},);
