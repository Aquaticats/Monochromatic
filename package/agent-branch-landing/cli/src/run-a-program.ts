/**
 The one seam that asks the machine to run a program: Git, `systemd-run`, and through it `mise`. A test replaces what is behind it
 with a stand-in on `PATH` rather than a fake in the code, so the words a landing really passes are the words a test reads.

 @module
 */

import { execFile, } from 'node:child_process';
import { promisify, } from 'node:util';

/**
 What one program printed and how it ended.
 */
export type ProgramOutput = {
  /**
   The status the program exited with, or minus one when it never ran to an exit.
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
   Why it never ran to an exit, when it never did: the program is not on `PATH`, or it ran past its bound and was killed.
   */
  readonly problem?: string;
};

/**
 One program to run: which one, with which words, in which folder, and for how long.
 */
export type RunProgramRequest = {
  /**
   The program, found on `PATH`.
   */
  readonly program: string;

  /**
   Its arguments, each kept whole, so no shell reads them.
   */
  readonly words: readonly string[];

  /**
   The folder it starts in.
   */
  readonly folder: string;

  /**
   Variables to add to this process's own environment, which is otherwise handed over whole.
   */
  readonly environment?: Readonly<Record<string, string>>;

  /**
   How long it may run before it is killed.
   */
  readonly timeoutMilliseconds: number;

  /**
   Most bytes of output kept. Past this the run is a failure naming the bound, never a silently shortened answer.
   */
  readonly outputBytes: number;
};

/**
 Runs one program and answers what it printed.
 */
export type RunProgram = (request: RunProgramRequest) => Promise<ProgramOutput>;

/**
 `execFile` with a promise result, which never goes through a shell. Typed through Node's own adapter contract, because the
 generic `promisify(execFile)` form trips `strict-void-return` (`doc/troubleshooting/oxlint-promisify-void-return.md`).
 */
const promisifyExecFile: (original: typeof execFile) => typeof execFile.__promisify__ = promisify;

/**
 Runs a program without a shell and waits for it to end.
 */
const execFileAsync: typeof execFile.__promisify__ = promisifyExecFile(execFile,);

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
 const complaint = textFieldOf(undefined,);
 ```
 */
function textFieldOf(value: unknown,): string {
  return ((typeof value) === 'string') ? value : '';
}

/**
 Reads the status a program exited with.

 @param value - the status the failure carries

 @returns the status, or minus one when the failure carries none

 @example
 ```ts
 const status = statusFieldOf(1,);
 ```
 */
function statusFieldOf(value: unknown,): number {
  return ((typeof value) === 'number') ? value : -1;
}

/**
 Reads what a failed program reported.

 @param caught - what the failed program threw

 @returns its status, its output, and why it never ran to an exit when it never did

 @example
 ```ts
 const failed = failureOf(new Error('x'),);
 ```
 */
function failureOf(caught: unknown,): ProgramOutput {
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
   The status the program exited with, which is not a number when it never ran to one.
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
 Runs one program in one folder, bounded in time and in the output kept.

 @param program - program

 @param words - the words Git is asked

 @param folder - the folder the program starts in

 @param environment - variables added to this process's own

 @param timeoutMilliseconds - how long it may take

 @param outputBytes - most bytes of output kept

 @returns what it printed and how it ended

 @example
 ```ts
 const merged = await runProgramWithChildProcess({
   program: 'git',
   words: ['merge', '--no-ff', '--no-commit', 'main',],
   folder: '/repository',
   timeoutMilliseconds: 1_000,
   outputBytes: 4_096,
 },);
 ```
 */
export async function runProgramWithChildProcess({
  program,
  words,
  folder,
  environment,
  timeoutMilliseconds,
  outputBytes,
}: RunProgramRequest,): Promise<ProgramOutput> {
  try {
    /**
     What the program printed.
     */
    const done = await execFileAsync(
      program,
      [...words,],
      {
        cwd: folder,
        env: {
          ...process.env,
          ...environment,
        },
        encoding: 'utf8',
        maxBuffer: outputBytes,
        timeout: timeoutMilliseconds,
        killSignal: 'SIGKILL',
      },
    );
    return {
      status: 0,
      stdout: done.stdout,
      stderr: done.stderr,
    };
  }
  catch (caught) {
    // A program that exits non-zero, that never started, or that ran past its own bound and was killed rejects here, and its
    // own output is what the caller reads.
    return failureOf(caught,);
  }
}
