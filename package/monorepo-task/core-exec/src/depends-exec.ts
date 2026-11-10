/**
 Command execution with output collapsing for task-depends.
 
 Captures stdout/stderr during execution. On success, output is discarded
 (collapsed). On failure, captured output is dumped to the parent process and
 the child's status becomes this process's exit status, unless the call allows
 failure.
 
 @example
 ```ts
 await executeWithCollapsedOutput({
   command: 'mise',
   commandArgs: ['run', 'build'],
   verbose: false,
   allowFailure: false,
   commandTimeoutMs: 3_600_000,
 });
 ```
 */

import {
  spawnBounded,
  timeoutNote,
} from './depends-bounded-wait.ts';
import { isCapturedSubprocessFailure, } from './tool-failure.ts';

//region Types

/**
 Options for collapsed-output command execution.
 
 @example
 ```ts
 const options: ExecuteOptions = {
   command: 'bun',
   commandArgs: ['build'],
   verbose: false,
   allowFailure: false,
   commandTimeoutMs: 3_600_000,
 };
 ```
 */
export type ExecuteOptions = {
  /**
   Executable to run
   */
  readonly command: string;
  /**
   Arguments passed to the command
   */
  readonly commandArgs: readonly string[];
  /**
   Whether to log diagnostic messages
   */
  readonly verbose: boolean;
  /**
   Whether to suppress command failures (exit 0 regardless)
   */
  readonly allowFailure: boolean;
  /**
   How long the command may run before it is ended, in milliseconds
   */
  readonly commandTimeoutMs: number;
};

//endregion Types

//region Execution: run command with collapsed output

/**
 Executes a command, capturing its output. Shows output only on failure.
 
 On success, stdout/stderr are discarded. On failure, captured output is
 written to the parent's stdout/stderr by {@link dumpAndHandleError}, which
 then sets the process exit status unless `allowFailure` is true.
 
 @param command - Executable to run
 
 @param commandArgs - Arguments passed to the command
 
 @param verbose - Whether to log diagnostic messages
 
 @param allowFailure - Whether to suppress command failures (exit 0 regardless)
 
 @param commandTimeoutMs - How long the command may run before it is ended; an ended command counts as a failure
 
 @example
 ```ts
 await executeWithCollapsedOutput({
   command: 'mise',
   commandArgs: ['run', 'build'],
   verbose: true,
   allowFailure: false,
   commandTimeoutMs: 3_600_000,
 });
 ```
 */
export async function executeWithCollapsedOutput({
  command,
  commandArgs,
  verbose,
  allowFailure,
  commandTimeoutMs,
}: ExecuteOptions,): Promise<void> {
  if (verbose)
    console.error(`[task-depends] running: ${command} ${commandArgs.join(' ',)}`,);

  try {
    // nano-spawn defaults to 'pipe', capturing stdout/stderr
    await spawnBounded({
      file: command,
      args: commandArgs,
      options: {},
      timeoutMs: commandTimeoutMs,
    },);

    if (verbose)
      console.error('[task-depends] command completed successfully',);
  }
  catch (error) {
    /**
     Sentence naming the bound when it ended the command; empty when the command failed by itself.
     */
    const note = timeoutNote(error,);
    if (note !== '') {
      console.error(
        `[task-depends] command ${note} (raise it with --command-timeout-seconds): ${command} ${commandArgs.join(' ',)}`,
      );
    }
    dumpAndHandleError({
      error,
      allowFailure,
    },);
  }
}

/**
 Options for {@link dumpAndHandleError}.
 
 @example
 ```ts
 const options: DumpAndHandleErrorOptions = {
   error: new Error('boom'),
   allowFailure: false,
 };
 ```
 */
type DumpAndHandleErrorOptions = {
  /**
   Captured failure thrown by {@link spawnBounded}: nano-spawn's SubprocessError,
   or the bound's own error carrying it in `cause`
   */
  readonly error: unknown;
  /**
   Whether to suppress the failure
   */
  readonly allowFailure: boolean;
};

/**
 Dumps captured output from a failed subprocess and handles the error.
 
 Writes any captured stdout/stderr to the parent process streams so the
 user can see what went wrong. Then, unless `allowFailure` is set, makes the
 child's status this process's exit status.
 
 Two failures carry no captured status of their own and so have nothing to
 dump: a command that never started, and the bound ending the command, whose
 subprocess error stays in `cause`. Both still end this process as failed.
 
 @param error - Captured failure thrown by {@link spawnBounded}
 
 @param allowFailure - Whether to suppress the failure
 
 @example
 ```ts
 try {
   await spawnBounded({ file: 'cmd', args: [], options: {}, timeoutMs: 60_000 },);
 }
 catch (error) {
   dumpAndHandleError({ error, allowFailure: false },);
 }
 ```
 */
function dumpAndHandleError({
  error,
  allowFailure,
}: DumpAndHandleErrorOptions,): void {
  if (isCapturedSubprocessFailure(error,)) {
    // Dump captured output so the user can see what happened
    if ((error.stdout
      !== undefined) && (error.stdout
        !== ''))
      process.stdout
        .write(`${error.stdout}\n`,);
    if ((error.stderr
      !== undefined) && (error.stderr
        !== ''))
      process.stderr
        .write(`${error.stderr}\n`,);

    if ((error.signalName
      !== undefined)
      && (error.signalName
        !== ''))
      {
      console.error(
        `[task-depends] command terminated by signal: ${error.signalName}`,
      );
    }

    if (!allowFailure)
      process.exitCode = error.exitCode
        ?? 1;
    return;
  }

  // A failed start reports no status, and the bound's own error keeps the subprocess error in `cause`,
  // so neither has captured output to dump; the command failed all the same.
  if (!allowFailure)
    process.exitCode = 1;
}

//endregion Execution
