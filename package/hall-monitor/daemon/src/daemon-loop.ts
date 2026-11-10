/**
 The daemon's run loop, the shutdown signal that ends it, and the detached
 shutdown the signal handlers start.

 `shutdown` used to flip a flag and leave the pending `setTimeout` armed, so the
 event loop stayed alive until the five-minute interval fired: systemd's default
 stop timeout escalated to SIGKILL, and `--kill-existing` left the old process
 behind after it had already released the lock. The wait is now cancellable, and
 the loop stops as soon as a shutdown is requested, whether the request arrives
 during the wait or between cycles.

 @module
 */

import { caughtValueText, } from '@monochromatic-dev/caught-value-core/ts';
import type { Logger, } from '@monochromatic-dev/logger-core/ts';

/**
 Cancellable shutdown signal and the between-cycle wait that listens to it.
 */
export type ShutdownSignal = {
  /**
   Records the shutdown request and wakes a pending wait.
   */
  readonly request: () => void;
  /**
   Whether a shutdown has been requested.
   */
  readonly requested: () => boolean;
  /**
   Resolves when the interval elapses or a shutdown is requested, whichever is
   first.
   */
  readonly waitForNextCycle: (options: Readonly<{ ms: number; }>) => Promise<void>;
};

//region Shutdown signal

/**
 Creates a shutdown signal and its cancellable wait.

 The signal holds no timer of its own: each wait owns its timer and clears it
 when a shutdown wakes it, so nothing keeps the process alive after the loop
 stops.

 @returns signal whose request ends any pending wait

 @example
 ```ts
 const signal = createShutdownSignal();
 process.on('SIGTERM', signal.request);
 await runDaemonLoop({ cycle, intervalMs, signal });
 ```
 */
export function createShutdownSignal(): ShutdownSignal {
  /**
   Signal state, held in one holder so the factory root needs no `let`.
   */
  const state: {
    /**
     Whether a shutdown has been requested.
     */
    requested: boolean;
    /**
     Wake-up for the wait in progress, absent while the loop is not waiting.
     */
    wake?: () => void;
  } = { requested: false, };

  return {
    request: function request(): void {
      state.requested = true;
      /**
       Wake-up of the wait in progress, taken out of the state so a second
       request cannot settle the same wait twice.
       */
      const { wake, } = state;
      delete state.wake;
      if (wake !== undefined)
        wake();
    },

    requested: function requested(): boolean {
      return state.requested;
    },

    waitForNextCycle: function waitForNextCycle({ ms, }: Readonly<{ ms: number; }>,): Promise<void> {
      if (state.requested)
        return Promise.resolve();

      // oxlint-disable-next-line promise/avoid-new -- callback bridge over setTimeout, which offers no promise of its own
      return new Promise<void>(
        /**
         Registers promise settlement with the interval timer.

         @param resolve - Promise settlement callback.

         @mutates resolve through the timer handle and the stored wake-up
         */
        function wait(resolve,): void {
          /**
           Interval timer that ends the wait when it fires.
           */
          const timer = setTimeout(
            function onInterval(): void {
              delete state.wake;
              resolve();
            },
            ms,
          );
          state.wake = function wakeOnShutdown(): void {
            clearTimeout(timer,);
            resolve();
          };
        },
      );
    },
  };
}

//endregion Shutdown signal

//region Loop

/**
 Runs the daemon's capture loop until a shutdown is requested.

 The first cycle runs at once, then the loop waits out the interval and runs the
 next one, stopping as soon as the signal is requested: a request that arrives
 during the wait wakes it, and one that arrives during a cycle is seen before the
 next wait starts.

 @param cycle - one capture-analyze-notify cycle

 @param intervalMs - milliseconds to wait between cycles

 @param signal - shutdown signal the loop watches

 @example
 ```ts
 await runDaemonLoop({ cycle, intervalMs: INTERVAL_MS, signal: createShutdownSignal() });
 ```
 */
export async function runDaemonLoop({
  cycle,
  intervalMs,
  signal,
}: Readonly<{
  cycle: () => Promise<void>;
  intervalMs: number;
  signal: ShutdownSignal;
}>): Promise<void> {
  await cycle();

  while (!signal.requested()) {
    // oxlint-disable-next-line no-await-in-loop -- sequential polling loop; each cycle must complete before the next
    await signal.waitForNextCycle({ ms: intervalMs, },);
    if (signal.requested())
      break;
    // oxlint-disable-next-line no-await-in-loop -- sequential polling loop; each cycle must complete before the next
    await cycle();
  }
}

//endregion Loop

//region Detached shutdown

/**
 Starts one shutdown from a synchronous signal handler.

 A signal handler cannot await, so it discards the promise this starts, and a
 discarded promise that rejects ends the process as an unhandled rejection. The
 started promise therefore never rejects: a failed shutdown is reported and
 flushed, and the records that explain it survive the exit.

 @param shutdown - shutdown work to run

 @param l - logger the failure is reported through, and flushed afterwards

 @example
 ```ts
 process.on('SIGTERM', function onSignal() {
   runDetachedShutdown({ shutdown, l });
 });
 ```
 */
export function runDetachedShutdown({
  shutdown,
  l,
}: Readonly<{
  shutdown: () => Promise<void>;
  l: Logger;
}>): void {
  void reportShutdownFailure({
    l,
    shutdown,
  },);
}

/**
 Runs one shutdown, reporting and flushing a failure instead of letting it end
 the process.

 @param shutdown - shutdown work to run

 @param l - logger the failure is reported through, and flushed afterwards

 @example
 ```ts
 await reportShutdownFailure({ shutdown, l });
 ```
 */
async function reportShutdownFailure({
  shutdown,
  l,
}: Readonly<{
  shutdown: () => Promise<void>;
  l: Logger;
}>): Promise<void> {
  try {
    await shutdown();
  }
  catch (error: unknown) {
    l.error(`Shutdown failed: ${caughtValueText(error,)}`,);
    // The records filed before the failure are what explain it, so they are flushed before the
    // process is left to exit on its own.
    await l.flush();
    process.exitCode = 1;
  }
}

//endregion Detached shutdown
