import { setTimeout as wait, } from 'node:timers/promises';
import type { Logger, } from '@monochromatic-dev/logger-core/ts';

import { watchDirectory, } from './watch-dir.ts';
import type { EventKind, } from './watch-filter.ts';
//region Watcher restart policy

/**
 Number of restart attempts after initial watcher failure before watch mode fails closed.
 */
const WATCHER_RESTART_LIMIT = 3;

/**
 Delay between watcher restart attempts.
 */
const WATCHER_RESTART_DELAY_MS = 50;

/**
 Watcher implementation signature used by the restart supervisor.
 */
type WatchDirectoryImplementation = typeof watchDirectory;

/**
 Logger surface needed by the watcher restart supervisor.
 */
type WatchSupervisorLogger = Pick<Logger, 'error' | 'info'>;

/**
 Options for one supervised watcher loop.
 */
type SupervisedWatcherOptions = {
  /**
   Directory path passed to chokidar.
   */
  dir: string;

  /**
   Abort signal used to stop watcher during normal teardown.
   */
  signal: AbortSignal;

  /**
   Absolute config path used for event classification.
   */
  configPath: string;

  /**
   Watch event callback forwarded to `watchDirectory`.
   */
  onEvent: (
    kind: EventKind,
    filename: string,
  ) => void;

  /**
   Callback fired after chokidar reports its initial scan is ready.
   */
  onReady?: () => void;

  /**
   Tagged logger for restart and limit diagnostics.
   */
  logger: WatchSupervisorLogger;

  /**
   Watcher implementation, injectable for focused tests.
   */
  watchDirectoryImpl?: WatchDirectoryImplementation;

  /**
   Restart attempts allowed after initial failure.
   */
  restartLimit?: number;

  /**
   Delay between restart attempts.
   */
  restartDelayMs?: number;
};

/**
 Returns attempt ordinals for initial watcher start and bounded restarts.
 
 @param restartLimit - Restart attempts allowed after initial failure.
 
 @returns Attempt ordinals, where zero is initial start.
 
 @example
 ```ts
 const attempts = watcherAttemptOrdinals({ restartLimit: 3 });
 ```
 */
function watcherAttemptOrdinals(
  { restartLimit, }: { readonly restartLimit: number; },
): readonly number[] {
  return Array.from(
    { length: restartLimit + 1, },
    function attemptOrdinal(
      _unusedValue,
      attemptIndex,
    ): number {
      return attemptIndex;
    },
  );
}

/**
 Returns whether a caught watcher error is the expected abort teardown path.
 
 @param error - Caught watcher error.
 
 @returns Whether the error is an abort signal.
 
 @example
 ```ts
 const expected = watcherErrorIsAbort(error);
 ```
 */
function watcherErrorIsAbort(error: unknown,): boolean {
  return (Error.isError(error,)) && (error.name === 'AbortError');
}

/**
 Builds the fail-closed error thrown after watcher restart attempts are exhausted.
 
 @param dir - Directory whose watcher could not stay running.
 
 @param restartLimit - Restart attempts allowed after initial failure.
 
 @param cause - Last watcher failure.
 
 @returns Error explaining that watch mode exhausted its restart limit.
 
 @example
 ```ts
 throw watcherRestartLimitError({ dir: '/repo/src', restartLimit: 3, cause: error });
 ```
 */
function watcherRestartLimitError(
  {
    dir,
    restartLimit,
    cause,
  }: {
    readonly cause: unknown;
    readonly dir: string;
    readonly restartLimit: number;
  },
): Error {
  return new Error(
    `watcher restart limit exceeded for ${dir} after ${String(restartLimit,)} restart attempts`,
    { cause, },
  );
}

/**
 Waits before restarting a watcher, treating abort as normal teardown.
 
 @param delayMs - Restart delay in milliseconds.
 
 @param signal - Abort signal for normal watcher teardown.
 
 @mutates signal through wait abort-listener registration and retention
 
 @example
 ```ts
 await waitBeforeWatcherRestart({ delayMs: 50, signal });
 ```
 */
async function waitBeforeWatcherRestart(
  {
    delayMs,
    signal,
  }: {
    readonly delayMs: number;
    readonly signal: AbortSignal;
  },
): Promise<void> {
  try {
    await wait(
      delayMs,
      undefined,
      { signal, },
    );
  }
  catch (waitError: unknown) {
    if (watcherErrorIsAbort(waitError,))
      return;

    throw waitError;
  }
}

//endregion Watcher restart policy

//region Supervised watcher loop

/**
 Runs one watcher and restarts it after non-abort failures up to a bounded limit.
 
 @param dir - Directory path passed to chokidar.
 
 @param signal - Abort signal used to stop watcher during normal teardown.
 
 @param configPath - Absolute config path used for event classification.
 
 @param onEvent - Watch event callback forwarded to {@link watchDirectory}.
 
 @param onReady - Callback fired after chokidar reports its initial scan is ready.
 
 @param logger - Tagged logger for restart and limit diagnostics.
 
 @param watchDirectoryImpl - Watcher implementation, injectable for focused tests.
 
 @param restartLimit - Restart attempts allowed after initial failure.
 
 @param restartDelayMs - Delay between restart attempts.
 
 @mutates signal through wait abort-listener registration and retention
 
 @throws {@link watcherRestartLimitError} When watcher failures exceed the restart limit.
 
 @example
 ```ts
 await watchDirectoryWithRestarts({
   dir: '/repo/src',
   signal: controller.signal,
   configPath: '/repo/file-enforcer.config.ts',
   logger,
   onEvent(kind, filename) {
     queue(kind, filename);
   },
 });
 ```
 */
export async function watchDirectoryWithRestarts(
  {
    dir,
    signal,
    configPath,
    onEvent,
    onReady,
    logger,
    watchDirectoryImpl = watchDirectory,
    restartLimit = WATCHER_RESTART_LIMIT,
    restartDelayMs = WATCHER_RESTART_DELAY_MS,
  }: SupervisedWatcherOptions,
): Promise<void> {
  for (const attempt of watcherAttemptOrdinals({ restartLimit, },)) {
    if (signal.aborted)
      return;

    try {
      // oxlint-disable-next-line no-await-in-loop -- restart attempts must observe each watcher failure before deciding whether to retry.
      await watchDirectoryImpl({
        dir,
        signal,
        configPath,
        onEvent,
        ...(onReady === undefined
          ? {}
          : { onReady, }),
      },);
      return;
    }
    catch (watchError: unknown) {
      if (signal.aborted || watcherErrorIsAbort(watchError,))
        return;

      logger.error(`watcher failed in ${dir}: ${String(watchError,)}`,);
      if (attempt >= restartLimit) {
        /**
         Fail-closed error surfaced to startWatching.
         */
        const limitError = watcherRestartLimitError({
          dir,
          restartLimit,
          cause: watchError,
        },);
        logger.error(limitError.message,);
        throw limitError;
      }

      logger.info(`restarting watcher in ${dir} after failure ${String(attempt + 1,)} of ${String(restartLimit,)}`,);
      // oxlint-disable-next-line no-await-in-loop -- bounded retry delay must complete before next watcher start attempt.
      await waitBeforeWatcherRestart({
        delayMs: restartDelayMs,
        signal,
      },);
    }
  }
}

//endregion Supervised watcher loop
