/**
 Runs one command and hands back what it printed, so the parts of the watch that ask the machine questions can be given the
 answers in a test instead of asking it.

 @module
 */

import { execFile, } from 'node:child_process';
import { promisify, } from 'node:util';
import { MILLISECONDS_PER_SECOND, } from './driver-constants.ts';

/**
 Bytes in one mebibyte.
 */
const BYTES_PER_MEBIBYTE = 1_048_576;

/**
 How many mebibytes of output a command may print before it is cut off. `ps -e` with its arguments printed 560 KB on this
 machine with 5,348 processes on 2026-11-08, and a machine with many more processes, or much longer command lines, still fits.
 */
const COMMAND_OUTPUT_MEBIBYTES = 8;

/**
 The largest output a command may print.
 */
const COMMAND_OUTPUT_BYTES: number = COMMAND_OUTPUT_MEBIBYTES * BYTES_PER_MEBIBYTE;

/**
 Seconds a question of the machine may take. `systemctl show` and `ps -e` each answered in tens of milliseconds on this machine
 on 2026-11-08, so a command still running after half a minute is one that will not answer, and the look must not wait for it.
 */
const COMMAND_TIMEOUT_SECONDS = 30;

/**
 Milliseconds a question of the machine may take when the caller names no bound of its own.
 */
const COMMAND_TIMEOUT_MILLISECONDS: number = COMMAND_TIMEOUT_SECONDS * MILLISECONDS_PER_SECOND;

/**
 `execFile` with a promise result, which never goes through a shell. Typed through Node's own adapter contract, because the
 generic `promisify(execFile)` form trips `strict-void-return`
 (`doc/troubleshooting/oxlint-promisify-void-return.md`).
 */
const promisifyExecFile: (original: typeof execFile) => typeof execFile.__promisify__ = promisify;

/**
 Runs a program without a shell and waits for it to end.
 */
const execFileAsync: typeof execFile.__promisify__ = promisifyExecFile(execFile,);

/**
 What one command printed and how it ended.
 */
export type CommandOutput = {
  /**
   The status the command exited with, or minus one when it never ran to an exit.
   */
  readonly status: number;

  /**
   What it printed.
   */
  readonly stdout: string;

  /**
   What it complained about.
   */
  readonly stderr: string;

  /**
   Why it never ran to an exit, when it never did.
   */
  readonly problem?: string;
};

/**
 Asks the machine one question.
 */
export type RunCommand = (request: {
  readonly command: string;
  readonly words: readonly string[];

  /**
   The folder the command runs in; absent for this process's own.
   */
  readonly cwd?: string;

  /**
   Variables this process's own environment is overridden with; absent to inherit all of it. A Git command that may be run
   where a person is watching gets no prompt for a credential this way.
   */
  readonly environment?: Readonly<Record<string, string>>;
}) => Promise<CommandOutput>;

/**
 Reads one field of a caught value, which is an error object this cannot name.

 @param caught - what was caught, known to be an object

 @param key - the field's name

 @returns the field's value, as anything

 @example
 ```ts
 const code = fieldOf({ caught, key: 'code', },);
 ```
 */
function fieldOf({
  caught,
  key,
}: {
  readonly caught: object;
  readonly key: string;
},): unknown {
  return Reflect.get(
    caught,
    key,
  );
}

/**
 Reads a field that should hold text.

 @param value - the field's value

 @returns the text, or empty text when the field holds anything else

 @example
 ```ts
 const shown = textFieldOf('',);
 ```
 */
function textFieldOf(value: unknown,): string {
  return ((typeof value) === 'string') ? value : '';
}

/**
 Reads the status a command exited with.

 @param value - the status the failure carries

 @returns the status, or minus one when the failure carries none

 @example
 ```ts
 const status = statusFieldOf(undefined,);
 ```
 */
function statusFieldOf(value: unknown,): number {
  return ((typeof value) === 'number') ? value : -1;
}

/**
 Reads what a failed command reported.

 @param caught - what the failed command threw

 @returns its status, its output, and why it never ran to an exit when it never did

 @example
 ```ts
 const failed = failureOf(new Error('x'),);
 ```
 */
function failureOf(caught: unknown,): CommandOutput {
  if (((typeof caught) !== 'object') || (caught === null)) {
    return {
      status: -1,
      stdout: '',
      stderr: '',
      problem: String(caught,),
    };
  }
  /**
   The failure in the shape this reads.
   */
  const failure: object = caught;
  /**
   The status the command exited with.
   */
  const status = statusFieldOf(fieldOf({
    caught: failure,
    key: 'code',
  },),);
  return {
    status,
    stdout: textFieldOf(fieldOf({
      caught: failure,
      key: 'stdout',
    },),),
    stderr: textFieldOf(fieldOf({
      caught: failure,
      key: 'stderr',
    },),),
    ...(status === (-1) ? { problem: textFieldOf(fieldOf({
      caught: failure,
      key: 'message',
    },),), } : {}),
  };
}

/**
 Runs one command.

 @param command - the program, found on `PATH`

 @param words - its arguments, each kept whole

 @param timeoutMilliseconds - how long it may take before it is killed

 @returns what it printed and how it ended

 @example
 ```ts
 const shown = await runCommandWithChildProcess({ command: 'systemctl', words: ['--user', 'show', 'pi-a.service',], },);
 ```
 */
export async function runCommandWithChildProcess({
  command,
  words,
  cwd,
  environment,
  timeoutMilliseconds = COMMAND_TIMEOUT_MILLISECONDS,
}: {
  readonly command: string;
  readonly words: readonly string[];
  readonly cwd?: string;
  readonly environment?: Readonly<Record<string, string>>;
  readonly timeoutMilliseconds?: number;
},): Promise<CommandOutput> {
  try {
    /**
     What the command printed.
     */
    const done = await execFileAsync(
      command,
      [...words,],
      {
        encoding: 'utf8',
        maxBuffer: COMMAND_OUTPUT_BYTES,
        timeout: timeoutMilliseconds,
        killSignal: 'SIGKILL',
        ...((cwd === undefined) ? {} : { cwd, }),
        ...((environment === undefined) ? {} : { env: {
          ...process.env,
          ...environment,
        }, }),
      },
    );
    return {
      status: 0,
      stdout: done.stdout,
      stderr: done.stderr,
    };
  }
  catch (caught) {
    // A command that exits non-zero, or that ran past its own bound and was killed, rejects here, and its own output is what
    // the caller reads.
    return failureOf(caught,);
  }
}
