import { spawn as spawnChild, } from 'node:child_process';
import { once, } from 'node:events';
import { rm, } from 'node:fs/promises';

import { wait, } from '@monochromatic-dev/async-time-core/ts';
import { tagged, } from '@monochromatic-dev/logger-core/ts';

import {
  PROCESS_ABSENT,
  readLinuxProcessIdentity,
} from './linux-process-identity.ts';
import {
  readSentinelMarker,
  sentinelMarkerPath,
} from './bypass-watch-sentinel-marker.ts';
import { run, } from './runner.ts';
import type { SudoEnvironment, } from './notification-user.ts';
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
 Checks of the marker before the sentinel is called unconfirmed.
 */
const CONFIRM_ATTEMPTS = 100;

/**
 Milliseconds between two checks of the marker.
 */
const CONFIRM_DELAY_MS = 20;

/**
 How the sentinel was started.
 */
export type SentinelPlacement = 'service' | 'beside the watcher';

/**
 The steps of starting a sentinel,
 replaceable in tests.
 */
export type SentinelLaunch = {
  /**
   Starts the sentinel as its own transient service of the system manager.
   */
  readonly startAsService: (call: { readonly args: readonly string[]; },) => Promise<void>;

  /**
   Starts the sentinel as a detached process of the caller.
   */
  readonly startDetached: (call: { readonly args: readonly string[]; },) => Promise<void>;

  /**
   Reads the marker the running sentinel leaves.
   */
  readonly readMarker: typeof readSentinelMarker;

  /**
   Removes a marker an earlier sentinel left.
   */
  readonly removeMarker: (call: { readonly statePath: string; },) => Promise<void>;

  /**
   Tells whether the process with this pid and start time runs.
   */
  readonly isAlive: (call: {
    readonly pid: number;
    readonly startTime: string
  },) => Promise<boolean>;

  /**
   Waits between two checks.
   */
  readonly pause: (milliseconds: number,) => Promise<unknown>;

  /**
   Says a loud report.
   */
  readonly report: (message: string,) => void;
};

/**
 The sudo identity in the environment of this process.

 @returns The values that name the user who started the tunnel through `sudo`,
 for those that are set.

 @example
 ```ts
 currentSudoEnvironment();
 ```
 */
function currentSudoEnvironment(): SudoEnvironment {
  /**
   The values in the environment of this process.
   */
  const {
    SUDO_UID: sudoUid,
    SUDO_GID: sudoGid,
  } = process.env;
  return {
    ...(sudoUid === undefined ? {} : { SUDO_UID: sudoUid, }),
    ...(sudoGid === undefined ? {} : { SUDO_GID: sudoGid, }),
  };
}

/**
 Name of the transient service that holds the sentinel of one watcher.

 @param identity - Registered watcher identity.

 @returns A unit name unique to this watcher process.

 @example
 ```ts
 sentinelUnitName({ identity: { ownerId: 'o', pid: 4242, startTime: '9001' } });
 ```
 */
export function sentinelUnitName({ identity, }: { readonly identity: WatcherProcessIdentity; },): string {
  return `wg-quicker-sentinel-${String(identity.pid,)}-${identity.startTime}`;
}

/**
 Arguments of the sentinel script.

 @param statePath - Persisted bypass state path.

 @param identity - Registered watcher identity.

 @returns Script path followed by its arguments.

 @example
 ```ts
 sentinelScriptArguments({ statePath, identity });
 ```
 */
export function sentinelScriptArguments(
  {
    statePath,
    identity,
  }: {
    readonly statePath: string;
    readonly identity: WatcherProcessIdentity;
  },
): readonly string[] {
  return [
    SENTINEL_PATH,
    statePath,
    identity.ownerId,
    String(identity.pid,),
    identity.startTime,
  ];
}

/**
 Arguments of `systemd-run` that start the sentinel as a service of the system manager,
 outside the cgroup of the watcher.
 A kill of the whole cgroup of the watcher,
 for example by `systemd-oomd`,
 then leaves the sentinel able to report,
 and `ManagedOOMPreference=omit` asks `systemd-oomd` not to pick the service itself.
 The sudo identity travels in the environment because the service does not inherit it.

 @param identity - Registered watcher identity.

 @param scriptArguments - Script path and its arguments.

 @param environment - Environment of the caller that holds the sudo identity.

 @returns Arguments after `systemd-run`.

 @example
 ```ts
 systemdRunArguments({ identity, scriptArguments, environment: { SUDO_UID: '1000' } });
 ```
 */
export function systemdRunArguments(
  {
    identity,
    scriptArguments,
    environment,
  }: {
    readonly identity: WatcherProcessIdentity;
    readonly scriptArguments: readonly string[];
    readonly environment: SudoEnvironment;
  },
): readonly string[] {
  /**
   Sudo identity variables the sentinel needs,
   as `--setenv` arguments.
   */
  const setenv: string[] = [];
  if (environment.SUDO_UID !== undefined)
    setenv.push(`--setenv=SUDO_UID=${environment.SUDO_UID}`,);
  if (environment.SUDO_GID !== undefined)
    setenv.push(`--setenv=SUDO_GID=${environment.SUDO_GID}`,);
  return [
    '--quiet',
    '--collect',
    '--no-ask-password',
    `--unit=${sentinelUnitName({ identity, },)}`,
    '--description=wg-quicker route watcher sentinel',
    '--property=ManagedOOMPreference=omit',
    ...setenv,
    '--',
    process.execPath,
    ...scriptArguments,
  ];
}

/**
 Waits until the sentinel has left its marker and its process runs.

 @param statePath - Persisted bypass state path.

 @param watcherPid - Process identifier of the watcher the sentinel waits for.

 @param launch - Steps,
 replaceable in tests.

 @returns Whether the sentinel confirmed it runs within the bounded wait.

 @example
 ```ts
 await confirmSentinelRunning({ statePath, watcherPid: 4242, launch });
 ```
 */
export async function confirmSentinelRunning(
  {
    statePath,
    watcherPid,
    launch,
  }: {
    readonly statePath: string;
    readonly watcherPid: number;
    readonly launch: SentinelLaunch;
  },
): Promise<boolean> {
  /**
   Bounded wait cursor.
   */
  const cursor = { attempt: 0, };
  while (cursor.attempt < CONFIRM_ATTEMPTS) {
    /**
     Marker the sentinel left.
     */
    // oxlint-disable-next-line eslint/no-await-in-loop -- The marker is the sentinel's asynchronous readiness signal.
    const marker = await launch.readMarker({ statePath, },);
    if (marker.found && (marker.marker
      .watcherPid
      === watcherPid)
      // oxlint-disable-next-line eslint/no-await-in-loop -- Liveness is checked only once a marker names this watcher.
      && (await launch.isAlive({
        pid: marker.marker
          .pid,
        startTime: marker.marker
          .startTime,
      },)))
      return true;
    // oxlint-disable-next-line eslint/no-await-in-loop -- Bounded wait avoids a busy spin.
    await launch.pause(CONFIRM_DELAY_MS,);
    cursor.attempt += 1;
  }
  return false;
}

/**
 Starts the sentinel detached beside the caller.

 @param args - Script path and its arguments.

 @throws When the process cannot start.

 @example
 ```ts
 await startDetachedSentinel({ args: [SENTINEL_PATH] });
 ```
 */
async function startDetachedSentinel({ args, }: { readonly args: readonly string[]; },): Promise<void> {
  /**
   Detached sentinel inheriting the environment that names the sudo caller.
   */
  const sentinel = spawnChild(
    process.execPath,
    [...args,],
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
 Starts the sentinel as a transient service of the system manager.

 @param args - Arguments after `systemd-run`.

 @throws When `systemd-run` is missing or the manager refuses the unit.

 @example
 ```ts
 await startServiceSentinel({ args: ['--unit=x', '--', '/usr/bin/true'] });
 ```
 */
async function startServiceSentinel({ args, }: { readonly args: readonly string[]; },): Promise<void> {
  await run({
    command: 'systemd-run',
    args,
  },);
}

/**
 The real steps of starting a sentinel.
 */
const PRODUCTION_LAUNCH: SentinelLaunch = {
  startAsService: startServiceSentinel,
  startDetached: startDetachedSentinel,
  readMarker: readSentinelMarker,
  removeMarker: async function removeMarker({ statePath, }: { readonly statePath: string; },): Promise<void> {
    await rm(
      sentinelMarkerPath({ statePath, },),
      { force: true, },
    );
  },
  isAlive: async function isAlive({
    pid,
    startTime,
  }: {
    readonly pid: number;
    readonly startTime: string
  },): Promise<boolean> {
    /**
     Procfs identity of the pid now.
     */
    const live = await readLinuxProcessIdentity({ pid, },);
    return (live !== PROCESS_ABSENT) && (live.startTime === startTime);
  },
  pause: wait,
  report: function report(message: string,): void {
    l.error(message,);
  },
};

/**
 Starts the sentinel outside the cgroup of the watcher,
 confirms that it runs,
 and says loudly when it cannot or did not,
 without failing the tunnel start,
 because the watcher itself is running.
 When the manager refuses the service the sentinel runs beside the watcher instead,
 and the report says that a kill of the watcher's whole cgroup then ends both.

 @param statePath - Persisted bypass state path.

 @param identity - Registered watcher identity.

 @param environment - Environment of the caller that holds the sudo identity.

 @param launch - Steps,
 replaceable in tests.

 @returns Where the sentinel was started and whether it confirmed it runs.

 @example
 ```ts
 await startSentinelOrReport({ statePath, identity });
 ```
 */
export async function startSentinelOrReport(
  {
    statePath,
    identity,
    environment = currentSudoEnvironment(),
    launch = PRODUCTION_LAUNCH,
  }: {
    readonly statePath: string;
    readonly identity: WatcherProcessIdentity;
    readonly environment?: SudoEnvironment;
    readonly launch?: SentinelLaunch;
  },
): Promise<{
  readonly placement?: SentinelPlacement;
  readonly confirmed: boolean
}> {
  await launch.removeMarker({ statePath, },);
  /**
   Script path and arguments.
   */
  const scriptArguments = sentinelScriptArguments({
    statePath,
    identity,
  },);
  /**
   Where the sentinel ended up.
   */
  const placed: { placement?: SentinelPlacement; } = {};
  try {
    await launch.startAsService({
      args: systemdRunArguments({
        identity,
        scriptArguments,
        environment,
      },),
    },);
    placed.placement = 'service';
  }
  catch (serviceError) {
    launch.report(policyNotInForce({
      policy: POLICY_BYPASS_TABLE,
      detail: `the sentinel that reports the route watcher's own death could not start as its own service (${String(serviceError,)}), `
        + 'so it starts beside the watcher: a kill of the watcher\'s whole cgroup, for example by systemd-oomd, then ends both without a report',
    },),);
    try {
      await launch.startDetached({ args: scriptArguments, },);
      placed.placement = 'beside the watcher';
    }
    catch (detachedError) {
      launch.report(policyNotInForce({
        policy: POLICY_BYPASS_TABLE,
        detail: `the sentinel that reports the route watcher's own death could not start at all (${String(detachedError,)}), `
          + 'so if the watcher is killed or runs out of memory nothing will say so',
      },),);
      return { confirmed: false, };
    }
  }
  /**
   Whether the sentinel left its marker and runs.
   */
  const confirmed = await confirmSentinelRunning({
    statePath,
    watcherPid: identity.pid,
    launch,
  },);
  if (!confirmed)
    launch.report(policyNotInForce({
      policy: POLICY_BYPASS_TABLE,
      detail: `the sentinel that reports the route watcher's own death was started (${placed.placement ?? 'nowhere'}) `
        + `but did not confirm that it runs within ${String(CONFIRM_ATTEMPTS * CONFIRM_DELAY_MS,)} ms, `
        + `so if the watcher is killed or runs out of memory nothing may say so; look at \`systemctl status ${sentinelUnitName({ identity, },)}\``,
    },),);
  return {
    ...(placed.placement === undefined ? {} : { placement: placed.placement, }),
    confirmed,
  };
}
