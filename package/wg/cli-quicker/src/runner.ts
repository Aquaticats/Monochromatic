import {
  spawn as spawnChild,
} from 'node:child_process';
import { text, } from 'node:stream/consumers';

import { tagged, } from '@monochromatic-dev/logger-core/ts';

import {
  CommandError,
  CommandSpawnError,
  CommandTimeoutError,
} from './errors.ts';

/**
 Module logger for external command execution.
 */
const l = tagged({ tag: 'runner', },);

/**
 Milliseconds after which a command this program runs itself is ended.

 Nothing here may wait forever,
 and a wedged kernel interface or daemon would otherwise hold a lifecycle command open.
 Sixty seconds is a stall detector and not a target:
 measured on this host inside a disposable user and network namespace,
 `ip link delete` took 52 ms,
 `ip link add` 8,
 `ip rule add` 6,
 `ip rule show` 7,
 `wg show` 3 to 5,
 `wg addconf` with 20 peers 12,
 `wg addconf` with one peer of 5000 prefixes 5,
 `nft list ruleset` over 200 rules 10,
 `nft -j list tables` 9,
 and `sysctl` 5,
 which puts the bound at more than a thousand times the slowest measured call.
 */
const COMMAND_DEADLINE_MILLISECONDS = 60_000;

/**
 Milliseconds after which a hook of the user's own is ended.

 A hook is the user's command and `wg-quick` waits for it,
 so its bound is far above the one the program's own commands get.
 Ten minutes is where a hook has certainly stopped making progress
 rather than a measured target.
 The error names the hook and the limit,
 so a hook that needs longer can be changed.
 */
export const HOOK_DEADLINE_MILLISECONDS: number = 600_000;

/**
 Sentinel standing for a program that was started,
 the state a spawn failure replaces.
 */
const PROGRAM_STARTED: unique symbol = Symbol('the program was started',);

/**
 Result of a successfully executed external command.
 */
export type CommandResult = {
  /**
   Captured standard output decoded as UTF-8.
   */
  readonly stdout: string;

  /**
   Captured standard error decoded as UTF-8.
   */
  readonly stderr: string;
};

/**
 Renders an argument list for logging without reading a caller-owned array.
 
 @param command - Executable name.
 
 @param args - Arguments passed to the executable.
 
 @returns Single-line rendering of the invocation.
 
 @example
 ```ts
 renderInvocation({ command: 'ip', args: ['link', 'show'] });
 ```
 */
function renderInvocation(
  {
    command,
    args,
  }: {
    readonly command: string;
    readonly args: readonly string[];
  },
): string {
  /**
   Fresh copy so joining never touches a caller-owned array.
   */
  const rendered: readonly string[] = [
    ...args,
  ];
  return `${command} ${rendered.join(' ',)}`;
}

/**
 Runs one external command,
 waiting for it no longer than `deadlineMilliseconds`,
 and ends it when that passes.

 The spawn failure and the close are collected here rather than left to the stream helpers:
 an unobserved `error` event ends the Node process,
 and a caller that catches the rejection is what runs the failed-up rollback.
 The command's own standard in is either the text the caller gives it or an absent one,
 never the terminal's,
 so a command that waits for input cannot hold a lifecycle open that way.

 @param command - Executable name resolved through `PATH`.

 @param args - Arguments passed to the command.

 @param input - Optional text written to the command's standard in.

 @param deadlineMilliseconds - Longest time to wait for the command.

 @returns What the command ended with,
 including a non-zero exit code,
 which each caller judges for itself.

 @throws {@link CommandSpawnError} when the program could not be started.

 @throws {@link CommandTimeoutError} when the command was still running at its deadline.

 @example
 ```ts
 await execute({ command: 'ip', args: ['link', 'show'], deadlineMilliseconds: 60_000 });
 ```
 */
async function execute(
  {
    command,
    args,
    input,
    deadlineMilliseconds,
  }: {
    readonly command: string;
    readonly args: readonly string[];
    readonly input?: string;
    readonly deadlineMilliseconds: number;
  },
): Promise<CommandResult & { readonly exitCode: number; }> {
  /**
   Spawned child with piped output for capture.
   */
  const child = spawnChild(
    command,
    [
      ...args,
    ],
    {
      stdio: [
        input === undefined ? 'ignore' : 'pipe',
        'pipe',
        'pipe',
      ],
    },
  );
  /**
   Both captured streams,
   read while the child runs:
   a closed pipe keeps nothing that was not read,
   and a command that ends at once would lose its output.
   */
  const captured = Promise.allSettled([
    child.stdout === null ? Promise.resolve('',) : text(child.stdout,),
    child.stderr === null ? Promise.resolve('',) : text(child.stderr,),
  ],);
  /**
   What the close and the error events establish about the run.
   */
  const observed: {
    spawnFailure: Readonly<Error> | typeof PROGRAM_STARTED;
    endedByDeadline: boolean;
  } = {
    spawnFailure: PROGRAM_STARTED,
    endedByDeadline: false,
  };
  /**
   Swallows `EPIPE` from a child that closes standard in early (for example a
   rejection), so the real error surfaces via the child's exit code instead of
   an unhandled stream error.
   
   @param error - Stream error from the standard-in pipe.
   */
  function onStdinError(error: Readonly<Error & { readonly code?: string; }>,): void {
    if (error.code !== 'EPIPE')
      l.error(`stdin error for ${command}: ${error.message}`,);
  }
  if (child.stdin !== null) {
    child.stdin
      .on(
      'error',
      onStdinError,
      );
    child.stdin
      .end(input,);
  }
  /**
   Settled by the close of the child,
   which follows a spawn failure as well as an ordinary end.
   */
  const {
    promise: closed,
    resolve: resolveClosed,
  } = Promise.withResolvers<void>();
  child.once(
    'error',
    function noteSpawnFailure(error: Readonly<Error>,): void {
      observed.spawnFailure = error;
    },
  );
  child.once(
    'close',
    function ended(): void {
      clearTimeout(deadline,);
      resolveClosed();
    },
  );
  /**
   Ends a command that outlives its deadline.
   A child that already exited keeps its own exit code:
   the deadline only claims what it ended.
   */
  const deadline = setTimeout(
    function end(): void {
      if ((child.exitCode !== null) || (child.signalCode !== null))
        return;
      observed.endedByDeadline = true;
      // SIGKILL, because a command that ignored the deadline would ignore a polite signal too,
      // and every command run here is idempotent or is rolled back.
      child.kill('SIGKILL',);
    },
    deadlineMilliseconds,
  );
  await closed;
  /**
   Captured stdout and stderr of the closed child.
   */
  const [stdoutResult, stderrResult,] = await captured;
  if (stdoutResult.status === 'rejected')
    l.error(`${command}: its standard output could not be read: ${String(stdoutResult.reason,)}`,);
  if (stderrResult.status === 'rejected')
    l.error(`${command}: its standard error could not be read: ${String(stderrResult.reason,)}`,);
  /**
   What the command printed.
   */
  const printed: CommandResult = {
    stdout: stdoutResult.status === 'fulfilled' ? stdoutResult.value : '',
    stderr: stderrResult.status === 'fulfilled' ? stderrResult.value : '',
  };
  if (observed.spawnFailure !== PROGRAM_STARTED)
    throw new CommandSpawnError({
      command,
      args,
      reason: observed.spawnFailure.message,
    },);
  if (observed.endedByDeadline)
    throw new CommandTimeoutError({
      command,
      args,
      deadlineMilliseconds,
    },);
  return {
    ...printed,
    exitCode: child.exitCode ?? (-1),
  };
}

/**
 Runs one external command, capturing output and throwing on non-zero exit.
 
 Input can be piped to standard in, which is how the raw peer config reaches
 `wg addconf` without an intermediate file.
 
 The call ends at `deadlineMilliseconds`,
 so an `ip`, `wg`, `nft`, `resolvectl` or `sysctl` that stops answering
 fails the step that ran it instead of holding the lifecycle open.
 A hook of the user's own goes through {@link executeHooks},
 which gives it a bound of its own.
 
 @param command - Executable name resolved through `PATH`.
 
 @param args - Arguments passed to the command.
 
 @param input - Optional text written to the command's standard in.
 
 @param deadlineMilliseconds - Longest time to wait for the command.
 
 @returns Captured stdout and stderr.
 
 @throws {@link CommandSpawnError} when the program could not be started.
 
 @throws {@link CommandTimeoutError} when the command was still running at its deadline.
 
 @throws {@link CommandError} when the command exits non-zero or closes without a code.
 
 @example
 ```ts
 await run({ command: 'ip', args: ['link', 'show', 'dev', 'wg0'] });
 ```
 */
export async function run(
  {
    command,
    args,
    input,
    deadlineMilliseconds = COMMAND_DEADLINE_MILLISECONDS,
  }: {
    readonly command: string;
    readonly args: readonly string[];
    readonly input?: string;
    readonly deadlineMilliseconds?: number;
  },
): Promise<CommandResult> {
  /**
   Function-scoped logger for one command invocation.
   */
  const fl = tagged({
    tag: run.name,
    l,
  },);
  fl.debug(`exec ${renderInvocation({
    command,
    args,
  },)}`,);
  /**
   What the command ended with.
   */
  const execution = await execute({
    command,
    args,
    ...(input === undefined ? {} : { input, }),
    deadlineMilliseconds,
  },);
  if (execution.exitCode !== 0) {
    fl.error(`${command} exited ${String(execution.exitCode,)}: ${execution.stderr}`,);
    throw new CommandError({
      command,
      args,
      exitCode: execution.exitCode,
      stderr: execution.stderr,
    },);
  }
  return {
    stdout: execution.stdout,
    stderr: execution.stderr,
  };
}

/**
 Runs a command that is allowed to fail, returning its exit code instead of throwing.
 
 Used for idempotent teardown and for probes where absence is expected.
 
 A program that could not be started is not an absence:
 it never ran,
 so it is refused,
 and so is a command that passed its deadline.
 
 @param command - Executable name resolved through `PATH`.
 
 @param args - Arguments passed to the command.
 
 @param deadlineMilliseconds - Longest time to wait for the command.
 
 @returns Exit code and captured output.
 
 @throws {@link CommandSpawnError} when the program could not be started.
 
 @throws {@link CommandTimeoutError} when the command was still running at its deadline.
 
 @example
 ```ts
 await runAllowingFailure({ command: 'ip', args: ['link', 'delete', 'dev', 'wg0'] });
 ```
 */
export function runAllowingFailure(
  {
    command,
    args,
    deadlineMilliseconds = COMMAND_DEADLINE_MILLISECONDS,
  }: {
    readonly command: string;
    readonly args: readonly string[];
    readonly deadlineMilliseconds?: number;
  },
): Promise<CommandResult & { readonly exitCode: number; }> {
  /**
   Function-scoped logger for one tolerated invocation.
   */
  const fl = tagged({
    tag: runAllowingFailure.name,
    l,
  },);
  fl.debug(`exec (allowed to fail) ${renderInvocation({
    command,
    args,
  },)}`,);
  return execute({
    command,
    args,
    deadlineMilliseconds,
  },);
}
