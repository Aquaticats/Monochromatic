/**
 Command execution with output collapsing for task-depends.
 
 Captures stdout/stderr during execution. On success, output is discarded
 (collapsed). On failure, captured output is dumped to the parent process
 before propagating the error.
 
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

import { caughtValueText, } from '@monochromatic-dev/caught-value-core/ts';

import {
  spawnBounded,
  timeoutNote,
} from './depends-bounded-wait.ts';

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
 written to the parent's stdout/stderr by {@link dumpAndHandleError} before
 the error is propagated (or swallowed when `allowFailure` is true).
 
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
   Error thrown by nano-spawn (typically SubprocessError)
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
 user can see what went wrong. Then either sets a non-zero exit code or
 swallows the error based on the `allowFailure` flag.
 
 @param error - Error thrown by nano-spawn (typically SubprocessError)
 
 @param allowFailure - Whether to suppress the failure
 
 @example
 ```ts
 try { await spawn('cmd'); } catch (e) { dumpAndHandleError({ error: e, allowFailure: false }); }
 ```
 */
function dumpAndHandleError({
  error,
  allowFailure,
}: DumpAndHandleErrorOptions,): void {
  // SubprocessError from nano-spawn includes captured stdout/stderr
  if ((error !== null) && ((typeof error) === 'object')) {
    /**
     Re-typed thrown error so its captured subprocess fields can be dumped to the parent streams.
     */
    const subprocessError = error as {
      readonly stdout?: string;
      readonly stderr?: string;
      readonly exitCode?: number;
      readonly signalName?: string;
      readonly message?: string;
    };

    // Dump captured output so the user can see what happened
    if ((subprocessError.stdout
      !== undefined) && (subprocessError.stdout
        !== ''))
      process.stdout
        .write(`${subprocessError.stdout}\n`,);
    if ((subprocessError.stderr
      !== undefined) && (subprocessError.stderr
        !== ''))
      process.stderr
        .write(`${subprocessError.stderr}\n`,);

    if ((subprocessError.signalName
      !== undefined)
      && (subprocessError.signalName
        !== ''))
      {
      console.error(
        `[task-depends] command terminated by signal: ${subprocessError.signalName}`,
      );
    }

    if (!allowFailure)
      process.exitCode = subprocessError.exitCode
        ?? 1;
    return;
  }

  // Non-subprocess error (e.g. command not found)
  console.error(
    `[task-depends] failed to execute command: ${
      caughtValueText(error,)
    }`,
  );

  if (!allowFailure)
    throw error;
}

//endregion Execution
