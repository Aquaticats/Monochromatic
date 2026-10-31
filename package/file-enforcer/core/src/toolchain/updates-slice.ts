/**
 Runs the toolchain's sync and update entry points inside a dedicated user slice.

 `wg-quicker-exempt` (`package/wg/cli-quicker-exempt`) attaches its socket-marking cgroup programs to this slice
 while a WireGuard tunnel with `ExemptMark` is up, so every socket a run and its children open
 (cargo-binstall, cargo, rustup, uv, pnpm, node fetch, curl) takes the physical route instead of the tunnel.
 Cgroup programs apply to descendant cgroups, so the per-run scope that `systemd-run` creates below the slice
 inherits the mark with no attach of its own.
 Without a tunnel the slice is inert, and without systemd the run happens in place.

 Node builtins only, so `sync.ts` and `update.ts` keep running without the workspace's `node_modules`.
 */

import {
  type ChildProcess,
  spawn,
} from 'node:child_process';
import { once, } from 'node:events';
import { constants as osConstants, } from 'node:os';
import {
  access,
  readFile,
} from 'node:fs/promises';
import { join, } from 'node:path';
import { l, } from '../logger.ts';
import type { PathEnv, } from './paths.ts';

//region Contract

/**
 Slice name `wg-quicker-exempt` matches exactly
 (`TOOLCHAIN_UPDATES_SLICE` in `package/wg/cli-quicker-exempt/src/application_targets.rs`).

 The `app-` prefix nests it directly under the user's `app.slice`, where the watcher scans and watches;
 `\x2d` is the escaped literal dash, so the name is one hierarchy level and not `app-monochromatic.slice`.
 */
export const TOOLCHAIN_UPDATES_SLICE: string = String.raw`app-monochromatic\x2dupdates.slice`;

/**
 Environment variable the re-executed child carries, so it never re-executes again.
 */
export const UPDATES_SLICE_GUARD_ENVIRONMENT = 'MONOCHROMATIC_TOOLCHAIN_IN_UPDATES_SLICE';

/**
 Environment variable whose value `off` keeps a run in place, for a host where the slice must not be used.
 */
export const UPDATES_SLICE_OPT_OUT_ENVIRONMENT = 'MONOCHROMATIC_TOOLCHAIN_CLEARNET';

/**
 Sentinel returned when the run was not moved into the slice and the caller must do the work itself.
 */
export const NOT_RELAUNCHED: unique symbol = Symbol('toolchain run stayed in place',);

//endregion Contract

//region Decision

/**
 Facts the relaunch decision reads.
 */
export type UpdatesSliceFacts = {
  /**
   Operating system platform of this process.
   */
  readonly platform: string;

  /**
   Environment of this process.
   */
  readonly env: PathEnv;

  /**
   Contents of `/proc/self/cgroup`, empty when unreadable.
   */
  readonly cgroupText: string;

  /**
   Whether the user's systemd manager socket exists.
   */
  readonly userManagerPresent: boolean;
};

/**
 Why a run stays in place, or that it moves into the slice.
 */
export type UpdatesSliceDecision =
  | { readonly relaunch: true; }
    | {
      readonly relaunch: false;
      readonly reason: string
    };

/**
 Reports whether a process already runs inside the updates slice.

 @param cgroupText - Contents of `/proc/self/cgroup`.

 @returns Whether the unified cgroup path contains the slice as one path segment.

 @example
 ```ts
 insideUpdatesSlice({ cgroupText: '0::/user.slice/app.slice/app-monochromatic\\x2dupdates.slice/run-1.scope\n' }); // true
 ```
 */
export function insideUpdatesSlice({ cgroupText, }: { readonly cgroupText: string; },): boolean {
  return cgroupText
    .split('\n',)
    .some(function hasSliceSegment(line,): boolean {
      return line.startsWith('0::',)
        && line
          .slice('0::'.length,)
          .split('/',)
          .includes(TOOLCHAIN_UPDATES_SLICE,);
    },);
}

/**
 Decides whether this run should re-execute itself inside the updates slice.

 @param platform - Operating system platform of this process.

 @param env - Environment of this process.

 @param cgroupText - Contents of `/proc/self/cgroup`.

 @param userManagerPresent - Whether the user's systemd manager socket exists.

 @returns Relaunch,
 or the reason to stay in place.

 @example
 ```ts
 decideUpdatesSlice({ platform: 'linux', env: {}, cgroupText: '', userManagerPresent: true });
 ```
 */
export function decideUpdatesSlice({
  platform,
  env,
  cgroupText,
  userManagerPresent,
}: UpdatesSliceFacts,): UpdatesSliceDecision {
  if (platform !== 'linux')
    return {
      relaunch: false,
      reason: `platform ${platform} has no user slices`,
    };
  if (env[UPDATES_SLICE_GUARD_ENVIRONMENT] !== undefined)
    return {
      relaunch: false,
      reason: 'this run was already re-executed into the slice',
    };
  if (env[UPDATES_SLICE_OPT_OUT_ENVIRONMENT] === 'off')
    return {
      relaunch: false,
      reason: `${UPDATES_SLICE_OPT_OUT_ENVIRONMENT}=off`,
    };
  if (insideUpdatesSlice({ cgroupText, },))
    return {
      relaunch: false,
      reason: 'already inside the updates slice',
    };
  if (!userManagerPresent)
    return {
      relaunch: false,
      reason: 'no user systemd manager socket (container or non-systemd host)',
    };
  return { relaunch: true, };
}

//endregion Decision

//region Commands

/**
 Builds the `systemd-run` arguments that place a command in a fresh scope below the updates slice.

 @param command - Executable and arguments run inside the scope.

 @returns Arguments for `systemd-run`.

 @example
 ```ts
 systemdRunArguments({ command: ['node', 'sync.ts',] });
 ```
 */
export function systemdRunArguments({ command, }: { readonly command: readonly string[]; },): readonly string[] {
  return [
    '--user',
    '--scope',
    '--quiet',
    '--collect',
    `--slice=${TOOLCHAIN_UPDATES_SLICE}`,
    '--',
    ...command,
  ];
}

/**
 Result of running one child process.
 */
export type ChildOutcome = {
  /**
   Exit code,
   or 128 plus the signal number when a signal ended it.
   */
  readonly code: number;
};

/**
 Shell convention for a command that could not be run.
 */
const COMMAND_NOT_RUNNABLE_EXIT_CODE = 127;

/**
 Offset a shell adds to a signal number to report death by signal.
 */
const SIGNAL_EXIT_CODE_OFFSET = 128;

/**
 Looks up a signal's number by name.

 @param name - Signal name such as `SIGTERM`.

 @returns The signal number,
 or 0 for a name this platform does not define.

 @example
 ```ts
 signalNumber({ name: 'SIGTERM' }); // 15
 ```
 */
function signalNumber({ name, }: { readonly name: string; },): number {
  /**
   Table value for the name,
   which is not a number for an undefined signal.
   */
  const number: unknown = Reflect.get(
    osConstants.signals,
    name,
  );
  return (typeof number) === 'number' ? number as number : 0;
}

/**
 Converts a child's close event into one exit code.

 @param code - Exit code,
 or `null` when a signal ended the child.

 @param signal - Terminating signal name,
 or `null` after a normal exit.

 @returns The exit code,
 128 plus the signal number after a signal,
 or 1 when neither is known.

 @example
 ```ts
 exitCodeOf({ code: null, signal: 'SIGTERM' }); // 143
 ```
 */
function exitCodeOf({
  code,
  signal,
}: {
  readonly code: unknown;
  readonly signal: unknown
},): number {
  if ((typeof code) === 'number')
    return code as number;
  if ((typeof signal) === 'string')
    return SIGNAL_EXIT_CODE_OFFSET + signalNumber({ name: signal, });
  return 1;
}

/**
 Forwards termination signals to a child for as long as the returned handle lives.

 @param child - Running child.

 @returns Handle that removes the listeners on disposal.

 @example
 ```ts
 using forwarding = forwardTerminationSignals({ child });
 ```
 */
function forwardTerminationSignals({ child, }: { readonly child: ChildProcess; },): Disposable {
  /**
   Passes the received signal on so work under the scope stops with the wrapper.

   @param signal - Signal this process received.
   */
  function forward(signal: NodeJS.Signals,): void {
    child.kill(signal,);
  }
  process.on(
    'SIGTERM',
    forward,
  );
  process.on(
    'SIGHUP',
    forward,
  );
  return {
    [Symbol.dispose]: function removeListeners(): void {
      process.removeListener(
        'SIGTERM',
        forward,
      );
      process.removeListener(
        'SIGHUP',
        forward,
      );
    },
  };
}

/**
 Runs a child process to completion.

 @param command - Executable.

 @param args - Arguments.

 @param env - Child environment.

 @param quiet - Whether to discard the child's standard streams instead of inheriting them.

 @returns Exit outcome,
 or a failure outcome when the process could not start.

 @example
 ```ts
 await runChild({ command: 'true', args: [], env: process.env, quiet: true });
 ```
 */
export async function runChild(
  {
    command,
    args,
    env,
    quiet,
  }: {
    readonly command: string;
    readonly args: readonly string[];
    readonly env: PathEnv;
    readonly quiet: boolean;
  },
): Promise<ChildOutcome> {
  /**
   Started child.
   */
  const child = spawn(
    command,
    [...args,],
    {
      env: { ...env, },
      stdio: quiet ? 'ignore' : 'inherit',
    },
  );
  /**
   Signal forwarding that ends when this function returns.
   */
  using forwarding = forwardTerminationSignals({ child, },);
  try {
    // `once` rejects when the child emits `error`, which is how a missing executable shows up.
    /**
     Close event payload: exit code and terminating signal.
     */
    const closing: readonly unknown[] = await once(
      child,
      'close',
    );
    return { code: exitCodeOf({
      code: closing[0],
      signal: closing[1],
    },), };
  }
  catch (error) {
    l.warn(`could not run ${command}: ${String(error,)}`,);
    return { code: COMMAND_NOT_RUNNABLE_EXIT_CODE, };
  }
}

//endregion Commands

//region Relaunch

/**
 Runs one `systemd-run` invocation.
 */
export type SystemdRunner = (
  call: {
    readonly args: readonly string[];
    readonly env: PathEnv;
    readonly quiet: boolean;
  },
) => Promise<ChildOutcome>;

/**
 Re-executes the current entry point inside the updates slice when that is possible.

 A silent probe through `systemd-run` first proves a user manager accepts the scope and creates the slice;
 a failed probe leaves the run in place, because `systemd-run` reports its own failures with the same exit code a
 failing sync uses, so the real run could not be told apart from them.

 @param argv - `process.argv` of the current run.

 @param execPath - Node executable.

 @param execArgv - Node flags of the current run.

 @param facts - Platform,
 environment,
 cgroup text,
 and user-manager presence.

 @param runSystemd - Runs `systemd-run`,
 replaceable in tests.

 @returns The child's exit code when the run happened in the slice,
 or {@link NOT_RELAUNCHED}.

 @example
 ```ts
 const outcome = await relaunchInUpdatesSlice({ argv: process.argv, execPath: process.execPath, execArgv: process.execArgv, facts, runSystemd });
 ```
 */
export async function relaunchInUpdatesSlice(
  {
    argv,
    execPath,
    execArgv,
    facts,
    runSystemd,
  }: {
    readonly argv: readonly string[];
    readonly execPath: string;
    readonly execArgv: readonly string[];
    readonly facts: UpdatesSliceFacts;
    readonly runSystemd: SystemdRunner;
  },
): Promise<number | typeof NOT_RELAUNCHED> {
  /**
   Outcome of the placement decision.
   */
  const decision = decideUpdatesSlice(facts,);
  if (!decision.relaunch) {
    l.debug(`toolchain run stays in place: ${decision.reason}`,);
    return NOT_RELAUNCHED;
  }
  /**
   Probe proving a user manager accepts a scope in the slice.
   */
  const probe = await runSystemd({
    args: systemdRunArguments({ command: [
      execPath,
      '--version',
    ], },),
    env: facts.env,
    quiet: true,
  },);
  if (probe.code !== 0) {
    l.warn(
      `systemd-run could not place a scope in ${TOOLCHAIN_UPDATES_SLICE} (exit ${String(probe.code,)}); `
        + 'update traffic will follow the default route',
    );
    return NOT_RELAUNCHED;
  }
  /**
   Entry script and its arguments as this process received them.
   */
  const [, ...scriptAndArguments] = argv;
  /**
   Run inside the slice with the guard set.
   */
  const outcome = await runSystemd({
    args: systemdRunArguments({ command: [
      execPath,
      ...execArgv,
      ...scriptAndArguments,
    ], },),
    env: {
      ...facts.env,
      [UPDATES_SLICE_GUARD_ENVIRONMENT]: '1',
    },
    quiet: false,
  },);
  return outcome.code;
}

/**
 Reads this process's cgroup membership.

 @returns Contents of `/proc/self/cgroup`,
 or an empty string where procfs is unavailable.

 @example
 ```ts
 await readCgroupText();
 ```
 */
async function readCgroupText(): Promise<string> {
  try {
    return await readFile(
      '/proc/self/cgroup',
      'utf8',
    );
  }
  catch (error) {
    l.debug(`cannot read /proc/self/cgroup: ${String(error,)}`,);
    return '';
  }
}

/**
 Reports whether a path exists.

 @param path - Absolute path.

 @returns Whether the path can be accessed.

 @example
 ```ts
 await pathExists({ path: '/run/user/1000/systemd/private' });
 ```
 */
async function pathExists({ path, }: { readonly path: string; },): Promise<boolean> {
  try {
    await access(path,);
    return true;
  }
  catch (error) {
    l.debug(`${path} is not accessible: ${String(error,)}`,);
    return false;
  }
}

/**
 Runs the real `systemd-run`.

 @param args - Arguments for `systemd-run`.

 @param env - Environment for the child.

 @param quiet - Whether to discard the child's standard streams.

 @returns Exit outcome.

 @example
 ```ts
 await runRealSystemd({ args: ['--version'], env: process.env, quiet: true });
 ```
 */
function runRealSystemd(
  {
    args,
    env,
    quiet,
  }: {
    readonly args: readonly string[];
    readonly env: PathEnv;
    readonly quiet: boolean;
  },
): Promise<ChildOutcome> {
  return runChild({
    command: 'systemd-run',
    args,
    env,
    quiet,
  },);
}

/**
 Re-executes the current process inside the updates slice using the real host.

 @returns The child's exit code when the run happened in the slice,
 or {@link NOT_RELAUNCHED}.

 @example
 ```ts
 const outcome = await runInUpdatesSlice();
 if (outcome !== NOT_RELAUNCHED) process.exitCode = outcome;
 ```
 */
export async function runInUpdatesSlice(): Promise<number | typeof NOT_RELAUNCHED> {
  /**
   User manager socket directory; absent in containers.
   */
  const runtimeDirectory = process.env
    .XDG_RUNTIME_DIR;
  /**
   Cgroup text of this process, empty when unreadable.
   */
  const cgroupText = await readCgroupText();
  /**
   Whether the user's systemd manager listens for this session.
   */
  const userManagerPresent = (runtimeDirectory !== undefined)
    && (runtimeDirectory !== '')
    && await pathExists({
      path: join(
        runtimeDirectory,
        'systemd',
        'private',
      ),
    },);
  return relaunchInUpdatesSlice({
    argv: process.argv,
    execPath: process.execPath,
    execArgv: process.execArgv,
    facts: {
      platform: process.platform,
      env: process.env,
      cgroupText,
      userManagerPresent,
    },
    runSystemd: runRealSystemd,
  },);
}

//endregion Relaunch
