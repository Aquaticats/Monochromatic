/**
 Controls for the bound every command the unchanged-bundle builder runs gets.

 The stand-in command never returns on its own,
 so a runner without a bound leaves the proof waiting on it,
 which is what the first case measures against the control's own watchdog.
 The second case is the positive control:
 a command that finishes still finishes.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { setTimeout as sleepFor, } from 'node:timers/promises';
import { runBoundedCommand, } from '../dist/final/node/index.mjs';

//region Fixtures

/**
 Bound the stand-in command is given,
 the same order as the builder's own bound against the work it does.
 */
const CONTROL_BOUND_MILLISECONDS = 300;

/**
 The control's own bound:
 past this the runner has not settled, which is the failure this case exists
 for.
 */
const CONTROL_WATCHDOG_MILLISECONDS = 3_000;

/**
 A command that never returns on its own,
 written so it also ends by itself if nothing ever kills it,
 which leaves no process behind after a failing run either.
 */
const STAND_IN_THAT_NEVER_RETURNS = 'setTimeout(function () {}, 8000);';

/**
 A command that finishes and says so.
 */
const STAND_IN_THAT_FINISHES = String.raw`process.stdout.write("ok\n");`;

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

//endregion Fixtures

//region The bound

await describe({
  name: 'bounded builder command',
  children: [
    it({
      name: 'ends a command that never returns at its bound, naming the command and the bound',
      fn: async () => {
        /**
         When this case started, so the bound it measures can be compared with
         the wait it actually took.
         */
        const startedAt = Date.now();
        /**
         What the runner did before the control's own watchdog fired,
         absent when it did nothing.
         */
        const outcome = await Promise.race([
          settledOrFailed({
            promise: runBoundedCommand({
              file: process.execPath,
              args: ['--eval', STAND_IN_THAT_NEVER_RETURNS,],
              boundMilliseconds: CONTROL_BOUND_MILLISECONDS,
            },),
          },),
          sleepFor(CONTROL_WATCHDOG_MILLISECONDS,),
        ],);
        if (outcome === undefined)
          throw new Error(
            `the bounded command runner did not settle within ${String(CONTROL_WATCHDOG_MILLISECONDS,)} ms, so a command that never returns holds the proof`,
          );
        /**
         How long the bound took to end the command.
         */
        const elapsedMilliseconds = Date.now() - startedAt;
        expect(elapsedMilliseconds,).toBeLessThan(CONTROL_WATCHDOG_MILLISECONDS,);
        expect(outcome.error,).toBeInstanceOf(Error,);
        if (!(Error.isError(outcome.error,)))
          throw new Error('the runner resolved a command that never returned',);
        expect(outcome.error.message,).toContain(
          `did not finish within ${String(CONTROL_BOUND_MILLISECONDS,)} ms`,
        );
        expect(outcome.error.message,).toContain('SIGKILL',);
        expect(outcome.error.message,).toContain(process.execPath,);
      },
    },),
    it({
      name: 'lets a command that finishes finish',
      fn: async () => {
        /**
         What the runner resolved the stand-in command with.
         */
        const outcome = await settledOrFailed({
          promise: runBoundedCommand({
            file: process.execPath,
            args: ['--eval', STAND_IN_THAT_FINISHES,],
            boundMilliseconds: CONTROL_WATCHDOG_MILLISECONDS,
          },),
        },);
        expect(outcome.error,).toBe(undefined,);
        expect(outcome.value?.stdout,).toBe('ok\n',);
      },
    },),
  ],
},);

//endregion The bound
