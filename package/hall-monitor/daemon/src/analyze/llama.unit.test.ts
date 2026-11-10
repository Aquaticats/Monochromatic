/**
 Exit-wait tests for the llama-server lifecycle.

 `stop` ends the server and then waits for its child to exit, and the child can
 already have exited by then: `pkill` may end the distrobox wrapper during the
 `await` before the wait starts, and Node emits `exit` exactly once, so a
 listener attached afterwards never runs and the daemon's cycle hangs forever.
 `awaitChildEnd` is the wait, and these cases prove it settles in every order the
 child can end in: already exited, exiting later, and never started at all.

 Every case uses an interpreter this test controls, so no case starts
 llama-server, needs a GPU, or signals anything.

 @module
 */

import { spawn as cpSpawn, } from 'node:child_process';
import { once, } from 'node:events';
import { setTimeout as delay, } from 'node:timers/promises';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { awaitChildEnd, } from './llama.ts';

//region Fixtures

/**
 Interpreter the cases spawn, so each child is a program this test decides on.
 */
const COMMAND = process.execPath;

/**
 Longest a wait may take before the case calls it hung.

 Every case spawns a process that ends in well under this, so reaching it means
 the wait never settled, which is the defect these cases cover.
 */
const HANG_DEADLINE_MS = 5_000;

/**
 How long the later-exit case's child stays alive, so the wait has a child to
 wait for.
 */
const CHILD_LIFETIME_MS = 100;

/**
 Binary that cannot exist, used for the case where the child never starts.
 */
const MISSING_COMMAND = '/nonexistent/hall-monitor-test-child';

/**
 Outcome of racing a wait against {@link HANG_DEADLINE_MS}.
 */
const SETTLED = 'settled';

/**
 Outcome of racing a wait against {@link HANG_DEADLINE_MS} when it never
 resolved.
 */
const HUNG = 'hung';

/**
 Races one wait against the hang deadline, so a wait that never settles fails
 the case instead of blocking the suite.

 @param wait - the wait under test

 @returns {@link SETTLED} when the wait resolved, {@link HUNG} when the deadline
   fired first

 @example
 ```ts
 expect(await settledWithinDeadline({ wait: awaitChildEnd({ child }) })).toBe(SETTLED);
 ```
 */
async function settledWithinDeadline({ wait, }: Readonly<{ wait: Promise<void>; }>,): Promise<string> {
  return Promise.race([
    wait.then(() => SETTLED,),
    delay(HANG_DEADLINE_MS, undefined, { ref: false, },)
      .then(() => HUNG,),
  ],);
}

//endregion Fixtures

await describe({
  name: '',
  children: [
    describe({
      name: awaitChildEnd.name,
      children: [
        it({
          name: 'settles at once for a child that exited before the wait began',
          fn: async () => {
            /**
             Child that has already finished, which is the state `pkill` leaves
             behind before `stop` starts waiting.
             */
            const child = cpSpawn(
              COMMAND,
              [
                '-e',
                '',
              ],
              { stdio: 'ignore', },
            );
            await once(
              child,
              'exit',
            );

            expect(await settledWithinDeadline({ wait: awaitChildEnd({ child, },), },),).toBe(SETTLED,);
            expect(child.exitCode,).toBe(0,);
          },
        },),

        it({
          name: 'settles when the child exits after the wait began',
          fn: async () => {
            /**
             Child still running when the wait starts.
             */
            const child = cpSpawn(
              COMMAND,
              [
                '-e',
                `setTimeout(function stayAlive() {}, ${CHILD_LIFETIME_MS});`,
              ],
              { stdio: 'ignore', },
            );

            expect(await settledWithinDeadline({ wait: awaitChildEnd({ child, },), },),).toBe(SETTLED,);
            expect(child.exitCode,).toBe(0,);
          },
        },),

        it({
          name: 'settles for a child that never started, instead of rejecting on its error event',
          fn: async () => {
            /**
             Child whose command does not exist, so it reports an `error` and
             never an `exit`.
             */
            const child = cpSpawn(
              MISSING_COMMAND,
              [],
              { stdio: 'ignore', },
            );

            expect(await settledWithinDeadline({ wait: awaitChildEnd({ child, },), },),).toBe(SETTLED,);
            expect(child.pid,).toBeUndefined();
          },
        },),
      ],
    },),
  ],
},);
