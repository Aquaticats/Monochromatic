import {
  join,
  resolve,
} from 'node:path';
import { caughtValueText, } from '@monochromatic-dev/caught-value-core/ts';
import { tagged, } from '@monochromatic-dev/logger-core/ts';

import { setActiveConfigPath, } from '../context.ts';
import { l, } from '../logger.ts';
import { invalidatePaths, } from '../io/cache.ts';
import { reset, } from '../tracker.ts';
import { notifyWriteProtection, } from './notify.ts';
import {
  DEBOUNCE_MS,
} from './watch-dir.ts';
import {
  type EventKind,
  watchDirs,
} from './watch-filter.ts';
import {
  createWatchRerunQueue,
  type WatchRerunBatch,
} from './watch-rerun-queue.ts';
import { createWatchModeLifecycle, } from './watch-lifecycle.ts';
import { watchDirectoryWithRestarts, } from './watch-supervisor.ts';

/**
 Watches source files and managed destinations, re-executing the config
 on source changes and reverting + notifying on external destination edits.
 Builds its watch-mode state with {@link createWatchModeLifecycle}, serializes
 reruns through {@link createWatchRerunQueue}, derives watched directories with
 {@link watchDirs}, supervises each one via {@link watchDirectoryWithRestarts},
 and reports protected-path edits with {@link notifyWriteProtection}.
 
 This function never returns under normal operation (it loops forever watching).
 
 @param configPath - Path to the file-enforcer config file
 
 @example
 ```ts
 await startWatching('./file-enforcer.config.ts');
 ```
 */
export function startWatching(configPath: string,): Promise<never> {
  /**
   Function-scoped logger tagged with the call site for traceable watch-mode logs.
   */
  const rl = tagged({
    tag: startWatching.name,
    l,
  },);
  /**
   Absolute config path for reliable comparisons
   */
  const absoluteConfig = resolve(configPath,);
  setActiveConfigPath({ configPath: absoluteConfig, },);
  rl.info('watch mode started',);

  /**
   Watch-mode state holder for watcher teardown, debounce cleanup, and fail-closed rejection.
   */
  const lifecycle = createWatchModeLifecycle();

  /**
   Paths accumulated during the debounce window, invalidated together on re-run
   */
  const pendingPaths: Set<string> = new Set<string>();
  /**
   Protected paths that need notification, accumulated during the debounce window
   */
  const pendingProtected: Set<string> = new Set<string>();
  /**
   Re-imports the config with a cache-busting query parameter,
   then updates the watcher set from newly tracked reads/writes.
   
   @param changedPaths - Absolute paths of files that triggered the re-run,
     invalidated from the read cache so only those files are re-read from disk
   */
  async function rerun(changedPaths: readonly string[],): Promise<void> {
    rl.info('re-running config...',);
    invalidatePaths(changedPaths,);
    reset();
    try {
      await import(`${absoluteConfig}?v=${String(Date.now(),)}`);
    }
    catch (importError: unknown) {
      rl.error(`config execution failed: ${caughtValueText(importError,)}`,);
      return;
    }
    rl.info('re-run complete',);
    lifecycle.closeAllWatchers();
    await setupWatchers();
  }

  /**
   Serial queue that prevents overlapping config reruns from interleaving tracker
   reset, config import, and watcher recreation.
   */
  const rerunQueue = createWatchRerunQueue({
    run: async function runWatchRerunBatch(batch: WatchRerunBatch,): Promise<void> {
      for (const protectedPath of batch.protectedPaths) {
        // oxlint-disable-next-line no-await-in-loop -- sequential notification to avoid spamming
        await notifyWriteProtection(protectedPath,);
      }
      await rerun(batch.paths,);
    },
    onError: function logWatchRerunError(runError: unknown,): void {
      rl.error(`watch rerun failed: ${caughtValueText(runError,)}`,);
    },
  },);

  /**
   Handles a classified filesystem event by accumulating the changed path
   and scheduling a debounced re-run. Multiple rapid events are coalesced
   into a single re-run that invalidates all accumulated paths.
   
   @param kind - classification of the filesystem event
   
   @param filename - filename from the watch event
   
   @param dir - directory the event occurred in
   */
  function handleEvent(
    {
      kind,
      filename,
      dir,
    }: {
      readonly kind: EventKind;
      readonly filename: string;
      readonly dir: string;
    },
  ): void {
    /**
     Absolute path of the file that triggered this event
     */
    const changedPath = resolve(join(
      dir,
      filename,
    ),);
    if (lifecycle.hasFailed())
      return;
    pendingPaths.add(changedPath,);
    if (kind === 'protected')
      pendingProtected.add(changedPath,);
    lifecycle.scheduleDebounce({
      delayMs: DEBOUNCE_MS,
      callback: function debouncedRerun(): void {
        /**
         Snapshot accumulated state before clearing
         */
        const paths = [...pendingPaths,];
        /**
         Snapshot of paths that need write-protection notifications, paired with `paths`.
         */
        const protectedPaths = [...pendingProtected,];
        /**
         Debounced watch event batch submitted to the serial rerun queue.
         */
        const batch: WatchRerunBatch = {
          paths,
          protectedPaths,
        };
        pendingPaths.clear();
        pendingProtected.clear();
        lifecycle.clearDebounceTimer();
        // oxlint-disable-next-line typescript/no-floating-promises -- queued reruns report errors and keep draining later batches
        rerunQueue.enqueue(batch,);
      },
    },);
  }

  /**
   Supervises one watcher, reporting exhausted restart attempts to watch mode.
   
   @param dir - Directory being watched.
   
   @param controller - Abort controller for normal watcher teardown.
   
   @param onReady - Callback fired after chokidar reports initial scan readiness.
   
   @mutates controller through wait abort-listener registration on controller.signal
   
   @example
   ```ts
   await monitorWatcher({ dir: '/repo/src', controller, onReady });
   ```
   */
  async function monitorWatcher(
    {
      dir,
      controller,
      onReady,
    }: {
      readonly controller: AbortController;
      readonly dir: string;
      readonly onReady: () => void;
    },
  ): Promise<void> {
    try {
      await watchDirectoryWithRestarts({
        dir,
        signal: controller.signal,
        configPath: absoluteConfig,
        logger: rl,
        onReady,
        onEvent: function onWatchEvent(
          kind,
          filename,
        ): void {
          handleEvent({
            kind,
            filename,
            dir,
          },);
        },
      },);
    }
    catch (watchError: unknown) {
      lifecycle.fail(watchError,);
    }
  }

  /**
   Creates watchers for every directory derived from tracked reads and writes,
   waiting until their initial chokidar scans have completed.
   */
  async function setupWatchers(): Promise<void> {
    if (lifecycle.hasFailed())
      return;
    /**
     Directories to watch, derived from current tracked reads + writes
     */
    const dirs = await watchDirs(absoluteConfig,);
    rl.info(`watching ${String(dirs.size,)} directories`,);

    /**
     Per-watcher readiness signals; all must resolve before setup is complete.
     */
    const readySignals = [...dirs,].map(function setupDir(dir,): Promise<void> {
      /**
       Per-directory abort controller for teardown
       */
      const controller = new AbortController();
      lifecycle.registerController({
        dir,
        controller,
      },);
      /**
       Readiness signal for this chokidar watcher.
       */
      const ready = Promise.withResolvers<void>();

      /**
       Marks this watcher ready after chokidar's initial scan completes.
       
       @example
       ```ts
       markWatcherReady();
       ```
       */
      function markWatcherReady(): void {
        ready.resolve();
      }

      // oxlint-disable-next-line typescript/no-floating-promises -- monitorWatcher catches watcher failures and rejects watchModeFailure after restart limits.
      monitorWatcher({
        dir,
        controller,
        onReady: markWatcherReady,
      },);

      return ready.promise;
    },);

    await Promise.race([
      Promise.all(readySignals,),
      lifecycle.failure,
    ],);
  }

  return (async function runInitialWatchSetup(): Promise<never> {
    await setupWatchers();
    return await lifecycle.failure;
  })();
}
