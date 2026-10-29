/**
 Tests the sliding-window request pacer.

 THE CASE IS XIEPT2 ON HYPER ALONE, 2026-09-03: 1,000 requests in a rolling
 hour is the account's limit, the pass spent them in minutes, every refusal
 retried four more times, the run lost. Here the pacer lets a window's worth
 start at once, makes the next wait for the oldest start to leave the window,
 keeps takes in arrival order, and lets an abort end a wait.

 @module
 */

import { once, } from 'node:events';

import { wait as pause, } from '@monochromatic-dev/async-time-core/ts';
import {
  caught,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { StatedRefusalError, } from '@monochromatic-dev/translation-repair-glossary';
import {
  createRequestPace,
  HYPER_PACE_WINDOW_MS,
  HYPER_REQUESTS_PER_HOUR,
  hyperRequestsPerHour,
} from '../dist/final/node/index.mjs';

import {
  HOUR_MS,
  stubWallClock,
  WALL_START_MS,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/wall-clock-stub.test-fixture.ts';

/**
 Abort signal that never fires.
 */
const SIGNAL = new AbortController().signal;

/**
 Window length the scripted pacers use.
 */
const WINDOW_MS = 60_000;

/**
 Window length for the pacer on the real clock: short, since a case sleeps
 through it once.
 */
const REAL_WINDOW_MS = 20;

/**
 Builds a pacer on a scripted clock whose sleeps advance the clock instead of
 waiting.

 @param perWindow - starts allowed per window

 @returns Pacer plus the clock and the sleeps it asked for

 @example
 ```ts
 const { pace, clock, sleeps, } = scriptedPace({ perWindow: 3, },);
 ```
 */
function scriptedPace(
  { perWindow, }: { readonly perWindow: number; },
): {
  readonly pace: ReturnType<typeof createRequestPace>;
  readonly clock: { now: number; };
  readonly sleeps: number[];
} {
  /**
   Scripted clock.
   */
  const clock = { now: 1_000_000, };
  /**
   Sleeps asked for, in order.
   */
  const sleeps: number[] = [];
  return {
    clock,
    sleeps,
    pace: createRequestPace({
      perWindow,
      windowMs: WINDOW_MS,
      now: () => clock.now,
      wait: async function wait({ ms, },): Promise<void> {
        sleeps.push(ms,);
        clock.now += ms;
      },
    },),
  };
}

/**
 How long the release case gives an abandoned take to end before calling it
 stuck behind another take's wait.
 */
const RELEASE_PATIENCE_MS = 200;

/**
 Sleeper that ends only when its caller gives up, standing for a timer far
 past the case's patience.

 @param signal - caller's abort, the only way out; the wait asked for is never
 reached here

 @example
 ```ts
 const pace = createRequestPace({ perWindow: 1, windowMs: WINDOW_MS, wait: untilAborted, },);
 ```
 */
async function untilAborted({ signal, }: { readonly ms: number; readonly signal?: AbortSignal; },): Promise<void> {
  await once(
    signal ?? SIGNAL,
    'abort',
  );
}

/**
 Resolves once the release case's patience has run out.

 @returns Marker saying the take was still waiting

 @example
 ```ts
 const outcome = await Promise.race([Promise.allSettled([take,],), patienceRunsOut(),],);
 ```
 */
async function patienceRunsOut(): Promise<'still waiting'> {
  await pause(RELEASE_PATIENCE_MS,);
  return 'still waiting';
}

await describe({
  name: '',
  children: [
    describe({
      name: createRequestPace.name,
      children: [
        it({
          name: 'LETS a window\'s worth of requests start at once and MAKES the next wait until the oldest '
            + 'start leaves the window',
          fn: async () => {
            const { pace, clock, sleeps, } = scriptedPace({ perWindow: 3, },);
            await pace.take({ signal: SIGNAL, },);
            clock.now += 10_000;
            await pace.take({ signal: SIGNAL, },);
            await pace.take({ signal: SIGNAL, },);
            expect(sleeps,).toEqual([],);
            expect(pace.inWindow(),).toBe(3,);
            // The fourth waits for the first start (at 1,000,000) to leave the window.
            await pace.take({ signal: SIGNAL, },);
            expect(sleeps,).toEqual([WINDOW_MS - 10_000,],);
            expect(pace.inWindow(),).toBe(3,);
          },
        },),

        it({
          name: 'STARTS no concurrent take before one that arrived earlier, and counts each start once',
          fn: async () => {
            const { pace, clock, sleeps, } = scriptedPace({ perWindow: 2, },);
            /**
             When each take, by arrival, was let start.
             */
            const startedAt: number[] = [];
            await Promise.all([0, 1, 2, 3,].map(async function taker(index,): Promise<void> {
              await pace.take({ signal: SIGNAL, },);
              startedAt[index] = clock.now;
            },),);
            // Takes that start together may resolve in either order; what the
            // window owes arrival order is that no later arrival starts sooner.
            expect(startedAt.every(function inOrder(at, index,): boolean {
              return (index === 0) || (at >= (startedAt[index - 1] ?? at));
            },),).toBe(true,);
            // The third waits a whole window, which empties it; the fourth then
            // finds a free place beside the third and does not wait.
            expect(sleeps,).toEqual([WINDOW_MS,],);
            expect(pace.inWindow(),).toBe(2,);
          },
        },),

        it({
          name: 'WAITS FOR THE START A WHOLE PLACE BACK: with starts at 0 and 600 ms in a window of 1,000 ms and '
            + 'two places, 700 ms reads 300 ms of wait',
          fn: async () => {
            /**
             Scripted clock, moved by hand.
             */
            const clock = { now: 0, };
            const pace = createRequestPace({
              perWindow: 2,
              windowMs: 1_000,
              now: () => clock.now,
              wait: async function wait({ ms, },): Promise<void> {
                clock.now += ms;
              },
            },);
            await pace.take({ signal: SIGNAL, },);
            clock.now = 600;
            await pace.take({ signal: SIGNAL, },);
            clock.now = 700;
            expect(pace.waitMs(),).toBe(300,);
          },
        },),

        it({
          name: 'REFUSES A PLACE COUNT THAT IS NOT A WHOLE NUMBER OF ONE OR MORE with a RangeError naming the '
            + 'count, where 1.5 places read the start list at a fraction and made a take at 700 ms wait 1,000 ms',
          fn: async () => {
            /**
             Counts a window cannot hold, each as the refusal writes it.
             */
            const refused: readonly [number, string,][] = [
              [1.5, '1.5',],
              [0, '0',],
              [-1, '-1',],
              [Number.NaN, 'NaN',],
              [Number.POSITIVE_INFINITY, 'Infinity',],
              [2 ** 60, String(2 ** 60,),],
            ];
            expect(refused.map(function refusalOf([perWindow,],): string {
              return String(caught(function builds(): unknown {
                return createRequestPace({
                  perWindow,
                  windowMs: 1_000,
                },);
              },),);
            },),).toEqual(refused.map(function expected([, written,],): string {
              return `RangeError: perWindow must be a whole number of one or more, and ${written} is not`;
            },),);
            expect(caught(function builds(): unknown {
              return createRequestPace({
                perWindow: 1.5,
                windowMs: 1_000,
              },);
            },),).toBeInstanceOf(RangeError,);
          },
        },),

        it({
          name: 'ENDS a wait with the abort reason when the caller gives up, REFUSES a place to a caller '
            + 'that gave up while queued',
          fn: async () => {
            /**
             Aborts the caller from inside its own sleep.
             */
            const aborter = new AbortController();
            const pace = createRequestPace({
              perWindow: 1,
              windowMs: WINDOW_MS,
              now: () => 5,
              wait: async function wait(): Promise<void> {
                aborter.abort(new Error('caller gave up',),);
              },
            },);
            await pace.take({ signal: aborter.signal, },);
            /**
             What the second take threw.
             */
            let thrown: unknown;
            try {
              await pace.take({ signal: aborter.signal, },);
            } catch (error) {
              thrown = error;
            }
            expect((thrown as Error).message,).toBe('caller gave up',);

            // A caller that aborts while queued behind a sleeping take gets no
            // place when its turn comes: the sleeping take's own wait is where the
            // queued caller gives up, so the abort lands after take() accepted it.
            const gaveUp = new AbortController();
            /**
             Scripted clock for the queued pacer.
             */
            const clock = { now: 1_000_000, };
            /**
             Sleeps the queued pacer asked for.
             */
            const sleeps: number[] = [];
            const queued = createRequestPace({
              perWindow: 1,
              windowMs: WINDOW_MS,
              now: () => clock.now,
              wait: async function wait({ ms, },): Promise<void> {
                sleeps.push(ms,);
                gaveUp.abort(new Error('abandoned in the queue',),);
                clock.now += ms;
              },
            },);
            await queued.take({ signal: SIGNAL, },);
            /**
             Outcomes of a live take, which sleeps, and one queued behind it
             that is abandoned during that sleep.
             */
            const outcomes = await Promise.allSettled([
              queued.take({ signal: SIGNAL, },),
              queued.take({ signal: gaveUp.signal, },),
            ],);
            expect(outcomes.map((outcome,) => outcome.status,),).toEqual(['fulfilled', 'rejected',],);
            // Without the check the abandoned caller would sleep a second window
            // and take a place of its own.
            expect(sleeps,).toEqual([WINDOW_MS,],);
            expect(queued.inWindow(),).toBe(1,);
          },
        },),

        it({
          name: 'SLEEPS ON THE REAL CLOCK WITH THE DEFAULT SLEEPER until the window has room, with the '
            + 'pacer\'s clock held still until the second take has started its sleep, so no stall can '
            + 'empty the window first and let the case pass without sleeping (ledger T5)',
          fn: async () => {
            /**
             Pacer's clock: still at zero while held, then real time since its
             release.
             */
            const clock = {
              held: true,
              releasedAt: 0,
            };
            const pace = createRequestPace({
              perWindow: 1,
              windowMs: REAL_WINDOW_MS,
              now: () => (clock.held ? 0 : performance.now() - clock.releasedAt),
            },);
            await pace.take({ signal: SIGNAL, },);

            // A TAKE RESERVES AND STARTS ITS SLEEP BEFORE IT FIRST AWAITS, so the
            // second one reads the held clock, finds the window full, and is
            // asleep on the default sleeper for the whole window before the clock
            // moves. On the real clock alone, a run stalled past the window
            // between the two takes found room, never slept, and still passed.
            /**
             The second take, asleep on the default sleeper.
             */
            const asleep = pace.take({ signal: SIGNAL, },);
            clock.releasedAt = performance.now();
            clock.held = false;
            await asleep;

            // ONE-SIDED, so load cannot fail it: the take returns only once the
            // pacer's clock has passed the window, and that clock is this one.
            expect(performance.now() - clock.releasedAt,).toBeGreaterThanOrEqual(REAL_WINDOW_MS,);
          },
        },),

        it({
          name: 'ENDS A REAL-CLOCK SLEEP ON ABORT with the abort reason (ledger T8)',
          fn: async () => {
            // A WINDOW NO STALL CAN OUTLAST, so the second take is certainly
            // asleep when the abort lands. On a window of 20 ms a loaded suite
            // run let the window empty first, and the take resolved (2026-09-29).
            const pace = createRequestPace({
              perWindow: 1,
              windowMs: WINDOW_MS,
            },);
            await pace.take({ signal: SIGNAL, },);

            /**
             A caller that gives up while the default sleeper holds it.
             */
            const aborter = new AbortController();
            /**
             Its take, asleep until the window has room.
             */
            const asleep = pace.take({ signal: aborter.signal, },);
            aborter.abort(new Error('the cat left the queue',),);
            await expect(asleep,).rejects.toThrow('the cat left the queue',);
          },
        },),
      ],
    },),

    describe({
      name: hyperRequestsPerHour.name,
      children: [
        it({
          name: 'READS a whole number from the variable and takes the account limit when it is unset',
          fn: async () => {
            expect(hyperRequestsPerHour({ env: { TRANSLATION_REPAIR_HYPER_REQUESTS_PER_HOUR: '300', }, },),).toBe(300,);
            expect(hyperRequestsPerHour({ env: {}, },),).toBe(HYPER_REQUESTS_PER_HOUR,);
            expect(HYPER_REQUESTS_PER_HOUR,).toBe(1_000,);
            expect(HYPER_PACE_WINDOW_MS,).toBe(3_600_000,);
          },
        },),
        it({
          name: 'REFUSES A RATE THAT IS NOT A WHOLE NUMBER OF ONE OR MORE, naming the variable, the value as written '
            + 'and what is accepted, as every other dial does (ledger D14): a fraction such as 1.5 read the pacer\'s '
            + 'start list at a fraction, and a mistyped rate that fell back to the account limit ran the launch at a '
            + 'pace nobody asked for',
          fn: async () => {
            /**
             Texts that are no whole number of one or more.
             */
            const written = ['lots', '0', '-5', '-1', '1.5', '0.5', 'NaN',];
            expect(written.map(function refusalOf(text,): string {
              return String(caught(function readsRate() {
                return hyperRequestsPerHour({ env: { TRANSLATION_REPAIR_HYPER_REQUESTS_PER_HOUR: text, }, },);
              },),);
            },),).toEqual(written.map(function expected(text,): string {
              return 'StatedRefusalError: TRANSLATION_REPAIR_HYPER_REQUESTS_PER_HOUR must be one or more requests '
                + 'per hour, as a whole number written in digits, at most '
                + `${String(Number.MAX_SAFE_INTEGER,)}, and ${JSON.stringify(text,)} is not; leave it unset for the `
                + 'account limit of 1000';
            },),);
            expect(caught(function readsRate() {
              return hyperRequestsPerHour({ env: { TRANSLATION_REPAIR_HYPER_REQUESTS_PER_HOUR: '1.5', }, },);
            },),).toBeInstanceOf(StatedRefusalError,);
          },
        },),
        it({
          name: 'TAKES THE ACCOUNT LIMIT for a variable set to blanks, as the grace, cap, spend ceiling and credit '
            + 'overrides do, where it read the blanks as a rate of zero and refused (ledger B73)',
          fn: async () => {
            expect(hyperRequestsPerHour({ env: { TRANSLATION_REPAIR_HYPER_REQUESTS_PER_HOUR: '  ', }, },),)
              .toBe(HYPER_REQUESTS_PER_HOUR,);
          },
        },),
        it({
          name: 'REFUSES A RATE NOT WRITTEN IN PLAIN DIGITS, which `Number` read as a pace nobody typed: a '
            + 'hexadecimal, an exponent, a sign, a space either side and a point missing digits on one side '
            + '(ledger B73)',
          fn: async () => {
            /**
             Spellings `Number` reads as a rate that no operator writes as one.
             */
            const spellings = [
              '0x10',
              '1e1',
              '+15',
              ' 15',
              '15 ',
              '15.',
              '.5',
            ];
            expect(spellings.map(function refusedOf(written,): boolean {
              return caught(function readSpelling(): number {
                return hyperRequestsPerHour({ env: { TRANSLATION_REPAIR_HYPER_REQUESTS_PER_HOUR: written, }, },);
              },) instanceof StatedRefusalError;
            },),).toEqual(spellings.map(function refused(): boolean {
              return true;
            },),);
          },
        },),
      ],
    },),

    describe({
      name: 'a full window never holds one caller behind another caller\'s wait (class one hundred forty-nine, hulicaijia30)',
      children: [
        it({
          name: 'RELEASES a caller that gives up while another take waits for the window, at once and with its own '
            + 'reason, and leaves the window holding only the start that happened',
          fn: async () => {
            const pace = createRequestPace({
              perWindow: 1,
              windowMs: WINDOW_MS,
              now: () => 0,
              wait: untilAborted,
            },);
            await pace.take({ signal: SIGNAL, },);
            const first = new AbortController();
            const second = new AbortController();
            const firstTake = pace.take({ signal: first.signal, },);
            const secondTake = pace.take({ signal: second.signal, },);
            second.abort(new Error('the second cat left the queue',),);
            /**
             The second take's outcome, or the marker when it was still waiting.
             */
            const outcome = await Promise.race([
              Promise.allSettled([secondTake,],),
              patienceRunsOut(),
            ],);
            expect(Array.isArray(outcome,),).toBe(true,);
            const [secondSettled,] = outcome as PromiseSettledResult<void>[];
            expect(secondSettled?.status,).toBe('rejected',);
            expect(((secondSettled as PromiseRejectedResult).reason as Error).message,).toBe('the second cat left the queue',);

            first.abort(new Error('the first cat left too',),);
            const [firstSettled,] = await Promise.allSettled([firstTake,],);
            expect(firstSettled.status,).toBe('rejected',);
            expect(pace.inWindow(),).toBe(1,);
          },
        },),
        it({
          name: 'SAYS how long a take would wait now: zero while the window has room, until the oldest start '
            + 'leaves it once full',
          fn: async () => {
            const { pace, clock, } = scriptedPace({ perWindow: 1, },);
            expect(pace.waitMs(),).toBe(0,);
            await pace.take({ signal: SIGNAL, },);
            clock.now += 10_000;
            expect(pace.waitMs(),).toBe(WINDOW_MS - 10_000,);
            clock.now += WINDOW_MS;
            expect(pace.waitMs(),).toBe(0,);
          },
        },),
      ],
    },),

    describe({
      name: 'the pacer on a clock the system time cannot move (ledger B78)',
      children: [
        it({
          name: 'ASKS NO LONGER THAN THE WINDOW when the system clock is set back an hour after a take, where the '
            + 'default pacer, reading the wall clock, made the next take wait an hour and a window',
          fn: async ctx => {
            const wall = stubWallClock({ sinon: ctx.sinon, atMs: WALL_START_MS, },);
            const pace = createRequestPace({
              perWindow: 1,
              windowMs: WINDOW_MS,
            },);
            await pace.take({ signal: SIGNAL, },);
            wall.step({ byMs: -HOUR_MS, },);
            expect(pace.waitMs(),).toBeLessThanOrEqual(WINDOW_MS,);
          },
        },),
        it({
          name: 'SLEEPS AGAIN WHEN A SLEEP ENDS BEFORE THE RESERVED START, as a timer read against a finer clock '
            + 'can end a millisecond early, where the take returned and its request started before its place opened',
          fn: async () => {
            /**
             Scripted clock.
             */
            const clock = { now: 1_000_000, };
            /**
             Sleeps asked for, in order.
             */
            const sleeps: number[] = [];
            const pace = createRequestPace({
              perWindow: 1,
              windowMs: WINDOW_MS,
              now: () => clock.now,
              wait: async function shortFirst({ ms, },): Promise<void> {
                sleeps.push(ms,);
                clock.now += (sleeps.length === 1) ? ms - 1 : ms;
              },
            },);
            await pace.take({ signal: SIGNAL, },);
            await pace.take({ signal: SIGNAL, },);
            expect(clock.now,).toBe(1_000_000 + WINDOW_MS,);
            expect(sleeps,).toEqual([WINDOW_MS, 1,],);
          },
        },),
      ],
    },),
  ],
},);
