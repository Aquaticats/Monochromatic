/**
 A wake-up for the first change in a directory, so a waiting gate learns of a released slot at once instead of by polling.

 The watch is established before the caller looks at the directory, so a change that happens between the look and the wait is
 not lost. It reports only that something changed; the caller looks again. A holder that dies makes no event, so the watch
 never replaces a bounded re-check, it only makes the common case (a gate that finished and released its slot) prompt.
 */
import { watch } from 'node:fs';

/**
 A watch on one directory.
 */
export type DirectoryWatch = {
  /**
   Settles on the first change in the directory; stays pending when nothing changes, after the watch was closed, and when
   the directory could not be watched.
   */
  readonly changed: Promise<void>;
  /**
   Stop watching.
   */
  readonly close: () => void;
  /**
   Stop watching when the scope ends.
   */
  readonly [Symbol.dispose]: () => void;
};

/**
 Stop nothing, for a directory that could not be watched.
 */
function closeNothing(): void {
  // The watcher never started.
}

/**
 Start the watcher on a directory.

 @param directory - the directory to watch

 @param onChange - called on every change the kernel reports

 @returns the function that stops the watcher, or one that stops nothing when the directory cannot be watched

 @example
 ```ts
 const close = startWatcher({ directory: '/l', onChange: () => {} });
 ```
 */
function startWatcher({
  directory,
  onChange,
}: {
  readonly directory: string;
  readonly onChange: () => void;
}): () => void {
  try {
    /**
     The watcher, which does not keep the process alive by itself.
     */
    const watcher = watch(
      directory,
      { persistent: false },
      onChange,
    );
    watcher.on(
      'error',
      function onError(error) {
        console.error(`The watch on ${directory} failed (${String(error)}); the bounded re-check decides instead.`);
        watcher.close();
      },
    );
    return watcher.close
      .bind(watcher);
  }
  catch (error) {
    console.error(`${directory} cannot be watched (${String(error)}); the bounded re-check decides instead.`);
    return closeNothing;
  }
}

/**
 Watch a directory for the first change.

 @param directory - the directory to watch

 @returns the watch; declare it with `using` so it is closed when the wait ends

 @example
 ```ts
 using watch = watchDirectoryChanges('/l/gate-slots');
 ```
 */
export function watchDirectoryChanges(directory: string): DirectoryWatch {
  /**
   The promise of the first change, settled by the watcher's callback.
   */
  const {
    promise,
    resolve
  } = Promise.withResolvers<void>();
  /**
   Stops the watcher.
   */
  const close = startWatcher({
    directory,
    onChange: resolve,
  });
  return {
    changed: promise,
    close,
    [Symbol.dispose]: close,
  };
}
