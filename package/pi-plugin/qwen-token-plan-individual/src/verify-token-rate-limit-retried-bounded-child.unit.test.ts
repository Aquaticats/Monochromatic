/**
 Controls for the bound one child of the proof gets, and for the way a child
 that survives the bound's own signal is ended.

 The stand-in ignores the first `SIGTERM` exactly as pi does
 (pi force-exits only on a second signal it counts),
 so a run bound that sends one signal and stops there leaves it alive,
 and the orphan's open pipes keep this process from ending.
 The second case is the positive control:
 a child that ends by itself still reports its exit status and its output.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { setTimeout as sleepFor, } from 'node:timers/promises';
import { runBoundedChild, } from '../dist/final/node/index.mjs';

//region Fixtures

/**
 Bound the stand-in child is given.
 */
const CONTROL_BOUND_MILLISECONDS = 300;

/**
 The control's own bound:
 past this the runner has not settled, which is the failure these cases exist
 for.
 */
const CONTROL_WATCHDOG_MILLISECONDS = 10_000;

/**
 A stand-in for pi:
 it says it started,
 it ignores the first signal,
 it exits on the second one as pi does,
 and it ends by itself if nothing ever signals it twice,
 which leaves no process behind after a failing run either.
 */
const STAND_IN_THAT_IGNORES_THE_FIRST_SIGNAL =
  String.raw`process.stdout.write("READY\n"); let signals = 0; process.on("SIGTERM", function () { signals += 1; if (signals > 1) process.exit(143); }); setTimeout(function () { process.exit(0); }, 10000);`;

/**
 A stand-in for a child that finishes by itself.
 */
const STAND_IN_THAT_FINISHES = String.raw`process.stdout.write("DONE\n");`;

/**
 What one settled promise left behind: its value or the error it failed with.
 */
type SettledOutcome<T> = {
  /**
   Value it resolved with.
   */
  readonly value?: T;
  /**
   Error it rejected with.
   */
  readonly error?: unknown;
};

/**
 Settle one promise into a value or an error, so a race against the control's
 watchdog can tell the two apart.

 @param promise - promise under control

 @returns the settled value, or the error it failed with

 @example
 ```typescript
 const outcome = await settledOrFailed({ promise });
 ```
 */
async function settledOrFailed<T>({
  promise,
}: {
  readonly promise: Promise<T>;
},): Promise<SettledOutcome<T>> {
  try {
    return { value: await promise, };
  } catch (error) {
    return { error, };
  }
}

/**
 Whether a process id still names a running process.

 @param pid - process id the run reported

 @returns `true` when something still answers to it

 @example
 ```typescript
 if (isRunning({ pid })) {
   // the run outlived its bound
 }
 ```
 */
function isRunning({
  pid,
}: {
  readonly pid: number;
},): boolean {
  try {
    // Signal 0 asks the kernel whether the process exists without signalling it.
    process.kill(
      pid,
      0,
    );
    return true;
  } catch (error) {
    // ESRCH is the kernel answering that no such process exists; anything else
    // is a question this control cannot answer, so it is raised rather than
    // guessed at.
    if ((Error.isError(error,)) && ('code' in error) && (error.code === 'ESRCH'))
      return false;
    throw error;
  }
}

//endregion Fixtures

//region The bound

await describe({
  name: 'bounded child run',
  children: [
    it({
      name: 'ends a child that ignores the first signal, and leaves nothing running',
      fn: async () => {
        /**
         When this case started, so the wait it took can be compared with the
         control's own bound.
         */
        const startedAt = Date.now();
        /**
         What the runner did before the control's own watchdog fired,
         absent when it did nothing.
         */
        const outcome = await Promise.race([
          settledOrFailed({
            promise: runBoundedChild({
              file: process.execPath,
              args: ['--eval', STAND_IN_THAT_IGNORES_THE_FIRST_SIGNAL,],
              cwd: process.cwd(),
              env: { PATH: process.env.PATH ?? '', },
              boundMilliseconds: CONTROL_BOUND_MILLISECONDS,
            },),
          },),
          sleepFor(CONTROL_WATCHDOG_MILLISECONDS,),
        ],);
        if (outcome === undefined)
          throw new Error(
            `the bounded child run did not settle within ${String(CONTROL_WATCHDOG_MILLISECONDS,)} ms, so a child that ignores its bound holds the proof`,
          );
        expect(outcome.error,).toBe(undefined,);
        /**
         What the stand-in printed before the bound ended it,
         and whether the run reported the bound.
         */
        const result = outcome.value;
        expect(result?.timedOut,).toBe(true,);
        expect(result?.stdout,).toContain('READY',);
        expect(Date.now() - startedAt,).toBeLessThan(CONTROL_WATCHDOG_MILLISECONDS,);
        expect(result?.pid,).not.toBe(undefined,);
        if (result?.pid === undefined)
          throw new Error('the run reported no process id, so the control cannot say the child ended',);
        expect(isRunning({ pid: result.pid, },),).toBe(false,);
      },
    },),
    it({
      name: 'lets a child that ends by itself end, with its exit status and output',
      fn: async () => {
        /**
         What the runner resolved the stand-in child with.
         */
        const outcome = await settledOrFailed({
          promise: runBoundedChild({
            file: process.execPath,
            args: ['--eval', STAND_IN_THAT_FINISHES,],
            cwd: process.cwd(),
            env: { PATH: process.env.PATH ?? '', },
            boundMilliseconds: CONTROL_WATCHDOG_MILLISECONDS,
          },),
        },);
        expect(outcome.error,).toBe(undefined,);
        expect(outcome.value?.timedOut,).toBe(false,);
        expect(outcome.value?.exitCode,).toBe(0,);
        expect(outcome.value?.stdout,).toBe('DONE\n',);
      },
    },),
  ],
},);

//endregion The bound
