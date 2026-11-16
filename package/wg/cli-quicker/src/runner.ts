import {
  spawn as spawnChild,
} from 'node:child_process';
import type { Readable, } from 'node:stream';

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
 Collects everything a stream delivers as UTF-8 text,
 and answers what arrived even when the stream is closed early.

 `text()` from `node:stream/consumers` refuses when a stream closes before its end,
 which is what the deadline does to a pipe that a process the command left behind still holds:
 the output the command had already written would go with it.
 Every chunk is kept here,
 and the answer is the text read so far,
 whether the stream ended, failed or was closed.

 @param stream - Standard output or standard error of a child that was piped.

 @returns What the stream delivered, as UTF-8 text.

 @example
 ```ts
 await collectText(child.stdout);
 ```
 */
function collectText(stream: Readable,): Promise<string> {
  /**
   Chunks read so far, in the order they arrived.
   */
  const chunks: Uint8Array[] = [];
  /**
   The text,
   answered by whichever of the stream's ends comes first.
   */
  const {
    promise,
    resolve,
  } = Promise.withResolvers<string>();
  /**
   Answers the text read so far.
   */
  function answer(): void {
    /**
     Bytes read so far, joined.
     */
    const joined = Buffer.concat(chunks,);
    resolve(joined.toString('utf8',),);
  }
  stream.on(
    'data',
    function keep(chunk: Uint8Array,): void {
      chunks.push(chunk,);
    },
  );
  stream.once(
    'end',
    answer,
  );
  stream.once(
    'close',
    answer,
  );
  stream.once(
    'error',
    answer,
  );
  return promise;
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
  const captured = Promise.all([
    // `ChildProcess` types a stream as absent when it was not piped,
    // and this spawn always pipes both.
    child.stdout === null ? Promise.resolve('',) : collectText(child.stdout,),
    child.stderr === null ? Promise.resolve('',) : collectText(child.stderr,),
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
   which follows a spawn failure as well as an ordinary end,
   or by the deadline when the close does not come.
   */
  const {
    promise: closed,
    resolve: resolveClosed,
  } = Promise.withResolvers<void>();
  /**
   Settled when the deadline passed,
   whatever the child and its children were doing.
   */
  const {
    promise: pastDeadline,
    resolve: resolvePastDeadline,
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
   Ends a command that outlives its deadline,
   and stops waiting for its output either way.

   The close of the child does not come while a process the command left behind
   still holds its standard output or standard error,
   so a shell that exits at once after starting a background program
   would hold this call for as long as that program runs.
   The read ends of those pipes are closed here,
   which ends the capture with whatever was read,
   and the wait ends at the deadline as it was asked to.

   A child that already exited keeps its own status:
   the deadline only claims what it ended,
   and a command that succeeded before its bound says so.
   */
  const deadline = setTimeout(
    function end(): void {
      if ((child.exitCode === null) && (child.signalCode === null)) {
        observed.endedByDeadline = true;
        // SIGKILL, because a command that ignored the deadline would ignore a polite signal too,
        // and every command run here is idempotent or is rolled back.
        child.kill('SIGKILL',);
      }
      else {
        l.warn(
          `${command} exited ${String(child.exitCode,)} before its ${String(deadlineMilliseconds,)} ms deadline, and a process it left behind still held its output open, so the call ended at the deadline with the output read so far`,
        );
      }
      /**
       Standard output of the child, when it was piped.
       */
      const output = child.stdout;
      if (output !== null)
        output.destroy();
      /**
       Standard error of the child, when it was piped.
       */
      const errors = child.stderr;
      if (errors !== null)
        errors.destroy();
      resolvePastDeadline();
    },
    deadlineMilliseconds,
  );
  await Promise.race([
    closed,
    pastDeadline,
  ],);
  /**
   Captured stdout and stderr of the child,
   closed by its own end or by the deadline.
   */
  const [stdout, stderr,] = await captured;
  /**
   What the command printed.
   */
  const printed: CommandResult = {
    stdout,
    stderr,
  };
  /**
   Why the program did not start, when it did not.
   */
  const { spawnFailure, } = observed;
  if (spawnFailure !== PROGRAM_STARTED)
    throw new CommandSpawnError({
      command,
      args,
      reason: spawnFailure.message,
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
