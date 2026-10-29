import { watch as chokidarWatch, } from 'chokidar';
import { tagged, } from '@monochromatic-dev/logger-core/ts';

import { l, } from '../logger.ts';
import {
  assertExistingWatchDirectory,
  createWatchDirectoryLifecycle,
  filenameForChokidarPath,
  type WatchDirectoryOptions,
} from './watch-dir-helpers.ts';
import { classifyEvent, } from './watch-filter.ts';

/**
 Chokidar path-level events that can affect tracked sources or protected destinations.
 */
const CHOKIDAR_PATH_EVENTS = [
  'add',
  'change',
  'unlink',
  'addDir',
  'unlinkDir',
] as const;

/**
 Minimum delay between re-runs to avoid overlapping executions from rapid saves
 */
export const DEBOUNCE_MS = 100;

/**
 Starts a chokidar watcher for a single directory, classifying events and
 calling the appropriate callback. Verifies the directory exists via
 {@link assertExistingWatchDirectory} and tracks teardown state with
 {@link createWatchDirectoryLifecycle}.
 
 Runs until the abort signal fires. Abort teardown is silently caught since
 it is expected during watcher re-setup. Other watcher failures are logged
 and rethrown so failed setup or broken watch loops do not look healthy.
 
 @param dir - Absolute directory path to watch
 
 @param signal - AbortSignal for teardown
 
 @param configPath - Absolute config path for event classification
 
 @param onEvent - Callback receiving the event kind and the changed filename
 
 @mutates signal through signal.addEventListener and signal.removeEventListener listener lifecycle
 
 @example
 ```ts
 const controller = new AbortController();
 await watchDirectory({
   dir: '/abs/src',
   signal: controller.signal,
   configPath: '/abs/config.ts',
   onEvent: function logEvent(kind, filename) {
     console.log(kind, filename);
   },
 });
 ```
 */
export async function watchDirectory(
  {
    dir,
    signal,
    configPath,
    onEvent,
    onReady,
  }: WatchDirectoryOptions,
): Promise<void> {
  /**
   Function-scoped logger tagged with the call site for traceable watcher logs.
   */
  const rl = tagged({
    tag: watchDirectory.name,
    l,
  },);
  /**
   Lifecycle state for completion settlement and chokidar teardown.
   */
  const lifecycle = createWatchDirectoryLifecycle();
  /**
   Signal fired when chokidar reports that its initial scan is ready.
   */
  const ready = Promise.withResolvers<void>();

  /**
   Closes and resolves this watcher after normal abort teardown.
   
   @example
   ```ts
   void closeForAbort();
   ```
   */
  async function closeForAbort(): Promise<void> {
    await lifecycle.resolveAfterClose();
  }

  /**
   Closes and rejects this watcher after chokidar or classification failure.
   
   @param watchError - Failure that should reject the watch loop.
   
   @example
   ```ts
   void closeForFailure(error);
   ```
   */
  async function closeForFailure(watchError: unknown,): Promise<void> {
    await lifecycle.rejectAfterClose({ watchError, },);
  }

  /**
   AbortSignal event listener. EventTarget listeners cannot be async, so this
   detaches the async close path and lets `completion` carry the result.
   
   @example
   ```ts
   onAbort();
   ```
   */
  function onAbort(): void {
    // oxlint-disable-next-line typescript/no-floating-promises -- completion.promise observes close success or failure.
    closeForAbort();
  }

  /**
   Chokidar ready listener that releases callers waiting for initial scan completion,
   closing this watcher via {@link closeForFailure} when the `onReady` callback throws.
   
   @example
   ```ts
   onWatcherReady();
   ```
   */
  function onWatcherReady(): void {
    try {
      if (onReady !== undefined)
        onReady();
      ready.resolve();
    }
    catch (readyError: unknown) {
      ready.resolve();
      // oxlint-disable-next-line typescript/no-floating-promises -- lifecycle.completion observes ready callback failures.
      closeForFailure(readyError,);
    }
  }

  /**
   Chokidar error listener that fails this watched directory closed.
   
   @param watchError - Error emitted by chokidar.
   
   @example
   ```ts
   onWatcherError(new Error('synthetic watcher error'));
   ```
   */
  function onWatcherError(watchError: unknown,): void {
    // oxlint-disable-next-line typescript/no-floating-promises -- lifecycle.completion observes close success or failure.
    closeForFailure(watchError,);
  }

  /**
   Classifies one chokidar path event via {@link classifyEvent}, after
   converting it to a filename with {@link filenameForChokidarPath}, and
   forwards actionable events.
   
   @param path - Path emitted by chokidar.
   
   @example
   ```ts
   await dispatchPathEvent('/repo/src/index.ts');
   ```
   */
  async function dispatchPathEvent(path: string,): Promise<void> {
    /**
     Filename relative to watched directory, preserving the old fs.watch API shape.
     */
    const filename = filenameForChokidarPath({
      dir,
      path,
    },);
    /**
     Classification determines whether this event triggers action.
     */
    const kind = await classifyEvent({
      filename,
      watchedDir: dir,
      configPath,
    },);
    if (kind === 'ignore')
      return;

    onEvent(
      kind,
      filename,
    );
  }

  /**
   Chokidar path-event listener. Chokidar ignores returned promises, so this
   listener contains its own async error boundary.
   
   @param path - Path emitted by chokidar.
   
   @example
   ```ts
   onPathEvent('/repo/src/index.ts');
   ```
   */
  function onPathEvent(path: string,): void {
    void (async function classifyAndDispatchPathEvent(): Promise<void> {
      try {
        await dispatchPathEvent(path,);
      }
      catch (dispatchError: unknown) {
        await closeForFailure(dispatchError,);
      }
    })();
  }

  try {
    await assertExistingWatchDirectory({ dir, },);
    if (signal.aborted)
      return;

    /**
     Chokidar watcher for this directory.
     */
    const watcher = chokidarWatch(
      dir,
      {
        atomic: true,
        awaitWriteFinish: false,
        depth: 0,
        followSymlinks: false,
        ignoreInitial: true,
        persistent: true,
      },
    );
    lifecycle.setWatcher({ watcher, },);
    signal.addEventListener(
      'abort',
      onAbort,
      { once: true, },
    );
    /**
     Removes abort listener when this watchDirectory invocation exits.
     */
    using _abortListenerCleanup = {
      [Symbol.dispose](): void {
        signal.removeEventListener(
          'abort',
          onAbort,
        );
      },
    };
    watcher.on(
      'ready',
      onWatcherReady,
    );
    watcher.on(
      'error',
      onWatcherError,
    );
    for (const eventName of CHOKIDAR_PATH_EVENTS) {
      watcher.on(
        eventName,
        onPathEvent,
      );
    }
    await Promise.race([
      ready.promise,
      lifecycle.completion,
    ],);
    await lifecycle.completion;
  }
  catch (watchError: unknown) {
    if (signal.aborted)
      return;
    rl.error(`watcher error in ${dir}: ${String(watchError,)}`,);
    throw watchError;
  }
}
