/**
 Runs one command and hands back what it printed, so the parts of the watch that ask the machine questions can be given the
 answers in a test instead of asking it.

 @module
 */

import {
  execFile,
  spawn,
} from 'node:child_process';
import { once, } from 'node:events';
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

  /**
   Whether the bound this module was given is what ended the command.
   */
  readonly killedByItsOwnBound?: boolean;

  /**
   The signal that ended the command, when a signal ended it rather than an exit.
   */
  readonly signal?: string;
};

/**
 Asks the machine one question.
 */
export type RunCommand = (request: {
  readonly command: string;
  readonly words: readonly string[];

  /**
   The folder the command runs in, which is this process's own when it names none.
   */
  readonly folder?: string;

  /**
   Variables to add to the environment the command inherits, for a program that would ask a question without them.
   */
  readonly environment?: Readonly<Record<string, string>>;

  /**
   How long the command may take, when the caller wants a bound other than the one this module ships.
   */
  readonly timeoutMilliseconds?: number;
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
 Reads whether the bound this module set is what ended the command, which Node reports as `killed` and a killed command says
 nothing else about.

 @param caught - the failure the command threw

 @returns whether the bound ended it

 @example
 ```ts
 const killedByTheBound = killedByItsOwnBoundIn({ caught, },);
 ```
 */
function killedByItsOwnBoundIn({ caught, }: { readonly caught: object },): boolean {
  return fieldOf({
    caught,
    key: 'killed',
  },) === true;
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
  /**
   The signal that ended the command, empty when it exited rather than being signalled.
   */
  const signal = textFieldOf(fieldOf({
    caught: failure,
    key: 'signal',
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
    // A command that a signal ended is a command with no status, whether the signal was this module's own bound or somebody
    // else's, and the two are told apart by `killed` alone: Node sets it when the bound is what it signalled.
    ...((signal === '') ? {} : { signal, }),
    ...(killedByItsOwnBoundIn({ caught: failure, },) ? { killedByItsOwnBound: true, } : {}),
  };
}

/**
 Runs one command.

 @param command - the program, found on `PATH`

 @param words - its arguments, each kept whole

 @param timeoutMilliseconds - how long it may take before it is killed

 @param folder - the folder it runs in, which is this process's own when it names none

 @param environment - variables to add to the environment it inherits

 @returns what it printed and how it ended

 @example
 ```ts
 const shown = await runCommandWithChildProcess({ command: 'systemctl', words: ['--user', 'show', 'pi-a.service',], },);
 ```
 */
export async function runCommandWithChildProcess({
  command,
  words,
  timeoutMilliseconds = COMMAND_TIMEOUT_MILLISECONDS,
  folder,
  environment,
}: {
  readonly command: string;
  readonly words: readonly string[];
  readonly timeoutMilliseconds?: number;
  readonly folder?: string;
  readonly environment?: Readonly<Record<string, string>>;
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
        ...(folder === undefined ? {} : { cwd: folder, }),
        ...(environment === undefined ? {} : { env: {
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

/**
 What a command that showed its own output ended with.
 */
export type ShownCommandEnding = {
  /**
   The status it exited with, or nothing when a signal ended it.
   */
  readonly status?: number;

  /**
   Why it could not be started, when it could not be.
   */
  readonly startProblem?: string;
};

/**
 Runs one command with the caller's own streams, so what it prints reaches the terminal as it prints it, and answers with how it
 ended.

 This is the shape a launcher needs: a run's progress must show while it runs, and its status is what the caller answers with.

 @param command - the program, found on `PATH`

 @param words - its arguments, each kept whole

 @param folder - the folder it runs in, which is this process's own when it names none

 @param environment - the environment it gets, which is this process's own when it names none

 @returns how it ended, which never throws for a failure of the command itself

 @example
 ```ts
 const ended = await runCommandShowingItsOutput({ command: 'systemd-run', words: ['--user', '--wait', '--unit=pi-a', 'true',], },);
 ```
 */
export async function runCommandShowingItsOutput({
  command,
  words,
  folder,
  environment,
}: {
  readonly command: string;
  readonly words: readonly string[];
  readonly folder?: string;
  readonly environment?: NodeJS.ProcessEnv;
},): Promise<ShownCommandEnding> {
  /**
   The command's process.
   */
  const child = spawn(
    command,
    [...words,],
    {
      stdio: 'inherit',
      ...(folder === undefined ? {} : { cwd: folder, }),
      ...(environment === undefined ? {} : { env: environment, }),
    },
  );
  try {
    /**
     What the process reported when it closed, which is its status or the signal that ended it.
     */
    const closed: readonly unknown[] = await once(
      child,
      'close',
    );
    /**
     The status the process exited with, or nothing when a signal ended it.
     */
    const [status,] = closed;
    return ((typeof status) === 'number') ? { status, } : {};
  }
  catch (error) {
    // `once` rejects when the process could not be started at all, and that is the one failure a caller reports rather than reads
    // as an exit.
    return { startProblem: String(error,) };
  }
}
