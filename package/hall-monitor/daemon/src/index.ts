import { parseArgs, } from 'node:util';

import {
  MS_PER_SECOND,
  SECONDS_PER_MINUTE,
} from '@monochromatic-dev/const-core/ts';

import { forceCleanup, } from './analyze/llama.ts';
import { cycle, } from './cycle.ts';
import {
  acquireLock,
  closeLock,
  killExisting,
} from './infra/lock.ts';
import { log, } from './infra/syslog.ts';



/**
 Minutes per cycle interval.
 */
const INTERVAL_MINUTES = 5;

/**
 Interval between capture-analyze-notify cycles.
 */
const INTERVAL_MS = INTERVAL_MINUTES * SECONDS_PER_MINUTE
  * MS_PER_SECOND;

/**
 Parsed CLI arguments for the hall-monitor daemon.
 */
const { values: args, } = parseArgs({
  args: process.argv
    .slice(2,),
  options: { 'kill-existing': {
    type: 'boolean',
    default: false,
  }, },
},);

/**
 Module-singleton mutable state for the daemon loop flag; wrapped so it satisfies no-module-root-let.
 */
const state: { running: boolean; } = { running: true, };

/**
 Gracefully shuts down the daemon by stopping the main loop,
 closing the lock socket via {@link closeLock}, and force-killing any
 llama-server processes via {@link forceCleanup}.
 */
async function shutdown(): Promise<void> {
  log.debug('[hall-monitor] Shutting down...',);
  state.running = false;
  closeLock();
  await forceCleanup();
  process.exitCode = 0;
}

/**
 Synchronous signal-listener wrapper; fires the async {@link shutdown} without awaiting.
 */
function onSignal(): void {
  void shutdown();
}

process.on(
  'SIGINT',
  onSignal,
);
process.on(
  'SIGTERM',
  onSignal,
);

/**
 Main daemon entry point.
 Acquires the lock (falling back to {@link killExisting} when
 `--kill-existing` is set and {@link acquireLock} fails), runs the first
 {@link cycle}, then loops at the configured interval.
 */
// Async IIFE required because `bun build --compile` does not support top-level await.
async function main(): Promise<void> {
  if (!(await acquireLock())) {
    if (args['kill-existing'])
      await killExisting();
    else {
      throw new Error(
        '[hall-monitor] Another instance is already running. Use --kill-existing to replace it.',
      );
    }
  }

  log.debug(
    '[hall-monitor] Starting: capturing every 5 minutes, retaining last 10 minutes',
  );
  log.debug(`[hall-monitor] PID ${process.pid}, lock: abstract socket`,);

  await cycle();

  // Not needed because we don't allow configuring interval: defense-in-depth: add a floor (e.g. Math.max(INTERVAL_MS, 60_000))
  // so a misconfigured or zero interval cannot cause a tight spin loop.
  while (state.running) {
    // oxlint-disable-next-line no-await-in-loop, promise/avoid-new -- sequential timer loop; setTimeout wrapper for delay
    await new Promise(function intervalDelay(resolve,) {
      setTimeout(
        resolve,
        INTERVAL_MS,
      );
    },);
    if (state.running) {
      // oxlint-disable-next-line no-await-in-loop -- sequential polling loop; each cycle must complete before the next
      await cycle();
    }
  }
}

// oxlint-disable-next-line unicorn/prefer-top-level-await -- bun build --compile does not support top-level await
void main();
