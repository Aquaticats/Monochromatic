/**
 Child-process helpers for the container suite:
 captured runs,
 process-group handles for fault injection,
 and file-marker waits.

 @module
 */

import { spawn, } from 'node:child_process';
import { once, } from 'node:events';
import {
  access,
  readFile,
} from 'node:fs/promises';
import {
  buffer as consumeBuffer,
  text as consumeText,
} from 'node:stream/consumers';
import { setTimeout as sleep, } from 'node:timers/promises';

//region Types

/**
 Settled child process.
 */
export type ProcessOutcome = Readonly<{
  /**
   Exit code,
   or `-1` when a signal ended the process.
   */
  exitCode: number;
  /**
   Terminating signal,
   when one ended the process.
   */
  signal?: string;
  /**
   Captured standard output.
   */
  stdout: string;
  /**
   Captured standard error.
   */
  stderr: string;
  /**
   Milliseconds from start to settlement.
   */
  durationMs: number;
}>;

/**
 Running child process in its own process group.
 */
export type RunningProcess = Readonly<{
  /**
   Group leader PID.
   */
  pid: number;
  /**
   Settles when the leader exits and its streams close.
   */
  outcome: Promise<ProcessOutcome>;
  /**
   Reports whether the process has settled.
   */
  isSettled: () => boolean;
  /**
   Sends `SIGKILL` to the whole group:
   wrapper,
   real Git,
   hooks,
   and editors.
   */
  killGroup: () => void;
}>;

/**
 Command to start.
 */
export type CommandSpec = Readonly<{
  /**
   Executable path or PATH name.
   */
  command: string;
  /**
   Argument vector.
   */
  args: readonly string[];
  /**
   Working directory.
   */
  cwd: string;
  /**
   Complete environment.
   */
  env: NodeJS.ProcessEnv;
  /**
   Standard input text.
   */
  input?: string;
}>;

//endregion Types

//region Errors

/**
 Setup command failed.
 */
export class FixtureCommandError extends Error {
  /**
   Creates an error with command output.

   @param message - command and diagnostics

   @example
   ```ts
   throw new FixtureCommandError('git init exited 1');
   ```
   */
  constructor(message: string,) {
    super(message,);
    this.name = 'FixtureCommandError';
  }
}

//endregion Errors

//region Running

/**
 Group kill functions of processes that have not settled,
 so a scenario timeout can stop everything it started.
 */
const activeGroups = new Set<() => void>();

/**
 Kills every unsettled process group started through {@link startProcess}.

 @returns number of groups signalled

 @example
 ```ts
 killActiveGroups();
 ```
 */
export function killActiveGroups(): number {
  /**
   Groups signalled now.
   */
  const groups = [...activeGroups,];
  groups.forEach(function killGroup(kill,) {
    kill();
  },);
  return groups.length;
}

/**
 Starts a command as a new process-group leader with captured streams.

 @param spec - command to start

 @returns running handle

 @throws {@link FixtureCommandError} when the child has no PID or streams

 @example
 ```ts
 const running = startProcess({ command: 'git', args: ['status'], cwd: '/work/repo', env });
 await running.outcome;
 ```
 */
export function startProcess(spec: CommandSpec,): RunningProcess {
  /**
   Start time for duration measurement.
   */
  const startedAt = performance.now();
  /**
   Detached child so its descendants share a killable process group.
   */
  const child = spawn(
    spec.command,
    [...spec.args,],
    {
      cwd: spec.cwd,
      env: spec.env,
      detached: true,
      stdio: [
        spec.input === undefined ? 'ignore' : 'pipe',
        'pipe',
        'pipe',
      ],
    },
  );
  // A command that cannot start emits `error` on the next tick; without a listener Node ends the whole process,
  // so the throw below could never reach a caller. The failed start is reported by that throw.
  child.once(
    'error',
    function ignoreStartError(): void {
      // The `close` event still settles the outcome of a child that did start.
    },
  );
  if ((child.pid === undefined) || (child.stdout === null)
    || (child.stderr === null))
    throw new FixtureCommandError(`could not start ${spec.command}`,);
  child.stdin
    ?.end(spec.input,);
  /**
   Group leader PID.
   */
  const { pid, } = child;
  /**
   Stream collection started before settlement.
   */
  const streams = Promise.all([
    consumeText(child.stdout,),
    consumeText(child.stderr,),
  ],);
  /**
   Sends `SIGKILL` to the child's process group.

   @example
   ```ts
   killGroup();
   ```
   */
  function killGroup(): void {
    try {
      process.kill(
        -pid,
        'SIGKILL',
      );
    }
    catch (error: unknown) {
      // ESRCH: the group already exited, which is the state killing aims for.
      if (!(Error.isError(error,) && ('code' in error)
        && (error.code === 'ESRCH')))
        throw error;
    }
  }
  /**
   Settlement flag read by marker waits.
   */
  const state = { settled: false, };
  /**
   Settlement with captured output.
   */
  const outcome = (async function settle(): Promise<ProcessOutcome> {
    await once(
      child,
      'close',
    );
    state.settled = true;
    activeGroups.delete(killGroup,);
    /**
     Collected standard streams.
     */
    const [stdout, stderr,] = await streams;
    return {
      exitCode: child.exitCode ?? (-1),
      ...(child.signalCode === null ? {} : { signal: child.signalCode, }),
      stdout,
      stderr,
      durationMs: Math.round(performance.now() - startedAt,),
    };
  })();
  activeGroups.add(killGroup,);
  return {
    pid,
    outcome,
    isSettled(): boolean {
      return state.settled;
    },
    killGroup,
  };
}

/**
 Runs a command to settlement.

 @param spec - command to run

 @returns settled outcome

 @example
 ```ts
 await runProcess({ command: '/opt/git/2.55.0/bin/git', args: ['--version'], cwd: '/', env });
 ```
 */
export async function runProcess(spec: CommandSpec,): Promise<ProcessOutcome> {
  return await startProcess(spec,)
    .outcome;
}

/**
 Runs a setup command that must succeed.

 @param spec - command to run

 @returns standard output

 @throws {@link FixtureCommandError} on non-zero exit

 @example
 ```ts
 await runChecked({ command: realGit, args: ['init'], cwd, env });
 ```
 */
export async function runChecked(spec: CommandSpec,): Promise<string> {
  /**
   Settled outcome.
   */
  const outcome = await runProcess(spec,);
  if (outcome.exitCode !== 0) {
    throw new FixtureCommandError(
      `${spec.command} ${spec.args
        .join(' ',)} exited ${String(outcome.exitCode,)}\n${outcome.stderr}${outcome.stdout}`,
    );
  }
  return outcome.stdout;
}

/**
 Runs a command that must succeed and returns its exact standard output bytes.

 @param spec - command to run

 @returns standard output bytes

 @throws {@link FixtureCommandError} on non-zero exit

 @example
 ```ts
 await runBytes({ command: realGit, args: ['cat-file', 'blob', oid], cwd, env });
 ```
 */
export async function runBytes(spec: CommandSpec,): Promise<Buffer> {
  /**
   Child with binary-safe output capture.
   */
  const child = spawn(
    spec.command,
    [...spec.args,],
    {
      cwd: spec.cwd,
      env: spec.env,
      stdio: [
        'ignore',
        'pipe',
        'pipe',
      ],
    },
  );
  /**
   Binary-safe collection started before settlement.
   */
  const streams = Promise.all([
    consumeBuffer(child.stdout,),
    consumeText(child.stderr,),
  ],);
  await once(
    child,
    'close',
  );
  /**
   Collected output and diagnostic.
   */
  const [stdout, stderr,] = await streams;
  if (child.exitCode !== 0)
    throw new FixtureCommandError(`${spec.command} ${spec.args
      .join(' ',)} exited ${String(child.exitCode,)}\n${stderr}`,);
  return stdout;
}

//endregion Running

//region Markers

/**
 Poll interval for marker files.
 */
const MARKER_POLL_MS = 10;

/**
 Reports whether a path exists.

 @param path - absolute path

 @returns existence

 @example
 ```ts
 await pathExists('/work/markers/a.pre-commit');
 ```
 */
export async function pathExists(path: string,): Promise<boolean> {
  try {
    await access(path,);
    return true;
  }
  catch (error: unknown) {
    if (Error.isError(error,) && ('code' in error)
      && (error.code === 'ENOENT'))
      return false;
    throw error;
  }
}

/**
 Waits until a marker exists,
 the watched process settles,
 or the timeout passes.

 @param path - marker path

 @param timeoutMs - upper bound

 @param isSettled - reports settlement of the process whose hook writes the marker

 @returns `marker`, `settled`, or `timeout`

 @example
 ```ts
 await waitForMarker({ path, timeoutMs: 30_000, isSettled: running.isSettled });
 ```
 */
export async function waitForMarker({
  path,
  timeoutMs,
  isSettled,
}: Readonly<{
  path: string;
  timeoutMs: number;
  isSettled?: () => boolean;
}>,): Promise<'marker' | 'settled' | 'timeout'> {
  /**
   Deadline in monotonic milliseconds.
   */
  const deadline = performance.now() + timeoutMs;
  for (;;) {
    // oxlint-disable-next-line no-await-in-loop -- Marker polling is serial by definition.
    if (await pathExists(path,))
      return 'marker';
    if (isSettled?.() === true)
      return 'settled';
    if (performance.now() >= deadline)
      return 'timeout';
    // oxlint-disable-next-line no-await-in-loop -- Bounded poll interval between marker checks.
    await sleep(MARKER_POLL_MS,);
  }
}

/**
 Reads a text file that may not exist yet.

 @param path - absolute path

 @returns file text, empty when missing

 @example
 ```ts
 await readOptionalText('/work/s1/logs/hooks.jsonl');
 ```
 */
export async function readOptionalText(path: string,): Promise<string> {
  try {
    return await readFile(
      path,
      'utf8',
    );
  }
  catch (error: unknown) {
    if (Error.isError(error,) && ('code' in error)
      && (error.code === 'ENOENT'))
      return '';
    throw error;
  }
}

//endregion Markers
