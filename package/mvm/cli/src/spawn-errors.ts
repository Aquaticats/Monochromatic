/**
 Errors raised when a host command mvm runs cannot start, fails, or outlasts its deadline.

 @module
 */

//region Command line rendering

/**
 Longest argument shown in full in a message or log line; guest agent requests can carry megabytes.
 */
const SHOWN_ARGUMENT_CHARACTERS = 300;

/**
 Renders a command and its arguments for a message or log line.
 Long arguments are cut and say how much was left out; the text is for reading, not for running.

 @param args - Arguments of the command

 @param command - Executable name or path

 @returns Command line text

 @example
 ```ts
 renderCommandLine({ args: ['list', '--all'], command: 'virsh' }); // => 'virsh list --all'
 ```
 */
export function renderCommandLine({
  args,
  command,
}: {
  readonly args: readonly string[];
  readonly command: string;
},): string {
  return [
    command,
    ...args.map(function shorten(argument,) {
      if (argument.length <= SHOWN_ARGUMENT_CHARACTERS) {
        return argument;
      }
      return `${
        argument.slice(
          0,
          SHOWN_ARGUMENT_CHARACTERS,
        )
      }...(${String(argument.length - SHOWN_ARGUMENT_CHARACTERS,)} more characters)`;
    },),
  ].join(' ',);
}

//endregion Command line rendering

//region Errors

/**
 The executable of a command does not exist, so nothing ran.

 @example
 ```ts
 try {
   await spawn({ args: ['--version'], command: 'virsh' });
 }
 catch (error) {
   if (error instanceof ExecutableNotFoundError) console.error(error.executable);
 }
 ```
 */
export class ExecutableNotFoundError extends Error {
  /**
   Executable name or path that was not found.
   */
  readonly executable: string;

  /**
   @param cause - Spawn failure reported by the operating system

   @param executable - Executable name or path that was looked for

   @param remedy - How to make the executable available or point mvm at another one; `''` when the caller knows none
   */
  constructor({
    cause,
    executable,
    remedy,
  }: {
    readonly cause: unknown;
    readonly executable: string;
    readonly remedy: string;
  },) {
    super(
      [
        executable.includes('/',)
          ? `The executable ${executable} does not exist, so nothing ran.`
          : `The executable \`${executable}\` was not found in any directory of PATH, so nothing ran.`,
        ...(remedy === '' ? [] : [remedy,]),
      ].join('\n',),
      { cause, },
    );
    this.name = 'ExecutableNotFoundError';
    this.executable = executable;
  }
}

/**
 A command ran and ended with a nonzero exit status or by a signal.

 @example
 ```ts
 try {
   await spawn({ args: ['start', 'mvm-dev'], command: 'virsh' });
 }
 catch (error) {
   if (error instanceof CommandFailedError) console.error(error.stderr);
 }
 ```
 */
export class CommandFailedError extends Error {
  /**
   Exit status of the command; absent when a signal ended it.
   */
  readonly exitCode?: number;

  /**
   Name of the signal that ended the command; absent on a normal exit.
   */
  readonly signal?: string;

  /**
   Everything the command wrote to standard error.
   */
  readonly stderr: string;

  /**
   Everything the command wrote to standard output.
   */
  readonly stdout: string;

  /**
   @param commandLine - Rendered command line, named in the message

   @param exitCode - Exit status, when the command exited by itself

   @param signal - Signal name, when a signal ended the command

   @param stderr - Standard-error text, quoted in the message

   @param stdout - Standard-output text, kept for callers
   */
  constructor({
    commandLine,
    exitCode,
    signal,
    stderr,
    stdout,
  }: {
    readonly commandLine: string;
    readonly exitCode?: number;
    readonly signal?: string;
    readonly stderr: string;
    readonly stdout: string;
  },) {
    super(
      [
        exitCode === undefined
          ? `Command was ended by ${signal ?? 'a signal'}: ${commandLine}`
          : `Command exited with status ${String(exitCode,)}: ${commandLine}`,
        ...(stderr.trim() === '' ? [] : [stderr.trim(),]),
      ].join('\n',),
    );
    this.name = 'CommandFailedError';
    if (exitCode !== undefined)
      this.exitCode = exitCode;
    if (signal !== undefined)
      this.signal = signal;
    this.stderr = stderr;
    this.stdout = stdout;
  }
}

/**
 A command was still running at its deadline and was stopped.

 @example
 ```ts
 try {
   await spawn({ args: ['list'], command: 'virsh', deadlineMs: 120_000 });
 }
 catch (error) {
   if (error instanceof CommandTimedOutError) console.error(error.deadlineMs);
 }
 ```
 */
export class CommandTimedOutError extends Error {
  /**
   Milliseconds the command was given.
   */
  readonly deadlineMs: number;

  /**
   @param commandLine - Rendered command line, named in the message

   @param deadlineMs - Milliseconds the command was given
   */
  constructor({
    commandLine,
    deadlineMs,
  }: {
    readonly commandLine: string;
    readonly deadlineMs: number;
  },) {
    super(`Command did not finish within ${String(deadlineMs,)} ms and was stopped: ${commandLine}`,);
    this.name = 'CommandTimedOutError';
    this.deadlineMs = deadlineMs;
  }
}

//endregion Errors

//region Recognizing a missing executable

/**
 Error code the operating system reports when an executable does not exist.
 */
const MISSING_EXECUTABLE_CODE = 'ENOENT';

/**
 Reads the `code` property of a caught value, when it has one.

 @param value - Caught value or its cause

 @returns The code, or `''` when the value carries none

 @example
 ```ts
 codeOf(Object.assign(new Error('spawn x ENOENT'), { code: 'ENOENT' })); // => 'ENOENT'
 ```
 */
function codeOf(value: unknown,): string {
  if (((typeof value) === 'object') && (value !== null)
    && ('code' in value)
    && ((typeof value.code) === 'string')) {
    return value.code;
  }
  return '';
}

/**
 Checks whether a caught spawn failure means the executable does not exist.
 Covers the error `node:child_process` emits and a `nano-spawn` error, which carries it as its cause.

 @param error - Value caught from a failed spawn

 @returns Whether the operating system reported a missing executable

 @example
 ```ts
 try {
   await nanoSpawn('ssh', ['-V']);
 }
 catch (error) {
   if (isMissingExecutable(error)) throw new ExecutableNotFoundError({ cause: error, executable: 'ssh', remedy: '' });
 }
 ```
 */
export function isMissingExecutable(error: unknown,): boolean {
  if (codeOf(error,) === MISSING_EXECUTABLE_CODE) {
    return true;
  }
  return ((typeof error) === 'object')
    && (error !== null)
    && ('cause' in error)
    && (codeOf(error.cause,) === MISSING_EXECUTABLE_CODE);
}

//endregion Recognizing a missing executable
