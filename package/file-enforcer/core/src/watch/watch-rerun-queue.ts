import { caughtValueText, } from '@monochromatic-dev/caught-value-core/ts';
import {
  tagged,
  type Logger,
} from '@monochromatic-dev/logger-core/ts';

import { l, } from '../logger.ts';

/**
 Batch of debounced watch events ready for one config rerun.
 */
export type WatchRerunBatch = Readonly<{
  /**
   Changed paths whose cached reads should be invalidated before rerun.
   */
  paths: readonly string[];

  /**
   Managed destinations requiring write-protection notification before rerun.
   */
  protectedPaths: readonly string[];
}>;

/**
 Runs one queued watch rerun batch.
 */
export type WatchRerunHandler = (batch: WatchRerunBatch) => Promise<void>;

/**
 Handles errors thrown by watch rerun handlers without stopping later batches.
 */
export type WatchRerunErrorHandler = (runError: unknown) => void;

/**
 Minimal logger surface used for queue error-reporter failures.
 */
export type WatchRerunReporterLogger = Pick<Logger, 'error'>;

/**
 Serial queue for watch reruns.
 */
export type WatchRerunQueue = {
  /**
   Adds one rerun batch and resolves when that batch has been processed.
   */
  readonly enqueue: (batch: WatchRerunBatch) => Promise<void>;

  /**
   Returns number of batches waiting behind any active rerun.
   */
  readonly pendingCount: () => number;

  /**
   Returns whether a rerun is currently active.
   */
  readonly running: () => boolean;
};

//region Watch rerun queue internals

/**
 Options used to create a serial watch rerun queue.
 */
type WatchRerunQueueOptions = {
  /**
   Handler invoked for each batch in queue order.
   */
  readonly run: WatchRerunHandler;

  /**
   Error reporter used when one handler throws.
   */
  readonly onError: WatchRerunErrorHandler;

  /**
   Logger used when error reporter itself throws.
   */
  readonly logger?: WatchRerunReporterLogger;
};

/**
 Internal queue entry paired with its completion resolver.
 */
type QueuedWatchRerun = {
  /**
   Event batch to pass to the rerun handler.
   */
  readonly batch: WatchRerunBatch;

  /**
   Completion resolver for callers waiting on this queued batch.
   */
  readonly resolve: () => void;
};

/**
 Returns whether queue drain is currently active.
 
 @param runningState - Single-key state holder for active reruns.
 
 @returns Whether a rerun is currently active.
 
 @example
 ```ts
 const active = watchRerunQueueIsActive({ runningState });
 ```
 */
function watchRerunQueueIsActive(
  { runningState, }: {
    readonly runningState: ReadonlyMap<'running', true>;
  },
): boolean {
  return runningState.has('running',);
}

/**
 Reports rerun handler error without allowing reporter failures to wedge queue draining.
 
 @param onError - Configured rerun error reporter.
 
 @param runError - Error thrown by rerun handler.
 
 @param logger - Logger used when error reporter itself fails.
 
 @example
 ```ts
 reportWatchRerunError({ onError, runError, logger });
 ```
 */
function reportWatchRerunError(
  {
    onError,
    runError,
    logger,
  }: {
    readonly logger: WatchRerunReporterLogger;
    readonly onError: WatchRerunErrorHandler;
    readonly runError: unknown;
  },
): void {
  try {
    onError(runError,);
  }
  catch (reportError: unknown) {
    /**
     Last-chance message for a failing error reporter.
     */
    const message = `watch rerun error reporter failed after rerun failure: rerun=${caughtValueText(runError,)} reporter=${caughtValueText(reportError,)}`;
    try {
      logger.error(message,);
    }
    catch (loggerError: unknown) {
      process.emitWarning(
        `${message}; logger=${caughtValueText(loggerError,)}`,
      );
    }
  }
}

//endregion Watch rerun queue internals

//region Watch rerun queue public API

/**
 Creates a serial queue for watch-mode config reruns.
 
 Debounced filesystem event batches may arrive while an earlier config rerun is
 still importing, resetting trackers, or rebuilding watchers. This queue keeps
 those batches in FIFO order so reruns never overlap and later events are not
 discarded while the active rerun is in progress; queued batches drain via
 {@link drainWatchRerunQueue}.
 
 @param run - Handler invoked once for each queued rerun batch.
 
 @param onError - Error reporter invoked when a handler throws.
 
 @returns Queue that serializes watch rerun batches.
 
 @example
 ```ts
 const queue = createWatchRerunQueue({
   run: async function rerunBatch(batch) {
     await rerun(batch.paths);
   },
   onError: function logError(error) {
     logger.error(caughtValueText(error,));
   },
 });
 await queue.enqueue({ paths: ['/repo/src.ts'], protectedPaths: [] });
 ```
 */
export function createWatchRerunQueue(
  {
    run,
    onError,
    logger,
  }: WatchRerunQueueOptions,
): WatchRerunQueue {
  /**
   Function-scoped logger for queue-level error paths.
   */
  const rl = logger
    ?? tagged({
      tag: createWatchRerunQueue.name,
      l,
    },);
  /**
   FIFO rerun batches waiting behind any active rerun.
   */
  const pendingReruns: QueuedWatchRerun[] = [];
  /**
   Single-key holder indicating a drain loop is active.
   */
  const runningState = new Map<'running', true>();

  /**
   Drains queued reruns serially until no queued batches remain, reporting
   handler failures via {@link reportWatchRerunError}.
   
   @example
   ```ts
   await drainWatchRerunQueue();
   ```
   */
  async function drainWatchRerunQueue(): Promise<void> {
    if (watchRerunQueueIsActive({ runningState, },))
      return;

    runningState
      .set(
      'running',
      true,
      );
    /**
     Cleanup that clears active state even when rerun or reporter code throws unexpectedly.
     */
    using _runningCleanup = {
      [Symbol.dispose](): void {
        runningState
          .delete('running',);
      },
    };
    while (pendingReruns.length > 0) {
      /**
       Next rerun to process after all earlier batches finished.
       */
      const queuedRerun = pendingReruns.shift();
      if (queuedRerun === undefined)
        throw new Error('Watch rerun queue was empty during dequeue',);
      /**
       Event batch and completion resolver for current queued rerun.
       */
      const {
        batch,
        resolve,
      } = queuedRerun;
      try {
        // oxlint-disable-next-line eslint/no-await-in-loop -- watch reruns must run serially to prevent tracker reset/import/watch teardown overlap
        await run(batch,);
      }
      catch (runError: unknown) {
        reportWatchRerunError({
          onError,
          runError,
          logger: rl,
        },);
      }
      resolve();
    }
  }

  return {
    enqueue(batch: WatchRerunBatch,): Promise<void> {
      /**
       Completion promise and resolver for this queued batch.
       */
      const {
        promise,
        resolve,
      } = Promise.withResolvers<void>();
      pendingReruns.push({
        batch,
        resolve,
      },);
      // oxlint-disable-next-line typescript/no-floating-promises -- queue drain reports handler errors and keeps processing later batches
      drainWatchRerunQueue();
      return promise;
    },

    pendingCount(): number {
      return pendingReruns.length;
    },

    running(): boolean {
      return watchRerunQueueIsActive({ runningState, },);
    },
  };
}

//endregion Watch rerun queue public API
