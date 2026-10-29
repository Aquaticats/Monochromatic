import { readFile, } from 'node:fs/promises';

import { BypassRouteError, } from './errors.ts';
import { PROCESS_ABSENT, } from './linux-process-identity.ts';

/**
 Persisted watcher process identity resistant to PID reuse.
 */
export type WatcherProcessIdentity = {
  readonly ownerId: string;
  readonly pid: number;
  readonly startTime: string;
};

/**
 Narrows caught value to Node filesystem error.
 
 @param error - Caught value.
 
 @returns Whether value carries error code.
 
 @example
 ```ts
 isErrnoException({ code: 'ENOENT' }); // true
 ```
 */
function isErrnoException(error: unknown,): error is NodeJS.ErrnoException {
  return ((typeof error) === 'object')
    && (error !== null)
    && ('code' in error);
}

/**
 Resolves watcher identity sidecar for state path.
 
 @param statePath - Persisted bypass state path.
 
 @returns Watcher identity path.
 
 @example
 ```ts
 watcherIdentityPath({ statePath: '/run/wg-quicker/interface.json' });
 ```
 */
export function watcherIdentityPath(
  { statePath, }: { readonly statePath: string; },
): string {
  return `${statePath}.watcher.json`;
}

/**
 Parses watcher sidecar JSON with lifecycle diagnostics.
 
 @param text - Sidecar JSON text.
 
 @param statePath - State path named in diagnostics.
 
 @returns Parsed unknown value.
 
 @throws {@link BypassRouteError} when JSON is malformed.
 
 @example
 ```ts
 parseWatcherJson({ text: '{}', statePath: '/tmp/state' });
 ```
 */
function parseWatcherJson(
  {
    text,
    statePath,
  }: {
    readonly text: string;
    readonly statePath: string;
  },
): unknown {
  try {
    return JSON.parse(text,);
  }
  catch (error) {
    throw new BypassRouteError(`Invalid bypass watcher identity for ${statePath}: ${String(error,)}`,);
  }
}

/**
 Parses watcher sidecar JSON and shape.
 
 @param text - Sidecar JSON text.
 
 @param statePath - State path named in diagnostics.
 
 @returns Validated watcher identity.
 
 @throws {@link BypassRouteError} when JSON or fields are invalid.
 
 @example
 ```ts
 parseWatcherIdentity({ text: '{"pid":1}', statePath: '/tmp/state' });
 ```
 */
function parseWatcherIdentity(
  {
    text,
    statePath,
  }: {
    readonly text: string;
    readonly statePath: string;
  },
): WatcherProcessIdentity {
  /**
   Parsed sidecar before shape checks.
   */
  const value: unknown = parseWatcherJson({
    text,
    statePath,
  },);
  if (((typeof value) !== 'object')
    || (value === null)
    || (!('ownerId' in value))
    || (!('pid' in value))
    || (!('startTime' in value))
    || ((typeof value.ownerId) !== 'string')
    || (value.ownerId === '')
    || ((typeof value.pid) !== 'number')
    || (!Number.isSafeInteger(value.pid,))
    || (value.pid <= 0)
    || ((typeof value.startTime) !== 'string')
    || (value.startTime === '')) {
    throw new BypassRouteError(`Invalid bypass watcher identity for ${statePath}.`,);
  }
  return {
    ownerId: value.ownerId,
    pid: value.pid,
    startTime: value.startTime,
  };
}

/**
 Reads watcher identity sidecar when present.
 
 @param statePath - Persisted bypass state path.
 
 @returns Validated identity or absence.
 
 @example
 ```ts
 await readWatcherIdentity({ statePath: '/run/wg-quicker/interface.json' });
 ```
 */
export async function readWatcherIdentity(
  { statePath, }: { readonly statePath: string; },
): Promise<WatcherProcessIdentity | typeof PROCESS_ABSENT> {
  try {
    return parseWatcherIdentity({
      text: await readFile(
        watcherIdentityPath({ statePath, },),
        'utf8',
      ),
      statePath,
    },);
  }
  catch (error) {
    if (isErrnoException(error,) && (error.code === 'ENOENT'))
      return PROCESS_ABSENT;
    throw error;
  }
}
