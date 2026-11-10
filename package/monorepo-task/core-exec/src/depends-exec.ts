/**
 Command execution with output collapsing for task-depends.
 
 Captures stdout/stderr during execution. On success, output is discarded
 (collapsed). On failure, captured output is dumped to the parent process and
 the child's status becomes this process's exit status, unless the call allows
 failure.
 
 Every reachable failure shape is reported, because each one hides its reason
 in a different place: a command that ran keeps its captured streams and status
 on the thrown value, a command that outlived its bound keeps them in the
 bound error's `cause`, and a command that never started has neither, so its
 spawn reason is the only account of why the task failed.
 
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
  BoundedWaitTimeoutError,
  spawnBounded,
  timeoutNote,
} from './depends-bounded-wait.ts';
import {
  type CapturedSubprocessFailure,
  isCapturedSubprocessFailure,
} from './tool-failure.ts';

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
      commandLine: commandLineOf({
        command,
        commandArgs,
      },),
    },);
  }
}

/**
 The command as the task named it, for a message a person reads.
 
 @param command - Executable to run
 
 @param commandArgs - Arguments passed to the executable
 
 @returns executable and arguments joined the way the verbose line joins them
 
 @example
 ```ts
 commandLineOf({ command: 'mise', commandArgs: ['run', 'build'], },); // 'mise run build'
 ```
 */
function commandLineOf({
  command,
  commandArgs,
}: {
  readonly command: string;
  readonly commandArgs: readonly string[]
},): string {
  return commandArgs.length === 0
    ? command
    : `${command} ${commandArgs.join(' ',)}`;
}

/**
 Options for {@link dumpAndHandleError}.
 
 @example
 ```ts
 const options: DumpAndHandleErrorOptions = {
   error: new Error('boom'),
   allowFailure: false,
   commandLine: 'mise run build',
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
  /**
   Executable and arguments as the task named them, so a failed start names
   the command a person has to look at
   */
  readonly commandLine: string;
};

/**
 Dumps what a failed command printed and handles the failure.
 
 Writes any captured stdout/stderr to the parent process streams so the
 user can see what went wrong, and reports why a command that never started
 could not be started. Then, unless `allowFailure` is set, makes the child's
 status this process's exit status.
 
 @param error - Captured failure thrown by {@link spawnBounded}
 
 @param allowFailure - Whether to suppress the failure
 
 @param commandLine - Executable and arguments as the task named them
 
 @example
 ```ts
 try {
   await spawnBounded({ file: 'cmd', args: [], options: {}, timeoutMs: 60_000 },);
 }
 catch (error) {
   dumpAndHandleError({ error, allowFailure: false, commandLine: 'cmd' },);
 }
 ```
 */
function dumpAndHandleError({
  error,
  allowFailure,
  commandLine,
}: DumpAndHandleErrorOptions,): void {
  if (error instanceof BoundedWaitTimeoutError) {
    /*
     The bound keeps nano-spawn's error in `cause`, and that error holds what the
     child printed before its group was ended. The bound's own line, printed by
     the caller, already says the bound ended it, so no signal is named here.
     */
    if (isCapturedSubprocessFailure(error.cause,))
      dumpCapturedStreams(error.cause,);

    if (!allowFailure)
      process.exitCode = 1;
    return;
  }

  if (isCapturedSubprocessFailure(error,)) {
    dumpCapturedStreams(error,);

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

  // A command that never started reports no status and nothing on either stream, so the reason is
  // the only account of the failure a person can get.
  reportStartFailure({
    error,
    commandLine,
  },);

  if (!allowFailure)
    process.exitCode = 1;
}

/**
 Writes the captured streams of a failed child to this process's streams.
 
 nano-spawn captures a child's output with the final newline removed, so each
 stream gets one back and a following line starts on its own line.
 
 @param captured - what nano-spawn captured; both streams may be empty
 
 @example
 ```ts
 dumpCapturedStreams({ stdout: 'built', stderr: '', },);
 ```
 */
function dumpCapturedStreams(captured: CapturedSubprocessFailure,): void {
  if ((captured.stdout
    !== undefined) && (captured.stdout
      !== ''))
    process.stdout
      .write(`${captured.stdout}\n`,);
  if ((captured.stderr
    !== undefined) && (captured.stderr
      !== ''))
    process.stderr
      .write(`${captured.stderr}\n`,);
}

/**
 Reports a command this process could not start: the command line, what the
 spawn call said, and what to check next.
 
 @param error - value nano-spawn threw for the failed start
 
 @param commandLine - executable and arguments as the task named them
 
 @example
 ```ts
 reportStartFailure({ error, commandLine: 'no-such-command', },);
 ```
 */
function reportStartFailure({
  error,
  commandLine,
}: {
  readonly error: unknown;
  readonly commandLine: string
},): void {
  console.error(
    `[task-depends] could not start \`${commandLine}\`: ${startFailureDetail(error,)}. ${startFailureCheck(error,)}`,
  );
}

/**
 What to check when a failed start reported no reason code this file can act on,
 such as a command this host cannot start for a reason of its own.
 */
const UNKNOWN_START_CHECK_TEXT = 'Check that the task names a command this host can run.';

/**
 Explains what to check for a command that could not be started.
 
 @param error - value nano-spawn threw for the failed start
 
 @returns sentence naming the checks that would make the command start
 
 @example
 ```ts
 startFailureCheck(error,);
 ```
 */
function startFailureCheck(error: unknown,): string {
  if (!hasCause(error,))
    return UNKNOWN_START_CHECK_TEXT;
  /**
   What the failed start reported as its reason.
   */
  const { cause, } = error;
  if (!hasReasonCode(cause,))
    return UNKNOWN_START_CHECK_TEXT;
  /**
   Node's code for the failed system call, such as `ENOENT`.
   */
  const reasonCode = cause.code;
  if (reasonCode === 'ENOENT')
    return 'Check that the executable is installed and on PATH, and that the task names it correctly.';
  if (reasonCode === 'EACCES')
    return 'Check that the file is executable and that this account may run it.';

  return UNKNOWN_START_CHECK_TEXT;
}

/**
 Text the spawn call reported for a command that never started.
 
 The innermost reason wins: Node's own message names the failed system call and
 its code (`spawn tsc ENOENT`), which nano-spawn only repeats as a command line
 in its own message (`Command failed: tsc`).
 
 @param error - value nano-spawn threw for the failed start
 
 @returns the reason as text, never empty
 
 @example
 ```ts
 startFailureDetail(error,); // 'spawn tsc ENOENT'
 ```
 */
function startFailureDetail(error: unknown,): string {
  if (hasCause(error,)) {
    /**
     What the thrower reported as the reason.
     */
    const { cause, } = error;
    if (hasMessage(cause,))
      return cause.message;
  }
  if (hasMessage(error,))
    return error.message;

  return caughtValueText(error,);
}

/**
 A thrown value that keeps another thrown value in `cause`, as Node's spawn
 errors and nano-spawn's wrapped errors both do.
 */
type ThrownValueWithCause = {
  /**
   What the thrower reported as the reason.
   */
  readonly cause: unknown;
};

/**
 A thrown value that carries a non-empty message.
 */
type ThrownValueWithMessage = {
  /**
   Text the thrower reported.
   */
  readonly message: string;
};

/**
 A thrown value that carries the reason code of a failed system call.
 */
type ThrownValueWithReasonCode = {
  /**
   Code such as `ENOENT`, as Node reports it.
   */
  readonly code: string;
};

/**
 Whether a thrown value keeps another thrown value in `cause`.
 
 @param value - value to inspect
 
 @returns whether the value carries a `cause`
 
 @example
 ```ts
 if (hasCause(error,)) read(error.cause,);
 ```
 */
function hasCause(value: unknown,): value is ThrownValueWithCause {
  if ((value === null) || ((typeof value) !== 'object'))
    return false;

  return 'cause' in value;
}

/**
 Whether a thrown value carries a non-empty message.
 
 @param value - value to inspect
 
 @returns whether the value carries text a person can read
 
 @example
 ```ts
 if (hasMessage(error,)) print(error.message,);
 ```
 */
function hasMessage(value: unknown,): value is ThrownValueWithMessage {
  if ((value === null) || ((typeof value) !== 'object'))
    return false;
  if (!('message' in value))
    return false;

  return ((typeof value.message) === 'string') && (value.message !== '');
}

/**
 Whether a thrown value carries the reason code of a failed system call.
 
 @param value - value to inspect
 
 @returns whether the value carries a code such as `ENOENT`
 
 @example
 ```ts
 if (hasReasonCode(cause,)) print(cause.code,);
 ```
 */
function hasReasonCode(value: unknown,): value is ThrownValueWithReasonCode {
  if ((value === null) || ((typeof value) !== 'object'))
    return false;
  if (!('code' in value))
    return false;

  return (typeof value.code) === 'string';
}

//endregion Execution
