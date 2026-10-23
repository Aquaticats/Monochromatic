/**
 Tests that the fake clock walk puts every kind of wait on the fake clock, including a wait
 built on `node:timers/promises`.

 The rig faked the global `setTimeout`, `clearTimeout` and `AbortSignal.timeout`, and a caller cut written with
 `import { setTimeout } from 'node:timers/promises'` was on none of them: its named import is bound once, and the
 sinon clock replaces the property of the CommonJS module object only, so the cut never fired and a case that expected
 it to be heard stayed green. The rig now syncs the built-in module's ES exports after it installs the clock and again
 when the walk ends. Each case here is a wait of two minutes that a tick past two minutes ends, so a wait on the real
 clock cannot end in the case's own time.

 Fixtures are cat-themed invention.

 @module
 */

import { setTimeout as sleepFor, } from 'node:timers/promises';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import { walkOnFakeClock, } from './fake-clock-walk.test-fixture.ts';

/**
 The wait this file's module was loaded with, which is the real one: no walk has run when the module is evaluated.
 */
const REAL_SLEEP_FOR = sleepFor;

/**
 How long the cat naps in each case, in fake milliseconds.
 */
const NAP_MS = 120_000;

/**
 A tick that ends the nap.
 */
const PAST_NAP_MS = 121_000;

/**
 The real milliseconds the rig is given to stop waiting for a walk that does not end after its release.
 */
const SHORT_HANG_STOP_MS = 200;

/**
 The real milliseconds after which the case calls the rig hung, far past {@link SHORT_HANG_STOP_MS}.
 */
const HUNG_RIG_BOUND_MS = 5_000;

/**
 Walks one nap on the fake clock.

 @param sinon - case's own sandbox
 @param nap - how the walk waits, given the signal that ends it early
 @param ticks - fake milliseconds to advance, one step after another

 @returns Whether the nap ended by itself

 @example
 ```ts
 const ended = await napWalk({
   sinon,
   nap: signal => sleepFor(NAP_MS, undefined, { signal, },),
   ticks: [PAST_NAP_MS,],
 },);
 ```
 */
async function napWalk(
  {
    sinon,
    nap,
    ticks,
  }: {
    readonly sinon: Parameters<typeof walkOnFakeClock>[0]['sinon'];
    readonly nap: (signal: AbortSignal,) => Promise<unknown>;
    readonly ticks: readonly number[];
  },
): Promise<boolean> {
  /**
   Ends the nap when the walk is released, so a nap on a clock the rig does not own cannot outlive the case.
   */
  const stop = new AbortController();
  const walk = await walkOnFakeClock({
    sinon,
    walkOnce: async function napOnce(): Promise<void> {
      try {
        await nap(stop.signal,);
      }
      catch (error) {
        // Only the release's own abort is the walk ending early; anything else is the case failing.
        if (!stop.signal.aborted)
          throw error;
      }
    },
    firstCall: Promise.resolve(),
    release: () => stop.abort(),
    ticks,
    cutCount: () => 0,
    waitForEnd: false,
  },);
  return walk.endedByItself;
}

await describe({
  name: walkOnFakeClock.name,
  // One case at a time: each installs a clock, and the built-in module's exports are one object for the whole process.
  concurrency: 1,
  children: [
    it({
      name: 'ENDS A WAIT BUILT ON THE GLOBAL setTimeout AT ITS FIGURE (the control arm)',
      fn: async ctx => {
        expect(await napWalk({
          sinon: ctx.sinon,
          nap: signal => new Promise<void>(function napping(resolve,): void {
            setTimeout(
              resolve,
              NAP_MS,
            );
            signal.addEventListener(
              'abort',
              () => resolve(),
            );
          },),
          ticks: [PAST_NAP_MS,],
        },),).toBe(true,);
      },
    },),

    it({
      name: 'ENDS A WAIT BUILT ON node:timers/promises AT ITS FIGURE: a cut written with that import was on the '
        + 'real clock, so the heard case stayed green however the cut was written',
      fn: async ctx => {
        expect(await napWalk({
          sinon: ctx.sinon,
          nap: signal => sleepFor(
            NAP_MS,
            undefined,
            { signal, },
          ),
          ticks: [PAST_NAP_MS,],
        },),).toBe(true,);
      },
    },),

    it({
      name: 'LEAVES A node:timers/promises WAIT UNENDED BEFORE ITS FIGURE (control arm): the rig advances that '
        + 'clock and no further, so the end of a nap on the fake clock is the clock\'s doing and not the release\'s',
      fn: async ctx => {
        expect(await napWalk({
          sinon: ctx.sinon,
          nap: signal => sleepFor(
            NAP_MS,
            undefined,
            { signal, },
          ),
          ticks: [60_000,],
        },),).toBe(false,);
      },
    },),

    it({
      name: 'PUTS node:timers/promises BACK WHEN THE WALK ENDS: during the walk the wait is the rig\'s, and when '
        + 'the walk has returned it is the real one again, so a wait in another case is real, not a wait on a '
        + 'clock whose case has finished (asserted inside the walking case, so it does not depend on any other '
        + 'case having walked)',
      fn: async ctx => {
        /**
         Whether the wait the walk's nap sees is the real one, read while the walk runs.
         */
        const seen: boolean[] = [];
        await napWalk({
          sinon: ctx.sinon,
          nap: (signal) => {
            seen.push(sleepFor === REAL_SLEEP_FOR,);
            return sleepFor(
              NAP_MS,
              undefined,
              { signal, },
            );
          },
          ticks: [PAST_NAP_MS,],
        },);

        expect(seen,).toEqual([false,],);
        expect(sleepFor,).toBe(REAL_SLEEP_FOR,);
      },
    },),

    it({
      name: 'FAILS THE CASE WHEN THE WALK DOES NOT END AFTER THE RELEASE: a sleep the release provokes is on the '
        + 'fake clock, which nobody ticks again, so the rig stops waiting at its hang stop and says so, where it '
        + 'waited for ever',
      fn: async ctx => {
        /**
         Ends the nap loop when the walk is released.
         */
        const stop = new AbortController();

        /**
         Ends the real wait that stands for a hung rig once the outcome is known.
         */
        const known = new AbortController();

        /**
         What the rig does with a walk that sleeps again after its release: the error it throws, or that it ended.
         */
        async function walkOutcome(): Promise<unknown> {
          try {
            await walkOnFakeClock({
              sinon: ctx.sinon,
              walkOnce: async function sleepsAfterTheRelease(): Promise<void> {
                await new Promise<void>(function released(resolve,): void {
                  stop.signal.addEventListener(
                    'abort',
                    () => resolve(),
                  );
                },);
                await sleepFor(NAP_MS,);
              },
              firstCall: Promise.resolve(),
              release: () => stop.abort(),
              ticks: [1,],
              cutCount: () => 0,
              waitForEnd: false,
              hangStopMs: SHORT_HANG_STOP_MS,
            },);
            return 'ended';
          }
          catch (error) {
            return error;
          }
        }

        /**
         The real wait after which the rig is called hung, which a rig that stops by itself never reaches.
         */
        async function hungAfterTheBound(): Promise<string> {
          try {
            await REAL_SLEEP_FOR(
              HUNG_RIG_BOUND_MS,
              undefined,
              { signal: known.signal, },
            );
          }
          catch (error) {
            // The outcome arrived first and ended this wait, which is the wait's purpose.
            // Anything else is the case failing.
            if (!(known.signal.aborted && Error.isError(error,) && (error.name === 'AbortError')))
              throw error;
          }
          return 'hung';
        }

        const outcome = await Promise.race([walkOutcome(), hungAfterTheBound(),],);
        known.abort();

        expect(outcome,).toBeInstanceOf(Error,);
        expect(String(outcome,),).toContain('did not end after the release',);
      },
    },),
  ],
},);
