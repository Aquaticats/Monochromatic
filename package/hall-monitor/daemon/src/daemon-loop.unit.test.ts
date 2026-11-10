/**
 Shutdown-signal and loop tests for the daemon's between-cycle wait.

 `shutdown` flipped a flag and closed the lock, but the pending interval timer
 stayed armed and held the event loop open, so SIGTERM during the five-minute
 sleep left the process running until the timer fired: systemd's default stop
 timeout then escalated to SIGKILL, and `--kill-existing` left the old process
 behind after it had released the lock. These cases prove the wait ends when
 shutdown is requested, that the loop then returns without running another
 cycle, and that the interval still wakes the loop on its own when nothing
 requests a shutdown.

 Every case is bounded by {@link HANG_DEADLINE_MS}, so a wait that never settles
 fails the case instead of blocking the suite.

 @module
 */

import { setTimeout as delay, } from 'node:timers/promises';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { recordingLogger, } from './infra/test-fixtures.ts';
import {
  createShutdownSignal,
  runDaemonLoop,
  runDetachedShutdown,
} from './daemon-loop.ts';

//region Fixtures

/**
 Interval no case waits out, standing in for the daemon's five minutes.
 */
const LONG_INTERVAL_MS = 60 * 60 * 1_000;

/**
 Interval the timer-path case does wait out.
 */
const SHORT_INTERVAL_MS = 5;

/**
 Time a control case watches before concluding a wait is still pending.
 */
const CONTROL_WINDOW_MS = 100;

/**
 Longest a wait may take before the case calls it hung.
 */
const HANG_DEADLINE_MS = 5_000;

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
 Outcome of racing a wait against a control window when it was still pending.
 */
const PENDING = 'pending';

/**
 Time a case gives the loop to start waiting before it requests a shutdown.
 */
const LOOP_START_GRACE_MS = 50;

/**
 Time a case gives a detached shutdown to report before it checks the logger.

 The report happens after the shutdown it started settles, which is one microtask
 away, so this is a margin rather than a wait for work.
 */
const DETACHED_REPORT_GRACE_MS = 50;

/**
 Races one wait against a window, so a case can observe both outcomes.

 @param wait - the wait under test

 @param windowMs - how long to watch before calling the wait still pending

 @returns {@link SETTLED} when the wait resolved first, {@link PENDING} when the
   window ended first
 */
async function outcomeWithin({
  wait,
  windowMs,
}: Readonly<{
  wait: Promise<unknown>;
  windowMs: number;
}>,): Promise<string> {
  return Promise.race([
    wait.then(() => SETTLED,),
    delay(windowMs, undefined, { ref: false, },)
      .then(() => PENDING,),
  ],);
}

/**
 Races one wait against the hang deadline.

 @param wait - the wait under test

 @returns {@link SETTLED} when the wait resolved, {@link HUNG} when the deadline
   fired first
 */
async function settledWithinDeadline({ wait, }: Readonly<{ wait: Promise<unknown>; }>,): Promise<string> {
  return outcomeWithin({
    wait,
    windowMs: HANG_DEADLINE_MS,
  },);
}

//endregion Fixtures

await describe({
  name: '',
  children: [
    describe({
      name: createShutdownSignal.name,
      children: [
        it({
          name: 'wakes the wait when the interval elapses and nothing asks it to stop',
          fn: async () => {
            expect(await settledWithinDeadline({
              wait: createShutdownSignal()
                .waitForNextCycle({ ms: SHORT_INTERVAL_MS, },),
            },),).toBe(SETTLED,);
          },
        },),

        it({
          name: 'leaves the wait pending until a shutdown is requested',
          fn: async () => {
            /**
             Signal under test, still waiting when the control window ends.
             */
            const signal = createShutdownSignal();
            /**
             Wait the interval would only end after an hour.
             */
            const wait = signal.waitForNextCycle({ ms: LONG_INTERVAL_MS, },);

            expect(await outcomeWithin({
              wait,
              windowMs: CONTROL_WINDOW_MS,
            },),).toBe(PENDING,);

            signal.request();

            expect(await settledWithinDeadline({ wait, },),).toBe(SETTLED,);
          },
        },),

        it({
          name: 'wakes a wait that starts after the shutdown request',
          fn: async () => {
            /**
             Signal already asked to stop.
             */
            const signal = createShutdownSignal();
            signal.request();

            expect(await settledWithinDeadline({
              wait: signal.waitForNextCycle({ ms: LONG_INTERVAL_MS, },),
            },),).toBe(SETTLED,);
            expect(signal.requested(),).toBe(true,);
          },
        },),
      ],
    },),

    describe({
      name: runDetachedShutdown.name,
      children: [
        it({
          name: 'starts the shutdown and reports nothing when it succeeds',
          fn: async () => {
            /**
             Recording logger the shutdown would report through.
             */
            const { logger, records, } = recordingLogger();
            /**
             Shutdowns the detached call started.
             */
            const started: string[] = [];

            runDetachedShutdown({
              l: logger,
              shutdown: async function succeed(): Promise<void> {
                started.push('shutdown',);
              },
            },);
            await delay(DETACHED_REPORT_GRACE_MS,);

            expect(started.length,).toBe(1,);
            expect(records,).toEqual([],);
          },
        },),

        it({
          name: 'reports and flushes a failed shutdown instead of letting it end the process',
          fn: async () => {
            /**
             Recording logger the failure is reported through.
             */
            const { logger, records, flushCount, } = recordingLogger();
            /**
             Exit code this case restores, because a failed shutdown sets it.
             */
            const previousExitCode = process.exitCode;
            /**
             Restores the exit code however this case ends.
             */
            await using _restoreExitCode = {
              async [Symbol.asyncDispose](): Promise<void> {
                process.exitCode = previousExitCode;
              },
            } as AsyncDisposable;

            runDetachedShutdown({
              l: logger,
              shutdown: async function fail(): Promise<void> {
                throw new Error('the lock socket would not close',);
              },
            },);
            await delay(DETACHED_REPORT_GRACE_MS,);

            expect(records,).toEqual([
              {
                level: 'error',
                message: 'Shutdown failed: the lock socket would not close',
              },
            ],);
            expect(flushCount(),).toBe(1,);
          },
        },),
      ],
    },),

    describe({
      name: runDaemonLoop.name,
      children: [
        it({
          name: 'returns after the first cycle when a shutdown is already requested',
          fn: async () => {
            /**
             Signal asked to stop before the loop starts.
             */
            const signal = createShutdownSignal();
            signal.request();
            /**
             Cycles the loop ran; a shutdown request must stop it at once.
             */
            const cycles: string[] = [];

            expect(await settledWithinDeadline({
              wait: runDaemonLoop({
                cycle: async function recordCycle(): Promise<void> {
                  cycles.push('cycle',);
                },
                intervalMs: LONG_INTERVAL_MS,
                signal,
              },),
            },),).toBe(SETTLED,);
            expect(cycles.length,).toBe(1,);
          },
        },),

        it({
          name: 'ends the wait and returns when a shutdown arrives between cycles',
          fn: async () => {
            /**
             Signal the case asks to stop once the loop is waiting.
             */
            const signal = createShutdownSignal();
            /**
             Cycles the loop ran; the shutdown must stop it before a second one.
             */
            const cycles: string[] = [];
            /**
             Loop under test, still waiting on its hour-long interval when the
             case requests the shutdown.
             */
            const loop = runDaemonLoop({
              cycle: async function recordCycle(): Promise<void> {
                cycles.push('cycle',);
              },
              intervalMs: LONG_INTERVAL_MS,
              signal,
            },);

            await delay(LOOP_START_GRACE_MS,);
            signal.request();

            expect(await settledWithinDeadline({ wait: loop, },),).toBe(SETTLED,);
            expect(cycles.length,).toBe(1,);
          },
        },),
      ],
    },),
  ],
},);
