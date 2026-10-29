/**
 Config-file watching for multi-process change propagation.
 
 Mirrors upstream `conf` 15.1.0's platform split:
 directory watching with a short debounce on Windows and macOS (atomic
 writes arrive as rename events on the directory),
 file watching with a longer debounce elsewhere,
 where upstream found `fs.watchFile` more reliable.
 
 @module
 */

import path from 'node:path';
import process from 'node:process';
import {
  unwatchFile,
  watch,
  watchFile,
} from 'node:fs';
import {
  tagged,
  type Logger,
} from '@monochromatic-dev/logger-core/ts';

import { debounce, } from './debounce.ts';

//region Types

/**
 One running watch with its teardown.
 
 @example
 ```ts
 const watcher = createConfigWatcher({ path: configPath, onChange: handleChange, });
 watcher.close();
 ```
 */
export type ConfigWatcher = {
  /**
   Stops watching and drops any pending debounced run.
   */
  readonly close: () => void;
};

//endregion Types

//region Constants

/**
 Debounce for directory-event platforms,
 where rename storms from atomic writes arrive in bursts.
 
 @example
 ```ts
 DIRECTORY_EVENT_DEBOUNCE_WAIT; // 100
 ```
 */
const DIRECTORY_EVENT_DEBOUNCE_WAIT = 100;

/**
 Debounce for polling platforms,
 sized to the `fs.watchFile` polling cadence.
 
 @example
 ```ts
 FILE_EVENT_DEBOUNCE_WAIT; // 1000
 ```
 */
const FILE_EVENT_DEBOUNCE_WAIT = 1_000;

//endregion Constants

//region Factory

/**
 Starts watching one config file,
 coalescing change bursts into `onChange` calls.
 
 @param path - Absolute config file path.
 
 @param onChange - Called once after the file settles.
 
 @param logger - Logger for watch lifecycle diagnostics.
 
 @returns Watcher whose `close` stops the underlying watch.
 
 @example
 ```ts
 const watcher = createConfigWatcher({
   path: '/tmp/app/config.json',
   onChange: function onChange(): void {
     console.log('config changed');
   },
 });
 ```
 */
export function createConfigWatcher({
  path: filePath,
  onChange,
  logger,
}: {
  readonly path: string;
  readonly onChange: () => void;
  readonly logger?: Logger;
},): ConfigWatcher {
  /**
   Logger wrapped with this factory's name so watch diagnostics name their
   origin.
   */
  const log = logger ?? tagged({
    tag: createConfigWatcher.name,
  },);
  /**
   Directory-event watcher kept for teardown on the platforms that use it.
   */
  const state: {
    directoryWatcher?: ReturnType<typeof watch>;
  } = {};
  if ((process.platform === 'win32') || (process.platform === 'darwin')) {
    /**
     Debounced change reporter for directory event bursts.
     */
    const debounced = debounce({
      fn: onChange,
      wait: DIRECTORY_EVENT_DEBOUNCE_WAIT,
    },);
    /**
     Directory holding the config file; watched instead of the file so
     atomic renames are observable.
     */
    const directory = path.dirname(filePath,);
    /**
     File name events must match to count as this config's changes.
     */
    const basename = path.basename(filePath,);
    state.directoryWatcher = watch(
      directory,
      {
        persistent: false,
        encoding: 'utf8',
      },
      function onDirectoryEvent(
        _eventType: string,
        // oxlint-disable-next-line no-restricted-syntax/no-nullish-union -- mirrors node:fs WatchListener<string>, whose filename parameter is string | null by Node's own signature.
        filename: string | null,
      ): void {
        if ((filename !== null) && (filename !== basename))
          return;
        debounced.trigger();
      },
    );
    return {
      close: function closeDirectoryWatcher(): void {
        debounced.cancel();
        state.directoryWatcher
          ?.close();
        delete state.directoryWatcher;
      },
    };
  }
  /**
   Debounced change reporter for file polling events.
   */
  const debounced = debounce({
    fn: onChange,
    wait: FILE_EVENT_DEBOUNCE_WAIT,
  },);
  watchFile(
    filePath,
    {
      persistent: false,
    },
    function onFileEvent(): void {
      debounced.trigger();
    },
  );
  log.debug(`watching ${filePath} via fs.watchFile`,);
  return {
    close: function closeFileWatcher(): void {
      debounced.cancel();
      unwatchFile(filePath,);
    },
  };
}

//endregion Factory
