import { parseArgs, } from 'node:util';

import {
  MS_PER_SECOND,
  SECONDS_PER_MINUTE,
} from '@monochromatic-dev/const-core/ts';
import { tagged, } from '@monochromatic-dev/logger-core/ts';

import { forceCleanup, } from './analyze/llama.ts';
import { cycle, } from './cycle.ts';
import {
  createShutdownSignal,
  runDaemonLoop,
} from './daemon-loop.ts';
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
 Shutdown signal the loop watches and the signal handlers request.
 */
const shutdownSignal = createShutdownSignal();

/**
 Journal flush that runs when its `await using` scope ends.

 The journal sink writes through `logger(1)`, so the records logged last are
 still in flight when a shutdown begins. Binding this to a scope delivers them
 on the throwing path too, so a failed shutdown keeps the records that explain it.
 */
const JOURNAL_FLUSH_ON_EXIT: AsyncDisposable = {
  async [Symbol.asyncDispose](): Promise<void> {
    await log.flush();
  },
};

/**
 Gracefully shuts down the daemon by stopping the main loop,
 closing the lock socket via {@link closeLock}, and force-killing any
 llama-server processes via {@link forceCleanup}.
 */
async function shutdown(): Promise<void> {
  /**
   Shutdown logger, tagged with this function's name so the last records of a run name their origin.
   */
  const l = tagged({
    l: log,
    tag: shutdown.name,
  },);
  l.debug('Shutting down...',);
  shutdownSignal.request();
  closeLock();
  /**
   Journal flush bound to this call's scope.
   */
  await using _flushRecords = JOURNAL_FLUSH_ON_EXIT;
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
        'Another instance is already running. Use --kill-existing to replace it.',
      );
    }
  }

  /**
   Startup logger, tagged with this function's name.
   */
  const l = tagged({
    l: log,
    tag: main.name,
  },);
  l.debug(
    'Starting: capturing every 5 minutes, retaining last 10 minutes',
  );
  l.debug(`PID ${process.pid}, lock: abstract socket`,);

  await runDaemonLoop({
    cycle,
    intervalMs: INTERVAL_MS,
    signal: shutdownSignal,
  },);
}

/**
 Flushes the records a startup failure would otherwise take with it, then lets
 the failure end the process.

 @example
 ```ts
 void runMain();
 ```
 */
async function runMain(): Promise<void> {
  try {
    await main();
  }
  catch (error: unknown) {
    // The records logged before the failure are what explains it, so they are
    // flushed before the rethrow ends the process.
    await log.flush();
    throw error;
  }
}

// oxlint-disable-next-line unicorn/prefer-top-level-await -- bun build --compile does not support top-level await
void runMain();
