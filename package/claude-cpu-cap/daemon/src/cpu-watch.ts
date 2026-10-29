// Persistent guard for Claude sessions' CPU use, run as the user unit `claude-cpu-watch.service`.
// 1. Reconciles the `claude-code-bash` cgroup quota to the thread count in `~/.config/claude-cpu-cap/threads`
//    (Claude Code recreates that cgroup in every boot and has no CPU limit setting of its own).
// 2. Logs one line per alert or recovery when the system degrades; routine samples go to the sample log only.
// Test the alert path with e.g. `CPU_WATCH_LIMIT_CPU_SOME=0 node cpu-watch.ts`.
import { once, } from 'node:events';
import { cpus, } from 'node:os';
import { setInterval, } from 'node:timers/promises';
import {
  type Logger,
  tagged,
} from '@monochromatic-dev/logger-core/ts';
import {
  ABSENT,
  INTERVAL_MS,
} from './constants.ts';
import { discoverSensors, } from './hwmon.ts';
import {
  resolveLimits,
  resolvePaths,
  resolveThermalTuning,
} from './paths.ts';
import { NO_READING, } from './power.ts';
import {
  INITIAL_STATE_BASE,
  type PassInputs,
  runPass,
} from './run-pass.ts';
import { INITIAL_STREAKS, } from './streak.ts';
import type { WatchState, } from './types.ts';

/**
 Fallback user id when the platform has no `getuid`; the cgroup path then points at the common first user.
 */
const FALLBACK_UID = 1_000;

/**
 Whether an error is the abort that ends the tick source.

 @param error - caught value

 @returns true for an `AbortError`

 @example
 ```ts
 isAbort(new DOMException('stop', 'AbortError',),); // true
 ```
 */
export function isAbort(error: unknown,): boolean {
  return (Error.isError(error,)) && (error.name === 'AbortError');
}

/**
 Runs a pass on every tick until the signal aborts; a failed pass is logged and the loop carries on
 with the state of the last good pass.

 @param inputs - constant pass inputs
 
 @param l - logger tagged for the loop
 
 @param signal - aborts the tick source and ends the loop

 @returns the last state

 @example
 ```ts
 await watchLoop({ inputs, l, signal: new AbortController().signal, },);
 ```
 */
export async function watchLoop({
  inputs,
  l,
  signal,
}: Readonly<{
  inputs: PassInputs;
  l: Logger;
  signal: AbortSignal;
}>,): Promise<WatchState> {
  /**
   State carried between passes, replaced after each good pass.
   */
  let state: WatchState = {
    ...INITIAL_STATE_BASE,
    streaks: INITIAL_STREAKS,
    reading: NO_READING,
    previousTickAt: performance.now(),
  };
  try {
    for await (const _tick of setInterval(
      INTERVAL_MS,
      undefined,
      { signal, },
    )) {
      try {
        state = await runPass({
          state,
          inputs,
          now: performance.now(),
        },);
      } catch (error) {
        l.error(`WATCHER-ERROR ${String(error,)}`,);
      }
    }
  } catch (error) {
    if (!isAbort(error,)) throw error;
    l.debug(`watch loop ended by abort (${String(error,)})`,);
  }
  return state;
}

/**
 Waits for SIGTERM, logs the stop line, flushes the logger so the line is not lost, and aborts the watch loop.

 @param l - logger for the stop line
 
 @param controller - controller whose signal ends the watch loop

 @example
 ```ts
 await stopOnSigterm({ l, controller: new AbortController(), },);
 ```
 */
export async function stopOnSigterm({
  l,
  controller,
}: Readonly<{
  l: Logger;
  controller: AbortController;
}>,): Promise<void> {
  await once(
    process,
    'SIGTERM',
  );
  l.info('WATCHER-STOPPED claude-cpu-watch received SIGTERM; no alerts until it restarts',);
  await l.flush();
  controller.abort();
}

/**
 Starts the watcher: resolves paths and sensors, logs the start line and runs the loop until SIGTERM.

 @param env - environment with optional `CPU_WATCH_*` overrides

 @example
 ```ts
 await main(process.env,);
 ```
 */
export async function main(env: Readonly<NodeJS.ProcessEnv>,): Promise<void> {
  /**
   Logger tagged with this function.
   */
  const l = tagged({ tag: main.name, },);
  /**
   Paths the watcher reads and writes.
   */
  const paths = resolvePaths({
    env,
    uid: process.getuid?.() ?? FALLBACK_UID,
  },);
  /**
   Sensor locations found on this host.
   */
  const sensors = await discoverSensors(paths.hwmonRoot,);
  /**
   Alert limits.
   */
  const limits = resolveLimits(env,);
  /**
   Ends the watch loop when SIGTERM arrives.
   */
  const controller = new AbortController();
  l.info(
    `WATCHER-STARTED limits ${JSON.stringify(limits,)} temperature ${
      sensors.temperatureFile === ABSENT ? 'unavailable' : sensors.temperatureFile
    }`,
  );
  await Promise.all([
    stopOnSigterm({
      l,
      controller,
    },),
    watchLoop({
      inputs: {
        paths,
        sensors,
        limits,
        ...resolveThermalTuning(env,),
        totalThreads: cpus()
          .length,
        l,
      },
      l,
      signal: controller.signal,
    },),
  ],);
}

await main(process.env,);
