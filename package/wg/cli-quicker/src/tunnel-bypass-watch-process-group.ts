//! The process group of a route watcher:
//! the leader itself,
//! the children it spawns (`ip monitor route` and its own `ip` calls),
//! and what a stop must do about them.
//! A watcher leads its own group (`setsid`),
//! so `SIGTERM` and `SIGKILL` to the group reach the monitor as well,
//! and a leader that died alone leaves the monitor in that group.

import { wait, } from '@monochromatic-dev/async-time-core/ts';

import {
  BypassRouteError,
  isErrnoException,
} from './errors.ts';
import {
  PROCESS_ABSENT,
  PROCESS_ZOMBIE_STATE,
  processArgumentsMatch,
  readLinuxProcessIdentity,
  readProcessGroupMembers,
} from './linux-process-identity.ts';
import type { WatcherProcessIdentity, } from './tunnel-bypass-watcher-sidecar.ts';

/**
 Source or built watcher filename adjacent to current artifact.
 */
const WATCHER_FILENAME: string = import.meta.url
  .endsWith('.ts',)
  ? 'bypass-watch.ts'
  : 'bypass-watch.mjs';

/**
 Absolute watcher entry path.
 */
export const WATCHER_PATH: string = new URL(
  WATCHER_FILENAME,
  import.meta.url,
).pathname;

/**
 Readiness and shutdown retry count.
 */
export const PROCESS_WAIT_ATTEMPTS = 100;

/**
 Delay between bounded process identity probes.
 */
export const PROCESS_WAIT_DELAY_MS = 10;

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
    || (live.state === PROCESS_ZOMBIE_STATE)) {
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
export function signalWatcherProcessGroup(
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
 Reports whether a detached watcher group still holds a member that can run.

 A member that exited released everything the group could hold,
 the route monitor's netlink socket included,
 and only its task entry waits for whoever adopted it to reap it.
 The kernel keeps answering `kill(-pgid, 0)` until that reap,
 so a check of the group's existence alone calls a monitor that has gone still running
 and fails a stop that has nothing left to stop.

 @param pid - Positive process-group leader identity.

 @param members - Reader of the group's members,
 replaceable in tests.

 @returns Whether a member is in a state other than `Z`.

 @example
 ```ts
 await watcherProcessGroupHasRunningMember({ pid: 123 });
 ```
 */
export async function watcherProcessGroupHasRunningMember(
  {
    pid,
    members = readProcessGroupMembers,
  }: {
    readonly pid: number;
    readonly members?: typeof readProcessGroupMembers;
  },
): Promise<boolean> {
  if ((!Number.isSafeInteger(pid,)) || (pid <= 0))
    throw new BypassRouteError(`Invalid process-group identifier: ${String(pid,)}`,);
  return (await members({ pgid: pid, },))
    .some(function stillRunning(member,): boolean {
      return member.state !== PROCESS_ZOMBIE_STATE;
    },);
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
 Answers that the watcher is not running,
 for the wait that only watches its process group.
 
 @returns A resolved `false`.

 @example
 ```ts
 await alreadyGone();
 ```
 */
function alreadyGone(): Promise<boolean> {
  return Promise.resolve(false,);
}

/**
 Ends the process group of a watcher whose leader is gone.
 
 The watcher starts `ip monitor route` and its own `ip` calls as children,
 so they are members of the group the watcher leads,
 and a `SIGKILL` to the watcher alone leaves them running:
 the monitor keeps its netlink socket and nothing else ever ends it.
 The group is signalled as a whole, which is what the live path does,
 and the wait is the same bounded one.
 
 A pid the kernel has given to another process is never signalled:
 while a process group still exists its number cannot be reused,
 so a live process at that pid with another start time means the group is another one.

 @param identity - Recorded watcher identity,
 whose pid is the group.

 @param statePath - Expected command state argument.

 @param watcherPath - Expected watcher script argument,
 replaceable in tests.

 @param read - Reader of the live process,
 replaceable in tests.

 @param groupExists - Process-group check,
 replaceable in tests.

 @param groupHasRunningMember - Check for a member that can still run,
 replaceable in tests.

 @param pause - Wait between checks,
 replaceable in tests.

 @throws {@link BypassRouteError} when the group survives `SIGKILL`.

 @example
 ```ts
 await stopLeftoverGroup({ identity, statePath: '/run/wg-quicker/interface.json' });
 ```
 */
export async function stopLeftoverGroup(
  {
    identity,
    statePath,
    watcherPath = WATCHER_PATH,
    read = readLinuxProcessIdentity,
    groupExists = watcherProcessGroupExists,
    groupHasRunningMember = watcherProcessGroupHasRunningMember,
    pause = wait,
  }: {
    readonly identity: WatcherProcessIdentity;
    readonly statePath: string;
    readonly watcherPath?: string;
    readonly read?: typeof readLinuxProcessIdentity;
    readonly groupExists?: typeof watcherProcessGroupExists;
    readonly groupHasRunningMember?: typeof watcherProcessGroupHasRunningMember;
    readonly pause?: (milliseconds: number,) => Promise<unknown>;
  },
): Promise<void> {
  if (!groupExists({ pid: identity.pid, }))
    return;
  /**
   The process at the recorded pid,
   when the kernel has given its number to another one.
   */
  const live = await read({ pid: identity.pid, },);
  if ((live !== PROCESS_ABSENT) && (live.startTime !== identity.startTime))
    return;
  signalWatcherProcessGroup({
    pid: identity.pid,
    signal: 'SIGTERM',
  },);
  if (await waitForWatcherStop({
    identity,
    statePath,
    watcherPath,
    isRunning: alreadyGone,
    groupExists,
    pause,
  },))
    return;
  signalWatcherProcessGroup({
    pid: identity.pid,
    signal: 'SIGKILL',
  },);
  if (!(await waitForWatcherStop({
    identity,
    statePath,
    watcherPath,
    isRunning: alreadyGone,
    groupExists,
    pause,
  },))) {
    // A member that has exited cannot run anything and cannot hold the monitor's socket,
    // so a group of them is gone for every purpose this program has:
    // the kernel only keeps answering the group until whoever adopted the member reaps it,
    // and that moment is not this program's to wait for.
    if (!(await groupHasRunningMember({ pid: identity.pid, },)))
      return;
    throw new BypassRouteError(
      `The process group of bypass watcher PID ${String(identity.pid,)} outlived it and survived SIGKILL; the record is kept.`,
    );
  }
}
