/**
 What every workflow step program shares: the runner files a step appends to, running a program with the exit status
 the shell would have given, and the lines a step prints.

 These programs replace shell steps of `.github/workflows` and `.github/actions` (the workflow shell rules of
 `monochromatic-lint` ask for one command per step), so each prints and exits exactly as the shell step did. Imports only
 `node:` modules, because a step runs right after checkout, before any dependency is installed, with the Node the runner
 provides. The lines go to standard output and standard error as they are: they are the step's exact terminal output
 (annotation commands such as `::error::` included), not logs, so no tagged logger wraps them.

 @module
 */

import { spawn, } from 'node:child_process';
import { once, } from 'node:events';
import {
  appendFile,
  readFile,
} from 'node:fs/promises';
import { constants, } from 'node:os';

//region Ending a step with a given status

/**
 Thrown to end a step with a status, the way `exit N` ends a shell step; {@link runStep} turns it into the exit status.

 @example
 ```ts
 throw new StepExitError(1,);
 ```
 */
export class StepExitError extends Error {
  /**
   The exit status the step ends with.
   */
  readonly status: number;

  /**
   Creates the ending.

   @param status - The exit status the step ends with
   */
  constructor(status: number,) {
    super(`The step ends with status ${String(status,)}.`,);
    this.name = 'StepExitError';
    this.status = status;
  }
}

/**
 Exit status of a program the shell cannot find, which `bash` reports as 127.
 */
const COMMAND_NOT_FOUND_STATUS = 127;

/**
 Added to a signal number to make the status a shell reports for a program a signal ended.
 */
const SIGNAL_STATUS_BASE = 128;

/**
 Runs a step's work, then sets the process exit status; a thrown {@link StepExitError} sets its own status, any other error
 prints its message to standard error and sets status 1.

 @param work - The step's whole work

 @example
 ```ts
 await runStep(async function sayDone() {
   say('done',);
 },);
 ```
 */
export async function runStep(work: () => Promise<void>,): Promise<void> {
  try {
    await work();
  }
  catch (error) {
    if (error instanceof StepExitError) {
      process.exitCode = error.status;
      return;
    }
    process.stderr
      .write(`${String(error,)}\n`,);
    process.exitCode = 1;
  }
}

//endregion Ending a step with a given status

//region Printing, like `echo`

/**
 Prints a line to standard output, like `echo`.

 @param line - Text without its line break

 @example
 ```ts
 say('tag 0123456789abcdef',);
 ```
 */
export function say(line: string,): void {
  process.stdout
    .write(`${line}\n`,);
}

/**
 Prints a line to standard error, like `echo ... >&2`.

 @param line - Text without its line break

 @example
 ```ts
 sayOnStandardError('::error::something failed',);
 ```
 */
export function sayOnStandardError(line: string,): void {
  process.stderr
    .write(`${line}\n`,);
}

//endregion Printing, like `echo`

//region Environment and the runner's files

/**
 Reads an environment variable the step cannot work without, like `"$NAME"` under `set -u`.

 @param name - Variable name

 @returns The value, which may be empty

 @throws {@link Error} When the variable is not set, as `bash` stops with `NAME: unbound variable`

 @example
 ```ts
 const tools = requiredEnvironment('TOOLS',);
 ```
 */
export function requiredEnvironment(name: string,): string {
  /**
   The value in this process's environment.
   */
  const value = process.env[name];
  if (value === undefined)
    throw new Error(`${name}: unbound variable`,);
  return value;
}

/**
 Appends text to the file a runner variable names (`GITHUB_OUTPUT`, `GITHUB_ENV`, `GITHUB_PATH`,
 `GITHUB_STEP_SUMMARY`), the way `>> "$GITHUB_OUTPUT"` does; creates the file when it is missing.

 @param variable - Name of the runner variable that holds the file path

 @param text - Exact text to append, line breaks included

 @example
 ```ts
 await appendToRunnerFile({ variable: 'GITHUB_OUTPUT', text: 'value=abc\n', },);
 ```
 */
export async function appendToRunnerFile(
  {
    variable,
    text,
  }: {
    readonly variable: string;
    readonly text: string;
  },
): Promise<void> {
  await appendFile(
    requiredEnvironment(variable,),
    text,
    'utf8',
  );
}

//endregion Environment and the runner's files

//region Text the way `jq` and the shell give it

/**
 Exit status `jq` ends with when a file cannot be opened.
 */
const JQ_UNREADABLE_FILE_STATUS = 2;

/**
 Exit status `jq` ends with when its input is not JSON.
 */
const JQ_INVALID_JSON_STATUS = 5;

/**
 Reads a file's text, ending the step with the status `jq` gives for a file it cannot open.

 @param path - File to read

 @returns The file's text

 @throws {@link StepExitError} With status 2 when the file cannot be read

 @example
 ```ts
 const text = await readUnreadableAsExit('toolchain.lock.json',);
 ```
 */
async function readUnreadableAsExit(path: string,): Promise<string> {
  try {
    return await readFile(
      path,
      'utf8',
    );
  }
  catch (error) {
    process.stderr
      .write(`${path} could not be read (${String(error,)})\n`,);
    throw new StepExitError(JQ_UNREADABLE_FILE_STATUS,);
  }
}

/**
 Reads a JSON file the way `jq` opens one.

 @param path - File to read

 @returns The parsed JSON

 @throws {@link StepExitError} With the status `jq` ends with: 2 when the file cannot be read, 5 when it is not JSON

 @example
 ```ts
 const lock = await readJsonFile('toolchain.lock.json',);
 ```
 */
export async function readJsonFile(path: string,): Promise<unknown> {
  /**
   The file's text.
   */
  const text = await readUnreadableAsExit(path,);
  try {
    return JSON.parse(text,) as unknown;
  }
  catch (error) {
    process.stderr
      .write(`${path} is not JSON (${String(error,)})\n`,);
    throw new StepExitError(JQ_INVALID_JSON_STATUS,);
  }
}

/**
 Follows keys through nested JSON the way a `jq` path does: a missing step gives `undefined`, never an error.

 @param value - JSON to start from

 @param keys - Object keys to follow in order

 @returns What the path reaches, or `undefined`

 @example
 ```ts
 const url = jsonAt({ value: lock, keys: ['tools', 'mpm-shadow', 'artifacts', 'linux-x64', 'url',], },);
 ```
 */
export function jsonAt(
  {
    value,
    keys,
  }: {
    readonly value: unknown;
    readonly keys: readonly string[];
  },
): unknown {
  return keys.reduce(
    function stepInto(
      current: unknown,
      key: string,
    ) {
      if (((typeof current) !== 'object') || (current === null)
        || Array.isArray(current,))
        return undefined;
      return Reflect.get(
        current,
        key,
      ) as unknown;
    },
    value,
  );
}

/**
 Text of a JSON value the way `jq --raw-output` prints it: a string as it is, any other value as JSON, `null` as `null`.

 @param value - Value a path reached; `undefined` stands for a missing one

 @returns The text `jq --raw-output '<path>'` prints for it

 @example
 ```ts
 const text = rawJsonText(undefined,); // 'null'
 ```
 */
export function rawJsonText(value: unknown,): string {
  if ((typeof value) === 'string')
    return value;
  return JSON.stringify(value ?? null,);
}

/**
 Text of a JSON value the way `jq --raw-output '<path> // empty'` prints it: nothing for a missing, `null` or `false`
 value.

 @param value - Value a path reached; `undefined` stands for a missing one

 @returns The text, which is empty when the value is missing, `null` or `false`

 @example
 ```ts
 const version = rawJsonTextOrEmpty(jsonAt({ value: lock, keys: ['tools', id, 'version',], },),);
 ```
 */
export function rawJsonTextOrEmpty(value: unknown,): string {
  if ((value === undefined) || (value === null)
    || (value === false))
    return '';
  return rawJsonText(value,);
}

/**
 Finds where trailing line breaks start.

 @param text - Captured output

 @returns Index just past the last character that is not a line break

 @example
 ```ts
 const end = endBeforeTrailingLineBreaks('200\n',); // 3
 ```
 */
function endBeforeTrailingLineBreaks(text: string,): number {
  /**
   Index just past the last character that is not a line break, found by walking back from the end.
   */
  let end = text.length;
  while ((end > 0) && (text[end - 1] === '\n'))
    end -= 1;
  return end;
}

/**
 Drops the line breaks at the end of captured output, as `$(...)` does.

 @param text - Captured output

 @returns The text without trailing line breaks

 @example
 ```ts
 const status = withoutTrailingLineBreaks('200\n',); // '200'
 ```
 */
export function withoutTrailingLineBreaks(text: string,): string {
  return text.slice(
    0,
    endBeforeTrailingLineBreaks(text,),
  );
}

//endregion Text the way `jq` and the shell give it

//region Running programs

/**
 What a program run left behind.
 */
export type ProgramResult = {
  /**
   Exit status; 127 when the program could not be started, 128 plus the signal number when a signal ended it.
   */
  readonly status: number;
  /**
   Standard output, when it was captured rather than passed through to the step's own output; otherwise empty.
   */
  readonly stdout: string;
};

/**
 What to run: the program, its arguments, where, and what to give and keep.
 */
export type ProgramRequest = {
  /**
   Program name, found on `PATH`, or a path.
   */
  readonly file: string;
  /**
   Arguments, passed as they are with no shell between.
   */
  readonly args: readonly string[];
  /**
   Text for the program's standard input; the program gets no standard input when it is left out.
   */
  readonly input?: string;
  /**
   Whether to return standard output instead of printing it.
   */
  readonly captureStdout?: boolean;
  /**
   Directory to run in.
   */
  readonly cwd: string;
};

/**
 Runs a program and waits for it.

 Standard error always passes through to the step's own. Standard output passes through unless `captureStdout` is set,
 which is `$(...)`. A program that cannot be started prints `<file>: command not found` and gives status 127, like `bash`.

 @param file - Program name, found on `PATH`, or a path

 @param args - Arguments, passed as they are with no shell between

 @param input - Text for the program's standard input; the program gets no standard input when it is left out

 @param captureStdout - Whether to return standard output instead of printing it

 @param cwd - Directory to run in

 @returns The status and, when captured, the standard output

 @example
 ```ts
 const { status, } = await runProgram({ file: 'tar', args: ['--version',], cwd: process.cwd(), },);
 ```
 */
export async function runProgram(
  {
    file,
    args,
    input,
    captureStdout = false,
    cwd,
  }: ProgramRequest,
): Promise<ProgramResult> {
  /**
   The child, with standard error always shared with the step.
   */
  const child = spawn(
    file,
    [...args,],
    {
      cwd,
      stdio: [
        input === undefined ? 'ignore' : 'pipe',
        captureStdout ? 'pipe' : 'inherit',
        'inherit',
      ],
    },
  );
  /**
   Pieces of captured standard output.
   */
  const pieces: string[] = [];
  child.stdout
    ?.setEncoding('utf8',)
    .on(
    'data',
    function collectStdout(chunk: string,) {
      pieces.push(chunk,);
    },
    );
  // A child that exits before it reads its input closes the pipe; its own status is what the step reports.
  child.stdin
    ?.on(
    'error',
    function ignoreClosedInput() {
        // Nothing to do: the status below says how the child ended.
    },
    )
    .end(input,);
  try {
    /**
     How the child ended.
     */
    await once(
      child,
      'close',
    );
    if (child.exitCode === null)
      return {
        status: SIGNAL_STATUS_BASE + (constants.signals[child.signalCode ?? 'SIGTERM'] ?? 0),
        stdout: pieces.join('',),
      };
    return {
      status: child.exitCode,
      stdout: pieces.join('',),
    };
  }
  catch (error) {
    // `once` rejects when the program could not be started (a missing file, no permission to run it).
    process.stderr
      .write(`${file}: command not found (${String(error,)})\n`,);
    return {
      status: COMMAND_NOT_FOUND_STATUS,
      stdout: '',
    };
  }
}

/**
 Runs a program and ends the step with its status when that is not 0, like a command under `set -e`.

 @param request - What {@link runProgram} takes; the program's standard output passes through to the step

 @throws {@link StepExitError} With the program's own status when it is not 0

 @example
 ```ts
 await runProgramOrEnd({ file: 'podman', args: ['--version',], cwd: process.cwd(), },);
 ```
 */
export async function runProgramOrEnd(request: ProgramRequest,): Promise<void> {
  /**
   What the program left.
   */
  const { status, } = await runProgram(request,);
  if (status !== 0)
    throw new StepExitError(status,);
}

//endregion Running programs
