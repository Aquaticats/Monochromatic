import { spawn as spawnChild, } from 'node:child_process';
import {
  rm,
  writeFile,
} from 'node:fs/promises';

import { wait, } from '@monochromatic-dev/async-time-core/ts';

import {
  BypassRouteError,
  isErrnoException,
} from './errors.ts';
import {
  PROCESS_ABSENT,
  processArgumentsMatch,
  readLinuxProcessIdentity,
  type LinuxProcessIdentity,
} from './linux-process-identity.ts';
import { bypassStatePath, } from './tunnel-bypass-path.ts';
import type { BypassState, } from './tunnel-bypass-types.ts';
import {
  readWatcherIdentity,
  watcherIdentityPath,
  type WatcherProcessIdentity,
} from './tunnel-bypass-watcher-sidecar.ts';
import { startSentinelOrReport, } from './bypass-watch-sentinel-launch.ts';

/**
 Source or built watcher filename adjacent to current artifact.
 */
const WATCHER_FILENAME = import.meta.url
  .endsWith('.ts',)
  ? 'bypass-watch.ts'
  : 'bypass-watch.mjs';

/**
 Absolute watcher entry path.
 */
const WATCHER_PATH = new URL(
  WATCHER_FILENAME,
  import.meta.url,
).pathname;

/**
 Readiness and shutdown retry count.
 */
const PROCESS_WAIT_ATTEMPTS = 100;

/**
 Delay between bounded process identity probes.
 */
const PROCESS_WAIT_DELAY_MS = 10;

/**
 Checks live process has exact watcher script and state arguments.

 A process that is exiting is not a different command:
 once the kernel has released its address space,
 `/proc/PID/cmdline` reads empty while the state still reads as live
 (measured: a `node` process holding 200 MB showed state `R` with an empty command line for hundreds of consecutive reads, in four of six runs),
 and that window falls inside the shutdown wait after a `SIGTERM`.
 Reading it as another command made `down` refuse a watcher it had just stopped
 and leave a half-removed tunnel
 (live incident of 2026-10-31).
 Its start time already identifies the process,
 so an empty command line counts as a watcher that is no longer running,
 and the shutdown wait still waits for its process group to disappear.

 @param identity - Persisted watcher identity.

 @param statePath - Exact state argument expected in command line.

 @param watcherPath - Exact watcher script expected in command line,
 replaceable in tests.

 @param read - Reader of the live process,
 replaceable in tests.

 @returns Whether same watcher remains live.

 @throws {@link BypassRouteError} when PID names another live command.

 @example
 ```ts
 await watcherIsRunning({ identity, statePath: '/tmp/state' });
 ```
 */
export async function watcherIsRunning(
  {
    identity,
    statePath,
    watcherPath = WATCHER_PATH,
    read = readLinuxProcessIdentity,
  }: {
    readonly identity: WatcherProcessIdentity;
    readonly statePath: string;
    readonly watcherPath?: string;
    readonly read?: typeof readLinuxProcessIdentity;
  },
): Promise<boolean> {
  /**
   Current process at persisted PID.
   */
  const live = await read({ pid: identity.pid, },);
  if ((live === PROCESS_ABSENT)
    || (live.startTime !== identity.startTime)
    || (live.state === 'Z')) {
    return false;
  }
  if (live.commandLine
    .length
    === 0)
    return false;
  if (!processArgumentsMatch({
    identity: live,
    expected: [
      watcherPath,
      statePath,
    ],
  },)) {
    throw new BypassRouteError(
      `Refusing to signal PID ${String(identity.pid,)} because command is not bypass watcher for ${statePath}. `
        + `It runs ${JSON.stringify(live.commandLine,)} and the watcher is expected to run its node binary, ${JSON.stringify(watcherPath,)} and ${JSON.stringify(statePath,)}.`,
    );
  }
  return true;
}

/**
 Signals detached watcher process group while tolerating disappearance.
 
 Watcher PID is validated before conversion to group target.
 Route-monitor and transient `ip` children inherit this process group.
 
 @param pid - Positive watcher group-leader PID.
 
 @param signal - Signal name.
 
 @example
 ```ts
 signalWatcherProcessGroup({ pid: 123, signal: 'SIGTERM' });
 ```
 */
function signalWatcherProcessGroup(
  {
    pid,
    signal,
  }: {
    readonly pid: number;
    readonly signal: NodeJS.Signals;
  },
): void {
  if ((!Number.isSafeInteger(pid,)) || (pid <= 0))
    throw new BypassRouteError(`Refusing to signal invalid watcher group PID ${String(pid,)}.`,);
  try {
    process.kill(
      -pid,
      signal,
    );
  }
  catch (error) {
    if (isErrnoException(error,) && (error.code === 'ESRCH'))
      return;
    throw error;
  }
}

/**
 Reports whether detached watcher process group still has members.
 
 @param pid - Positive process-group leader identity.
 
 @returns Whether kernel still resolves process group.
 
 @example
 ```ts
 watcherProcessGroupExists({ pid: 123 });
 ```
 */
export function watcherProcessGroupExists(
  { pid, }: { readonly pid: number; },
): boolean {
  if ((!Number.isSafeInteger(pid,)) || (pid <= 0))
    throw new BypassRouteError(`Invalid watcher process-group identifier: ${String(pid,)}`,);
  try {
    process.kill(
      -pid,
      0,
    );
    return true;
  }
  catch (error) {
    if (isErrnoException(error,) && (error.code === 'ESRCH'))
      return false;
    throw error;
  }
}

/**
 Waits bounded interval for exact watcher group to disappear.
 
 @param identity - Watcher identity being stopped.
 
 @param statePath - Expected command state argument.

 @param watcherPath - Expected watcher script argument,
 replaceable in tests.

 @param isRunning - Identity check,
 replaceable in tests.

 @param groupExists - Process-group check,
 replaceable in tests.

 @param pause - Wait between checks,
 replaceable in tests.

 @returns Whether watcher disappeared before bound.

 @example
 ```ts
 await waitForWatcherStop({ identity, statePath: '/tmp/state' });
 ```
 */
export async function waitForWatcherStop(
  {
    identity,
    statePath,
    watcherPath = WATCHER_PATH,
    isRunning = watcherIsRunning,
    groupExists = watcherProcessGroupExists,
    pause = wait,
  }: {
    readonly identity: WatcherProcessIdentity;
    readonly statePath: string;
    readonly watcherPath?: string;
    readonly isRunning?: typeof watcherIsRunning;
    readonly groupExists?: typeof watcherProcessGroupExists;
    readonly pause?: (milliseconds: number,) => Promise<unknown>;
  },
): Promise<boolean> {
  /**
   Bounded process-stop cursor.
   */
  const cursor = { attempt: 0, };
  while (cursor.attempt < PROCESS_WAIT_ATTEMPTS) {
    // oxlint-disable-next-line eslint/no-await-in-loop -- Exact process identity must be rechecked after each bounded delay.
    if ((!(await isRunning({
      identity,
      statePath,
      watcherPath,
    },))) && (!groupExists({ pid: identity.pid, })))
      return true;
    // oxlint-disable-next-line eslint/no-await-in-loop -- Bounded shutdown wait avoids busy spin.
    await pause(PROCESS_WAIT_DELAY_MS,);
    cursor.attempt += 1;
  }
  return false;
}

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
  await writeFile(
    path,
    JSON.stringify({
      ownerId,
      pid: process.pid,
      startTime: live.startTime,
    },),
    {
      flag: 'wx',
      mode: 0o600,
    },
  );
  return {
    async [Symbol.asyncDispose](): Promise<void> {
      /**
       Current sidecar identity before owner removal.
       */
      const current = await readWatcherIdentity({ statePath, },);
      if ((current !== PROCESS_ABSENT)
        && (current.ownerId === ownerId)
        && (current.pid === process.pid)
        && (current.startTime === live.startTime)) {
        await rm(path,);
      }
    },
  };
}

/**
 Stops detached watcher only when owner,
 PID start time,
 and complete command identity match.
 
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
   Registered watcher when present.
   */
  const identity = await readWatcherIdentity({ statePath, },);
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
    const identity = await readWatcherIdentity({ statePath, },);
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
    await writeFile(
      watcherIdentityPath({ statePath, },),
      JSON.stringify({
        ownerId: identity.ownerId,
        pid: identity.pid,
        startTime: identity.startTime,
      },),
      {
        flag: 'wx',
        mode: 0o600,
      },
    );
    throw new BypassRouteError(`Timed-out bypass watcher group ${String(pid,)} survived SIGKILL; ownership retained.`,);
  }
  throw new BypassRouteError('Bypass route watcher did not register process identity.',);
}
