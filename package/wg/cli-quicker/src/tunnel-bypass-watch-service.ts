import { spawn as spawnChild, } from 'node:child_process';
import { rm, } from 'node:fs/promises';

import { wait, } from '@monochromatic-dev/async-time-core/ts';
import { tagged, } from '@monochromatic-dev/logger-core/ts';

import { BypassRouteError, } from './errors.ts';
import {
  PROCESS_ABSENT,
  processArgumentsMatch,
  readLinuxProcessIdentity,
} from './linux-process-identity.ts';
import { bypassStatePath, } from './tunnel-bypass-path.ts';
import type { BypassState, } from './tunnel-bypass-types.ts';
import {
  readWatcherRecord,
  type WatcherProcessIdentity,
  watcherIdentityPath,
  writeWatcherRecord,
} from './tunnel-bypass-watcher-sidecar.ts';
import { identityOrNone, } from './tunnel-bypass-watch-record.ts';
import {
  PROCESS_WAIT_ATTEMPTS,
  PROCESS_WAIT_DELAY_MS,
  signalWatcherProcessGroup,
  stopLeftoverGroup,
  waitForWatcherStop,
  WATCHER_PATH,
  watcherIsRunning,
} from './tunnel-bypass-watch-process-group.ts';
import { startSentinelOrReport, } from './bypass-watch-sentinel-launch.ts';

/**
 Module logger for the watcher record and the watcher's process group.
 */
const l = tagged({ tag: 'tunnel-bypass-watch-service', },);
/**
 Registers current watcher identity and removes it on exit.
 
 @param statePath - Persisted bypass state path.
 
 @param ownerId - State owner binding sidecar to tunnel lifecycle.
 
 @returns Asynchronous cleanup guard.
 
 @example
 ```ts
 await using registration = await registerBypassWatcher({ statePath, ownerId: 'owner' });
 ```
 */
export async function registerBypassWatcher(
  {
    statePath,
    ownerId,
  }: {
    readonly statePath: string;
    readonly ownerId: string;
  },
): Promise<AsyncDisposable> {
  /**
   Current watcher process identity.
   */
  const live = await readLinuxProcessIdentity({ pid: process.pid, },);
  if (live === PROCESS_ABSENT)
    throw new BypassRouteError('Current watcher process disappeared during registration.',);
  /**
   Sidecar path owned by this watcher.
   */
  const path = watcherIdentityPath({ statePath, },);
  await writeWatcherRecord({
    statePath,
    identity: {
      ownerId,
      pid: process.pid,
      startTime: live.startTime,
    },
  },);
  return {
    async [Symbol.asyncDispose](): Promise<void> {
      /**
       Current sidecar identity before owner removal.
       */
      const record = await readWatcherRecord({ statePath, },);
      if (record.kind === 'identity') {
        /**
         Identity the record holds,
         which is this watcher's when all three parts match.
         */
        const { identity: recordedIdentity, } = record;
        if ((recordedIdentity.ownerId === ownerId)
          && (recordedIdentity.pid === process.pid)
          && (recordedIdentity.startTime === live.startTime))
          await rm(path,);
      }
    },
  };
}

/**
 Stops detached watcher only when owner,
 PID start time,
 and complete command identity match.
 A watcher whose leader is already gone has its process group ended,
 so the route monitor it spawned does not outlive it.

 @param state - Persisted bypass state.

 @param watcherPath - Expected watcher script argument,
 replaceable in tests.

 @example
 ```ts
 await stopBypassWatcher({ state });
 ```
 */
export async function stopBypassWatcher(
  {
    state,
    watcherPath = WATCHER_PATH,
  }: {
    readonly state: BypassState;
    readonly watcherPath?: string;
  },
): Promise<void> {
  /**
   Persisted state path and watcher identity.
   */
  const statePath = bypassStatePath({ interfaceName: state.interfaceName, },);
  /**
   Registered watcher when present,
   with a record that holds no identity moved aside instead of refusing the stop for ever.
   */
  const identity = await identityOrNone({ statePath, },);
  if (identity === PROCESS_ABSENT)
    return;
  if (identity.ownerId !== state.ownerId) {
    throw new BypassRouteError(`Refusing to stop bypass watcher owned by ${identity.ownerId}.`,);
  }
  if (!(await watcherIsRunning({
    identity,
    statePath,
    watcherPath,
  }))) {
    // The leader is gone, and a route monitor it spawned can outlive it:
    // the monitor and every `ip` call of the watcher are children in the leader's process group,
    // and nothing else ever ends that group.
    await stopLeftoverGroup({
      identity,
      statePath,
      watcherPath,
    },);
    await rm(
      watcherIdentityPath({ statePath, },),
      { force: true, },
    );
    return;
  }
  signalWatcherProcessGroup({
    pid: identity.pid,
    signal: 'SIGTERM',
  },);
  if (!(await waitForWatcherStop({
    identity,
    statePath,
    watcherPath,
  }))) {
    if (!(await watcherIsRunning({
      identity,
      statePath,
      watcherPath,
    }))) {
      await stopLeftoverGroup({
        identity,
        statePath,
        watcherPath,
      },);
      await rm(
        watcherIdentityPath({ statePath, },),
        { force: true, },
      );
      return;
    }
    signalWatcherProcessGroup({
      pid: identity.pid,
      signal: 'SIGKILL',
    },);
    if (!(await waitForWatcherStop({
      identity,
      statePath,
      watcherPath,
    }))) {
      throw new BypassRouteError(`Bypass watcher PID ${String(identity.pid,)} survived SIGKILL.`,);
    }
  }
  await rm(
    watcherIdentityPath({ statePath, },),
    { force: true, },
  );
}

/**
 Starts detached watcher in caller's network namespace and verifies registration.
 
 @param state - Persisted bypass state.
 
 @example
 ```ts
 await startBypassWatcher({ state });
 ```
 */
export async function startBypassWatcher(
  { state, }: { readonly state: BypassState; },
): Promise<void> {
  await stopBypassWatcher({ state, },);
  /**
   Persisted state path passed to watcher.
   */
  const statePath = bypassStatePath({ interfaceName: state.interfaceName, },);
  /**
   Detached watcher inheriting caller's network namespace and privileges.
   */
  const watcher = spawnChild(
    process.execPath,
    [
      WATCHER_PATH,
      statePath,
    ],
    {
      detached: true,
      stdio: 'ignore',
    },
  );
  watcher.unref();
  /**
   Spawn failures captured without unhandled process event.
   */
  const spawnFailureMessages: string[] = [];
  watcher.once(
    'error',
    function captureSpawnError(error: Readonly<Error>,): void {
      spawnFailureMessages[0] = error.message;
    },
  );
  /**
   Child PID validated before any signal path.
   */
  const { pid, } = watcher;
  if ((pid === undefined) || (pid <= 0))
    throw new BypassRouteError('Bypass route watcher did not receive a positive PID.',);
  /**
   Bounded readiness cursor.
   */
  const cursor = { attempt: 0, };
  while (cursor.attempt < PROCESS_WAIT_ATTEMPTS) {
    /**
     First spawn failure when detached child could not start.
     */
    const [spawnFailureMessage,] = spawnFailureMessages;
    if (spawnFailureMessage !== undefined)
      throw new BypassRouteError(`Bypass route watcher spawn failed: ${spawnFailureMessage}`,);
    /**
     Registration sidecar observed during bounded readiness wait.
     */
    // oxlint-disable-next-line eslint/no-await-in-loop -- Registration sidecar is asynchronous child readiness boundary.
    const identity = await identityOrNone({ statePath, },);
    if ((identity !== PROCESS_ABSENT)
      && (identity.ownerId === state.ownerId)
      && (identity.pid === pid)) {
      // oxlint-disable-next-line eslint/no-await-in-loop -- The sentinel starts once, when the watcher is registered, and ends the loop.
      await startSentinelOrReport({
        statePath,
        identity,
      },);
      return;
    }
    if (watcher.exitCode !== null)
      throw new BypassRouteError(`Bypass route watcher exited during startup with ${String(watcher.exitCode,)}.`,);
    // oxlint-disable-next-line eslint/no-await-in-loop -- Bounded readiness wait avoids busy spin.
    await wait(PROCESS_WAIT_DELAY_MS,);
    cursor.attempt += 1;
  }
  /**
   Timed-out child process checked before signaling.
   */
  const live = await readLinuxProcessIdentity({ pid, },);
  if (live === PROCESS_ABSENT)
    throw new BypassRouteError('Bypass route watcher disappeared before readiness.',);
  if (!processArgumentsMatch({
    identity: live,
    expected: [
      WATCHER_PATH,
      statePath,
    ],
  },)) {
    throw new BypassRouteError(`Timed-out watcher PID ${String(pid,)} no longer identifies expected command.`,);
  }
  /**
   Exact timed-out watcher identity used for group shutdown confirmation.
   */
  const identity: WatcherProcessIdentity = {
    ownerId: state.ownerId,
    pid,
    startTime: live.startTime,
  };
  signalWatcherProcessGroup({
    pid,
    signal: 'SIGKILL',
  },);
  if (!(await waitForWatcherStop({
    identity,
    statePath,
  }))) {
    await writeWatcherRecord({
      statePath,
      identity,
    },);
    throw new BypassRouteError(`Timed-out bypass watcher group ${String(pid,)} survived SIGKILL; ownership retained.`,);
  }
  throw new BypassRouteError('Bypass route watcher did not register process identity.',);
}
