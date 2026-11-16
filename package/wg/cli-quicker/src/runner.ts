import {
  spawn as spawnChild,
} from 'node:child_process';
import type { Readable, } from 'node:stream';

import { wait, } from '@monochromatic-dev/async-time-core/ts';
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
 Milliseconds the collector waits, after the command itself exited,
 for the output the command had already written.

 The kernel buffer of a pipe holds at most 64 KiB and the collector reads it
 while the command runs, so what is left at the exit is small:
 measured on this host, 64 KiB written by a command finished arriving 0 to 4 ms
 after the command exited, and this bound is twenty-five times the slowest of ten rounds.
 The wait exists only for a process the command left behind that holds the pipes:
 without it, the call would wait for that process rather than for the command.
 */
const OUTPUT_DRAIN_MILLISECONDS = 100;

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
 One captured stream of a command: what it delivered, and how to stop keeping it.

 Reading keeps the stream in flowing mode for as long as it stays open,
 so a process the command left behind can go on writing instead of dying of a broken pipe
 at its first write. What arrives after the call has taken its answer is read and dropped,
 so a program that logs for ever costs memory only until the call answers.
 */
type CollectedStream = {
  /**
   The text the stream delivered, taking it out of the collector so nothing more accumulates.
   */
  readonly take: () => string;

  /**
   Stops keeping what arrives, and lets the process end while the stream is still open.
   */
  readonly release: () => void;
};

/**
 Lets the process end while a stream of a child is still open.

 A piped child stream is a socket, which holds the event loop open until it is unref'd.
 The collector no longer keeps what arrives, so nothing is lost by letting the process go.

 @param stream - Stream to unref, when it has that ability.

 @example
 ```ts
 releaseHandle({ stream: child.stdout });
 ```
 */
function releaseHandle({ stream, }: { readonly stream: Readable; },): void {
  if (!('unref' in stream))
    return;
  /**
   The stream's own unref, which a socket has and a plain readable stream does not.
   */
  const { unref, } = stream;
  if ((typeof unref) === 'function')
    unref.call(stream,);
}

/**
 Collects what a stream delivers as UTF-8 text.

 `text()` from `node:stream/consumers` refuses when a stream closes before its end,
 and waiting for such a stream is waiting for whatever else holds its write end,
 which is why the text is taken rather than awaited:
 the command's own output is what the call answers with,
 and a process the command left behind goes on writing to a reader that no longer keeps it.

 @param stream - Standard output or standard error of a child that was piped.

 @returns The collector.

 @example
 ```ts
 const output = collectStream(child.stdout);
 output.take();
 ```
 */
function collectStream(stream: Readable,): CollectedStream {
  /**
   Chunks read so far, in the order they arrived, while they are still kept.
   */
  const chunks: Uint8Array[] = [];
  /**
   Whether arriving chunks are still kept.
   */
  const keeping = { on: true, };
  stream.on(
    'data',
    function keep(chunk: Uint8Array,): void {
      if (keeping.on)
        chunks.push(chunk,);
    },
  );
  stream.on(
    'error',
    function ignore(error: Readonly<Error>,): void {
      // A pipe that breaks after the call answered is not this call's failure:
      // the writer's own ending is its business, and the reader has already reported.
      l.debug(`a pipe of the command ended with ${error.message}`,);
    },
  );
  return {
    take: function take(): string {
      keeping.on = false;
      /**
       Bytes read so far, joined.
       */
      const joined = Buffer.concat(chunks,);
      chunks.length = 0;
      return joined.toString('utf8',);
    },
    release: function release(): void {
      keeping.on = false;
      chunks.length = 0;
      // A stream nobody keeps must not hold the process open: wg-quicker ends when its work does,
      // and a program it left writing simply loses the pipe then.
      releaseHandle({ stream, },);
    },
  };
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

   The child stays in the caller's process group, which is what a terminal sends its
   interrupt to: a detached child would never see the Ctrl-C a person pressed for the command.
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
   They stay read after the call answers, so a process the command left behind
   goes on writing instead of dying at its first write.
   */
  // `ChildProcess` types a stream as absent when it was not piped,
  // and this spawn always pipes both.
  /**
   Collected standard output, absent when it was not piped.
   */
  const output = child.stdout === null ? null : collectStream(child.stdout,);
  /**
   Collected standard error, absent when it was not piped.
   */
  const errors = child.stderr === null ? null : collectStream(child.stderr,);
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
   Settled when the command itself ended or could not start,
   which is earlier than the close of its pipes when it left a process behind.
   */
  const {
    promise: ended,
    resolve: resolveEnded,
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
      resolveEnded();
    },
  );
  child.once(
    'exit',
    function exited(): void {
      resolveEnded();
    },
  );
  child.once(
    'close',
    function closedNow(): void {
      clearTimeout(deadline,);
      resolveClosed();
      resolveEnded();
    },
  );
  /**
   The wait for a command that has ended,
   which the close of its pipes may outlast when it left a process behind.

   The command's own output is read as it arrives, so the drain only covers
   what was written just before the end.
   Then the call answers with the command's own status, as `wg-quick` does,
   and the streams stay read without being kept,
   so a program the command left behind keeps a standard output it can write to
   for as long as wg-quicker runs.
   */
  async function finishAfterEnd(): Promise<void> {
    await ended;
    await Promise.race([
      closed,
      wait(OUTPUT_DRAIN_MILLISECONDS,),
    ],);
  }
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
      resolvePastDeadline();
    },
    deadlineMilliseconds,
  );
  await Promise.race([
    finishAfterEnd(),
    pastDeadline,
  ],);
  clearTimeout(deadline,);
  if (observed.endedByDeadline) {
    // The kill was just sent: what the command had written before it arrives within the same drain.
    await Promise.race([
      ended,
      wait(OUTPUT_DRAIN_MILLISECONDS,),
    ],);
  }
  /**
   What the command printed, taken out of the collectors, which then stop keeping.
   */
  const stdout = output === null ? '' : output.take();
  /**
   What the command wrote to standard error.
   */
  const stderr = errors === null ? '' : errors.take();
  output?.release();
  errors?.release();
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
