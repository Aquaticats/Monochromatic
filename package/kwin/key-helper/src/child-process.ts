/**
 Running one external command to completion and reporting how it ended.

 The daemon starts `ydotool` and the launcher and can do nothing about their
 failures, but it must not lose them either: a nonzero exit, a signal, or a
 command that could not be started all throw {@link CommandFailedError} naming
 the command, how it ended, and what it wrote to stderr. Callers log the error
 and carry on, so a broken injection never crashes the daemon.

 The one child-process shape of this package lives here, so `ydotool` and the
 launcher cannot drift apart in how they report a failure.

 @module
 */

import { execFile } from 'node:child_process';
import { promisify } from 'node:util';

import { CommandFailedError } from './errors.ts';

//region Promisified exec

/**
 Specialized promisified `execFile`, typed through Node's own adapter contract.
 The generic `promisify(execFile)` form trips `strict-void-return`;
 see `doc/troubleshooting/oxlint-promisify-void-return.md`.
 */
const promisifyExecFile: (original: typeof execFile) => typeof execFile.__promisify__ = promisify;

/**
 Runs a program without a shell and waits for it to end, rejecting with the
 runtime's own failure object, which carries `code`, `signal` and `stderr`.
 */
const execFileAsync: typeof execFile.__promisify__ = promisifyExecFile(execFile);

//endregion Promisified exec

//region Reporting

/**
 What a failed child reports about itself, read off the caught value.
 
 Every field is the empty string when the runtime did not supply it, so a
 caller reads absence as "this is not how it ended" rather than as a nullish
 branch.

 @example
 ```ts
 const failure: ChildFailure = childFailureOf(error);
 ```
 */
type ChildFailure = {
  /**
   Exit status the child reported, or the empty string when it did not exit.
   */
  readonly exitCode: string;
  /**
   Signal that ended the child, or the empty string when none did.
   */
  readonly signal: string;
  /**
   Text the child wrote to stderr, or the empty string.
   */
  readonly standardError: string;
  /**
   The runtime's own account, which explains a command that never started.
   */
  readonly message: string;
};

/**
 One command to run to completion.
 
 @example
 ```ts
 const request: RunCommandOptions = { command: 'ydotool', args: ['key', '29:1'] };
 ```
 */
export type RunCommandOptions = {
  /**
   Executable to run, resolved through `PATH` unless absolute.
   */
  readonly command: string;
  /**
   Arguments passed to the executable, never through a shell.
   */
  readonly args: readonly string[];
};

/**
 Read one field of a caught value without assuming its shape.
 
 @param source - Caught value of unknown type
 
 @param key - Field name to read
 
 @returns The field's value, or `undefined` when it cannot be read
 
 @example
 ```ts
 readField({ source: error, key: 'stderr' });
 ```
 */
function readField({
  source,
  key,
}: {
  readonly source: unknown;
  readonly key: string;
}): unknown {
  if (((typeof source) !== 'object') || (source === null)) {
    return undefined;
  }
  return Reflect.get(
    source,
    key
  );
}

/**
 Read a number field as text.
 
 @param value - Field value of unknown type
 
 @returns The number as text, or the empty string when it is not a number
 
 @example
 ```ts
 numberTextOf(7); // '7'
 ```
 */
function numberTextOf(value: unknown): string {
  return (typeof value) === 'number' ? String(value) : '';
}

/**
 Read a string field.
 
 @param value - Field value of unknown type
 
 @returns The string, or the empty string when it is not one
 
 @example
 ```ts
 textOf('boom'); // 'boom'
 ```
 */
function textOf(value: unknown): string {
  return (typeof value) === 'string' ? value : '';
}

/**
 Project the parts of a failed child this module reports.
 
 @param error - Value the command rejected with
 
 @returns Its exit status, signal, stderr and message, each possibly empty
 
 @example
 ```ts
 childFailureOf(error); // { exitCode: '7', signal: '', standardError: 'boom', message: '...' }
 ```
 */
function childFailureOf(error: unknown): ChildFailure {
  return {
    exitCode: numberTextOf(readField({
      source: error,
      key: 'code',
    })),
    signal: textOf(readField({
      source: error,
      key: 'signal',
    })),
    standardError: textOf(readField({
      source: error,
      key: 'stderr',
    })),
    message: textOf(readField({
      source: error,
      key: 'message',
    })),
  };
}

/**
 Say how one child ended, in the words an operator reads.
 
 @param failure - What the failed child reported
 
 @returns One clause naming the exit status, the signal, or the start failure
 
 @example
 ```ts
 endingTextOf({ exitCode: '7', signal: '', standardError: '', message: '' });
 // => 'exited with code 7'
 ```
 */
function endingTextOf(failure: ChildFailure): string {
  if (failure.exitCode !== '') {
    return `exited with code ${failure.exitCode}`;
  }
  if (failure.signal !== '') {
    return `was ended by ${failure.signal}`;
  }
  return `could not be started: ${failure.message}`;
}

/**
 Build the diagnostic for one failed child: the invocation, how it ended, and
 whatever it wrote to stderr.
 
 @param command - Executable that was run
 
 @param args - Arguments it was given
 
 @param failure - What the failed child reported
 
 @returns One line naming the command and its failure
 
 @example
 ```ts
 failureTextOf({ command: 'ydotool', args: ['key'], failure });
 // => "'ydotool key' exited with code 7; stderr: ydotoold is not running"
 ```
 */
function failureTextOf({
  command,
  args,
  failure,
}: {
  readonly command: string;
  readonly args: readonly string[];
  readonly failure: ChildFailure;
}): string {
  /**
   The command line an operator would retype.
   */
  const invocation = [
    command,
    ...args
  ].join(' ');
  /**
   Invocation and ending, the part every failure reports.
   */
  const headline = `'${invocation}' ${endingTextOf(failure)}`;
  /**
   stderr the child wrote, trimmed.
   */
  const output = failure.standardError
    .trim();
  return output === '' ? headline : `${headline}; stderr: ${output}`;
}

//endregion Reporting

//region Running

/**
 Run one command to completion, throwing unless it exited successfully.
 
 A successful child resolves; a nonzero exit, a signal, or a command that could
 not be started throws {@link CommandFailedError}, whose message names the
 command, how it ended, and any stderr it wrote.
 
 @param command - Executable to run
 
 @param args - Arguments passed to it
 
 @throws {@link CommandFailedError} when the command did not exit successfully
 
 @example
 ```ts
 await runCommand({ command: 'ydotool', args: ['key', '29:1'] });
 ```
 */
export async function runCommand({
  command,
  args
}: RunCommandOptions): Promise<void> {
  try {
    await execFileAsync(
      command,
      [...args]
    );
  } catch (error) {
    throw new CommandFailedError(failureTextOf({
      command,
      args,
      failure: childFailureOf(error),
    },));
  }
}

//endregion Running
