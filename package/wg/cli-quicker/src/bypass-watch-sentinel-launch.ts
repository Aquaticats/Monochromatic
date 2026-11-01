import { spawn as spawnChild, } from 'node:child_process';
import { once, } from 'node:events';

import { tagged, } from '@monochromatic-dev/logger-core/ts';

import type { WatcherProcessIdentity, } from './tunnel-bypass-watcher-sidecar.ts';
import {
  POLICY_BYPASS_TABLE,
  policyNotInForce,
} from './update-policy-notice.ts';

/**
 Module logger for the sentinel launch.
 */
const l = tagged({ tag: 'bypass-watch-sentinel-launch', },);

/**
 Source or built sentinel filename adjacent to current artifact.
 */
const SENTINEL_FILENAME = import.meta.url
  .endsWith('.ts',)
  ? 'bypass-watch-sentinel.ts'
  : 'bypass-watch-sentinel.mjs';

/**
 Absolute sentinel entry path.
 */
const SENTINEL_PATH = new URL(
  SENTINEL_FILENAME,
  import.meta.url,
).pathname;

/**
 Starts the detached sentinel that reports the death of a registered watcher.

 @param statePath - Persisted bypass state path.

 @param identity - Registered watcher identity.

 @throws When the sentinel process cannot start.

 @example
 ```ts
 await startWatcherSentinel({ statePath, identity });
 ```
 */
async function startWatcherSentinel(
  {
    statePath,
    identity,
  }: {
    readonly statePath: string;
    readonly identity: WatcherProcessIdentity;
  },
): Promise<void> {
  /**
   Detached sentinel inheriting the environment that names the sudo caller.
   */
  const sentinel = spawnChild(
    process.execPath,
    [
      SENTINEL_PATH,
      statePath,
      identity.ownerId,
      String(identity.pid,),
      identity.startTime,
    ],
    {
      detached: true,
      stdio: 'ignore',
    },
  );
  sentinel.unref();
  await once(
    sentinel,
    'spawn',
  );
}

/**
 Starts the sentinel,
 and says loudly when it cannot start without failing the tunnel start,
 because the watcher itself is running.

 @param statePath - Persisted bypass state path.

 @param identity - Registered watcher identity.

 @example
 ```ts
 await startSentinelOrReport({ statePath, identity });
 ```
 */
export async function startSentinelOrReport(
  {
    statePath,
    identity,
  }: {
    readonly statePath: string;
    readonly identity: WatcherProcessIdentity;
  },
): Promise<void> {
  try {
    await startWatcherSentinel({
      statePath,
      identity,
    },);
  }
  catch (error) {
    l.error(policyNotInForce({
      policy: POLICY_BYPASS_TABLE,
      detail: `the sentinel that reports the route watcher's own death could not start (${String(error,)}), `
        + 'so if the watcher is killed or runs out of memory nothing will say so',
    },),);
  }
}
