import { stat, } from 'node:fs/promises';
import { relative, } from 'node:path';
import type { FSWatcher, } from 'chokidar';

import type { EventKind, } from './watch-filter.ts';

/**
 Arguments accepted by watchDirectory.
 */
export type WatchDirectoryOptions = {
  /**
   Absolute directory path to watch.
   */
  dir: string;

  /**
   AbortSignal for teardown.
   */
  signal: AbortSignal;

  /**
   Absolute config path for event classification.
   */
  configPath: string;

  /**
   Callback receiving event classification and changed filename relative to {@link dir}.
   */
  onEvent: (
    kind: EventKind,
    filename: string,
  ) => void;

  /**
   Optional callback fired after chokidar reports its initial scan is ready.
   */
  onReady?: () => void;
};

/**
 Lifecycle helper for one chokidar-backed watchDirectory call.
 */
export type WatchDirectoryLifecycle = Readonly<{
  /**
   Promise resolved by normal close and rejected by watcher failure.
   */
  completion: Promise<void>;

  /**
   Records active chokidar watcher for later teardown.
   */
  setWatcher: (args: { readonly watcher: FSWatcher; }) => void;

  /**
   Resolves completion after closing chokidar for normal abort teardown.
   */
  resolveAfterClose: () => Promise<void>;

  /**
   Rejects completion after closing chokidar for watcher failure paths.
   */
  rejectAfterClose: (args: { readonly watchError: unknown; }) => Promise<void>;
}>;

/**
 Verifies that chokidar is being asked to watch an existing directory.
 
 Chokidar can silently treat missing paths as future paths. File-enforcer's
 restart supervisor depends on setup failures rejecting, so this explicit
 check preserves the previous `fs.watch` failure contract.
 
 @param dir - Absolute directory path to validate.
 
 @throws When metadata lookup fails or `dir` is not a directory.
 
 @example
 ```ts
 await assertExistingWatchDirectory({ dir: '/repo/src' });
 ```
 */
export async function assertExistingWatchDirectory(
  { dir, }: { readonly dir: string; },
): Promise<void> {
  /**
   Filesystem metadata for watched root.
   */
  const dirStat = await stat(dir,);
  if (dirStat.isDirectory())
    return;

  throw new Error(`watch root is not a directory: ${dir}`,);
}

/**
 Converts chokidar's emitted path into the filename shape expected by
 {@link classifyEvent} and existing {@link watchDirectory} callers.
 
 @param dir - Watched directory root.
 
 @param path - Path emitted by chokidar.
 
 @returns Path relative to `dir`, or `.` for the watched root itself.
 
 @example
 ```ts
 const filename = filenameForChokidarPath({ dir: '/repo/src', path: '/repo/src/a.ts' });
 ```
 */
export function filenameForChokidarPath(
  {
    dir,
    path,
  }: {
    readonly dir: string;
    readonly path: string;
  },
): string {
  /**
   Relative event path preserving nested child paths under the watched root.
   */
  const filename = relative(
    dir,
    path,
  );
  if (filename === '')
    return '.';

  return filename;
}

/**
 Creates completion and watcher-teardown helpers for one watched directory.
 
 @returns Lifecycle helpers closed over one watcher state holder.
 
 @example
 ```ts
 const lifecycle = createWatchDirectoryLifecycle();
 lifecycle.setWatcher({ watcher });
 await lifecycle.resolveAfterClose();
 ```
 */
export function createWatchDirectoryLifecycle(): WatchDirectoryLifecycle {
  /**
   Resolver pair for abort or failure completion of this watcher.
   */
  const completion = Promise.withResolvers<void>();
  /**
   Single-key state preventing abort, chokidar error, and dispatch error from settling twice.
   */
  const settledState = new Map<'settled', true>();
  /**
   Chokidar watcher holder, populated after setup succeeds.
   */
  const watcherState = new Map<'watcher', FSWatcher>();

  /**
   Returns whether one completion path already settled.
   
   @returns Whether completion was already resolved or rejected.
   
   @example
   ```ts
   const done = alreadySettled();
   ```
   */
  function alreadySettled(): boolean {
    return settledState.has('settled',);
  }

  /**
   Marks completion as settled if it was still pending.
   
   @returns Whether this call won the settlement race.
   
   @example
   ```ts
   const first = settleOnce();
   ```
   */
  function settleOnce(): boolean {
    if (alreadySettled())
      return false;

    settledState.set(
      'settled',
      true,
    );
    return true;
  }

  /**
   Closes active chokidar watcher if setup reached watcher creation.
   
   @example
   ```ts
   await closeActiveWatcher();
   ```
   */
  async function closeActiveWatcher(): Promise<void> {
    /**
     Active chokidar watcher, absent only when setup failed before creation.
     */
    const activeWatcher = watcherState.get('watcher',);
    if (activeWatcher === undefined)
      return;

    await activeWatcher.close();
  }

  /**
   Resolves completion after closing chokidar for normal abort teardown.
   
   @example
   ```ts
   await resolveAfterClose();
   ```
   */
  async function resolveAfterClose(): Promise<void> {
    if (!settleOnce())
      return;

    try {
      await closeActiveWatcher();
    }
    catch (closeError: unknown) {
      completion.reject(closeError,);
      return;
    }

    completion.resolve();
  }

  /**
   Rejects completion after closing chokidar for watcher failure paths.
   
   @param watchError - Failure that should reject the watch loop.
   
   @mutates watchError through completion.reject rejection retention
   
   @example
   ```ts
   await rejectAfterClose({ watchError });
   ```
   */
  async function rejectAfterClose(
    { watchError, }: { readonly watchError: unknown; },
  ): Promise<void> {
    if (!settleOnce())
      return;

    try {
      await closeActiveWatcher();
    }
    catch (closeError: unknown) {
      completion.reject(closeError,);
      return;
    }

    completion.reject(watchError,);
  }

  return {
    completion: completion.promise,

    setWatcher(
      { watcher, }: { readonly watcher: FSWatcher; },
    ): void {
      watcherState.set(
        'watcher',
        watcher,
      );
    },

    resolveAfterClose,

    rejectAfterClose,
  };
}
