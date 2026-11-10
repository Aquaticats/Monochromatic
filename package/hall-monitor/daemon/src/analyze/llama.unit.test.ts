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

import {
  type ChildProcess,
  spawn as cpSpawn,
} from 'node:child_process';
import { once, } from 'node:events';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  awaitChildEnd,
  startLlamaServer,
  stopLlamaServer,
} from '../../dist/final/node/testing.mjs';

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
 How long a fixture child stays alive if the case does not end it, so every case
 has a running server to stop.
 */
const FIXTURE_CHILD_LIFETIME_MS = 60_000;

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
  /**
   Deadline timer, held so it can be cleared once the race settles.

   The timer is referenced, so it keeps the process alive long enough to fire: an
   unreferenced one lets Node report an unsettled top-level await instead, which
   hides the failure this helper exists to report.
   */
  const deadline: { timer?: ReturnType<typeof setTimeout>; } = {};

  /**
   Outcome of the race between the wait and the deadline.
   */
  const outcome = await Promise.race([
    wait.then(() => SETTLED,),
    new Promise<string>(
      /**
       Resolves the deadline when it fires.

       @param resolve - Promise settlement callback.
       */
      function armDeadline(resolve,): void {
        deadline.timer = setTimeout(
          function onDeadline(): void {
            resolve(HUNG,);
          },
          HANG_DEADLINE_MS,
        );
      },
    ),
  ],);
  clearTimeout(deadline.timer,);
  return outcome;
}

//endregion Fixtures

//region Server fixtures

/**
 Spawns a child that stays alive until the case ends it, standing in for the
 server process.

 @returns handle of the still-running child

 @example
 ```ts
 const child = spawnLivingChild();
 ```
 */
function spawnLivingChild(): ChildProcess {
  return cpSpawn(
    COMMAND,
    [
      '-e',
      `setTimeout(function stayAlive() {}, ${FIXTURE_CHILD_LIFETIME_MS});`,
    ],
    { stdio: 'ignore', },
  );
}

/**
 Reports readiness without waiting for a health endpoint, because a fixture child
 is not a server.

 @returns an already-settled wait
 */
function noHealthWait(): Promise<void> {
  return Promise.resolve();
}

/**
 Ends a fixture child the case has left running, so a failed assertion cannot
 leave a process behind.

 @param child - child the case started
 */
function endIfAlive({ child, }: Readonly<{ child: ChildProcess; }>,): void {
  if ((child.exitCode === null) && (child.signalCode === null))
    child.kill('SIGKILL',);
}

/**
 Ends every fixture child of one case when the case's scope ends, whichever way
 it ends.

 @param children - the children the case started

 @returns disposable that ends them

 @example
 ```ts
 const child = spawnLivingChild();
 await using _cleanup = endingFixtureChildren({ children: [child] });
 ```
 */
function endingFixtureChildren({
  children,
}: Readonly<{ children: readonly ChildProcess[]; }>,): AsyncDisposable {
  return {
    async [Symbol.asyncDispose](): Promise<void> {
      for (const child of children)
        endIfAlive({ child, },);
    },
  };
}

/**
 Ends one fixture child at the moment a signal is requested, which is the way
 `pkill` ends the wrapped server while `stop` is still delivering the call.

 @param child - child the case started

 @returns the signal the case injects into `stop`

 @example
 ```ts
 await stopLlamaServer({ signalServer: shutDownDuringCall({ child }) });
 ```
 */
function shutDownDuringCall({ child, }: Readonly<{ child: ChildProcess; }>,): () => Promise<void> {
  return async function signalAndWaitForExit(): Promise<void> {
    child.kill('SIGTERM',);
    await once(
      child,
      'exit',
    );
  };
}

//endregion Server fixtures

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

    describe({
      name: 'the server run',
      // These cases drive the one server handle this module holds, so each must
      // settle before the next starts: a parallel case would take another case's
      // handle for its own.
      concurrency: 1,
      children: [
        it({
          name: 'ends a server the signal ended during the call, instead of reading back a cleared handle',
          fn: async () => {
            /**
             Server process the injected signal ends while `stop` awaits it.
             */
            const child = spawnLivingChild();
            await using _cleanup = endingFixtureChildren({ children: [child,], },);

            await startLlamaServer({
              spawnServer: () => child,
              waitForReady: noHealthWait,
            },);
            // The signal ends the child, whose exit listener clears the stored
            // handle before this call returns; `stop` must still hold the one it
            // took on entry.
            await stopLlamaServer({ signalServer: shutDownDuringCall({ child, },), },);

            expect(child.signalCode,).toBe('SIGTERM',);
          },
        },),

        it({
          name: 'starts a fresh server once the handle it holds has exited',
          fn: async () => {
            /**
             First server, which ends on its own before the second start.
             */
            const first = spawnLivingChild();
            /**
             Replacement server the second start spawns.
             */
            const second = spawnLivingChild();
            await using _cleanup = endingFixtureChildren({ children: [
              first,
              second,
            ], },);
            /**
             Handles the case's spawn functions have handed out, in order.
             */
            const spawned: ChildProcess[] = [];

            await startLlamaServer({
              spawnServer: () => {
                spawned.push(first,);
                return first;
              },
              waitForReady: noHealthWait,
            },);
            first.kill('SIGKILL',);
            await once(
              first,
              'exit',
            );

            await startLlamaServer({
              spawnServer: () => {
                spawned.push(second,);
                return second;
              },
              waitForReady: noHealthWait,
            },);
            await stopLlamaServer({ signalServer: shutDownDuringCall({ child: second, },), },);

            expect(spawned.length,).toBe(2,);
            expect(second.signalCode,).toBe('SIGTERM',);
          },
        },),

        it({
          name: 'starts nothing while the server it holds is still running',
          fn: async () => {
            /**
             Server process the case keeps alive across both starts.
             */
            const child = spawnLivingChild();
            await using _cleanup = endingFixtureChildren({ children: [child,], },);
            /**
             Number of times the case's spawn function was asked for a server.
             */
            let spawnedCount = 0;

            /**
             Hands out the running server, counting every request.

             @returns the running server
             */
            function spawnServer(): ChildProcess {
              spawnedCount += 1;
              return child;
            }

            await startLlamaServer({
              spawnServer,
              waitForReady: noHealthWait,
            },);
            await startLlamaServer({
              spawnServer,
              waitForReady: noHealthWait,
            },);

            expect(spawnedCount,).toBe(1,);
            await stopLlamaServer({ signalServer: shutDownDuringCall({ child, },), },);
          },
        },),

        it({
          name: 'signals nothing when no server is held',
          fn: async () => {
            /**
             Whether the injected signal was asked for, which it must not be.
             */
            let signalled = false;

            await stopLlamaServer({
              signalServer: async () => {
                signalled = true;
              },
            },);

            expect(signalled,).toBe(false,);
          },
        },),
      ],
    },),
  ],
},);
