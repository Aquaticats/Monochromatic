import {
  readFile,
  writeFile,
} from 'node:fs/promises';

import { tagged, } from '@monochromatic-dev/logger-core/ts';

import {
  PROCESS_ABSENT,
  readLinuxProcessIdentity,
} from './linux-process-identity.ts';

/**
 Module logger for the sentinel marker.
 */
const l = tagged({ tag: 'bypass-watch-sentinel-marker', },);

/**
 What reading the marker found:
 the marker,
 or nothing usable.
 A union on `found` is used and not a shared symbol,
 because the built bundles each hold their own copy of any symbol.
 */
export type SentinelMarkerRead =
  | { readonly found: false; }
    | {
      readonly found: true;
      readonly marker: SentinelMarker
    };

/**
 What a running sentinel leaves for the process that started it,
 so the launcher can confirm the sentinel got as far as running
 and not only that its process was created.
 */
export type SentinelMarker = {
  /**
   Process identifier of the watcher the sentinel waits for.
   */
  readonly watcherPid: number;

  /**
   Process identifier of the sentinel.
   */
  readonly pid: number;

  /**
   Start time of the sentinel process,
   so a reused pid is not mistaken for it.
   */
  readonly startTime: string;
};

/**
 Resolves the marker file of the sentinel for a state path.

 @param statePath - Persisted bypass state path.

 @returns Marker path beside the state.

 @example
 ```ts
 sentinelMarkerPath({ statePath: '/run/wg-quicker/wg0.json' });
 ```
 */
export function sentinelMarkerPath({ statePath, }: { readonly statePath: string; },): string {
  return `${statePath}.sentinel.json`;
}

/**
 Writes the marker of the running sentinel.

 @param statePath - Persisted bypass state path.

 @param watcherPid - Process identifier of the watcher the sentinel waits for.

 @throws When the sentinel process is not in procfs or the file cannot be written.

 @example
 ```ts
 await writeSentinelMarker({ statePath, watcherPid: 4242 });
 ```
 */
export async function writeSentinelMarker(
  {
    statePath,
    watcherPid,
  }: {
    readonly statePath: string;
    readonly watcherPid: number;
  },
): Promise<void> {
  /**
   Procfs identity of this process.
   */
  const own = await readLinuxProcessIdentity({ pid: process.pid, },);
  if (own === PROCESS_ABSENT)
    throw new Error('The sentinel process is not in procfs.',);
  await writeFile(
    sentinelMarkerPath({ statePath, },),
    JSON.stringify({
      watcherPid,
      pid: process.pid,
      startTime: own.startTime,
    } satisfies SentinelMarker,),
    { mode: 0o600, },
  );
}

/**
 Reads the marker of the sentinel.

 @param statePath - Persisted bypass state path.

 @returns The marker,
 or that none was found because there is none or it is malformed.

 @example
 ```ts
 await readSentinelMarker({ statePath });
 ```
 */
export async function readSentinelMarker(
  { statePath, }: { readonly statePath: string; },
): Promise<SentinelMarkerRead> {
  try {
    /**
     Parsed content of the marker file,
     checked field by field.
     */
    const parsed: unknown = JSON.parse(await readFile(
      sentinelMarkerPath({ statePath, },),
      'utf8',
    ),);
    if (((typeof parsed) === 'object') && (parsed !== null)
      && ('watcherPid' in parsed)
      && ('pid' in parsed)
      && ('startTime' in parsed)
      && ((typeof parsed.watcherPid) === 'number')
      && ((typeof parsed.pid) === 'number')
      && ((typeof parsed.startTime) === 'string'))
      return {
        found: true,
        marker: {
          watcherPid: parsed.watcherPid,
          pid: parsed.pid,
          startTime: parsed.startTime,
        },
      };
    return { found: false, };
  }
  catch (error) {
    l.debug(`no readable sentinel marker for ${statePath}: ${String(error,)}`,);
    return { found: false, };
  }
}
