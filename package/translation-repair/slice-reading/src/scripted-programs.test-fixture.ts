import {
  readFile,
  writeFile,
} from 'node:fs/promises';
import { constants, } from 'node:os';

import { nonNullishOrThrow, } from '@monochromatic-dev/or-throw-core/ts';

import type { ProgramRunner, } from '../dist/final/node/index.mjs';

//region Scripted programs
// A PROGRAM RUNNER THAT STARTS NO PROCESS, for cases about code that reaches
// for command-line tools. Every run the code under test asks for is recorded,
// and what each program does is a stand-in the case names: a decoder that
// copies its input, a reader that transcribes its input, a program that is
// not installed, a program that exits unhappy.
//
// TEST SUPPORT, NOT PACKAGE SOURCE.
//
// THE STAND-INS DO REAL FILE WORK, at the paths the code under test hands
// them, which are paths inside the scratch directory that code made for
// itself. So a case reads the wiring off the result: the words a reading
// carries are the bytes the case handed in only when the picture was written
// to the path the decoder was given, decoded to the path the reader was
// given, and read back from the stem the reader was given.
//
// THE TWO REJECTIONS COPY NODE'S OWN, field for field, as `promisify(execFile)`
// raised them on Node 26.10.0 when this was written: a plain `Error` with the
// spawn's fields written onto it, the way Node builds them, and no class of
// its own, since code that renders a caught value by its name prints `Error`
// for the real ones. Nothing here is trusted from that one measurement: the
// `runInstalledProgram` cases in `image-ocr.unit.test.ts` hold each one
// against the rejection the real runner raises, every time the file runs.

/**
 One run the code under test asked a runner for.

 @example
 ```ts
 const run: ProgramRun = { program: 'purr', args: ['--loud',], };
 ```
 */
export type ProgramRun = {
  /**
   Program as the code under test named it.
   */
  readonly program: string;

  /**
   Arguments in the order they were given.
   */
  readonly args: readonly string[];
};

/**
 The command line Node prints in a spawn failure: the program and its
 arguments joined by single spaces, unquoted.

 @param program - program as it was named

 @param args - arguments as they were given

 @returns Joined line

 @example
 ```ts
 const line = commandLine({ program: 'purr', args: ['--loud',], },); // 'purr --loud'
 ```
 */
function commandLine(
  {
    program,
    args,
  }: ProgramRun,
): string {
  return [
    program,
    ...args,
  ].join(' ',);
}

/**
 The rejection Node raises when the program to spawn is not installed: its
 message, and the system's code (`ENOENT`, a string) beside the call, the
 program, its arguments and the empty output of a program that never ran.

 @param run - program that is not installed, and its arguments

 @returns Plain `Error` carrying Node's spawn-failure fields

 @example
 ```ts
 throw missingProgramFailure({ program: 'purr', args: ['--loud',], },);
 ```
 */
export function missingProgramFailure(run: ProgramRun,): Error {
  return Object.assign(
    new Error(`spawn ${run.program} ENOENT`,),
    {
      errno: -constants.errno
        .ENOENT,
      code: 'ENOENT',
      syscall: `spawn ${run.program}`,
      path: run.program,
      spawnargs: [...run.args,],
      cmd: commandLine(run,),
      stdout: '',
      stderr: '',
    },
  );
}

/**
 The rejection Node raises when a program ran and exited with a code other
 than zero: its message, the exit code (a number, where a missing program
 carries a string), and what the program printed.

 `signal`, WHICH NODE SETS TO `null` ON SUCH AN EXIT, IS LEFT OFF, and the
 cases comparing this with the real rejection leave that one field out by
 name.

 @param exit - program that ran, its arguments, the code it exited with and
 what it wrote to standard error

 @returns Plain `Error` carrying Node's exit-failure fields

 @example
 ```ts
 throw programExitFailure({ program: 'purr', args: [], exitCode: 1, stderr: 'hiss\n', },);
 ```
 */
export function programExitFailure(
  exit: ProgramRun & {
    readonly exitCode: number;
    readonly stderr: string;
  },
): Error {
  return Object.assign(
    new Error(`Command failed: ${commandLine(exit,)}\n${exit.stderr}`,),
    {
      code: exit.exitCode,
      killed: false,
      cmd: commandLine(exit,),
      stdout: '',
      stderr: exit.stderr,
    },
  );
}

/**
 Stand-in for a decoder that works: copies the file at its first argument
 to its last argument, which is where both `dwebp source -o target` and
 `magick source target` name their input and their output.

 @param args - decoder's arguments, input first and output last

 @throws Whatever reading the input or writing the output raises, which the
 code under test meets as a decoder that failed

 @example
 ```ts
 const programs = new Map([['dwebp', copyingDecoder,],],);
 ```
 */
export async function copyingDecoder({ args, }: ProgramRun,): Promise<void> {
  await writeFile(
    nonNullishOrThrow(args.at(-1,),),
    await readFile(nonNullishOrThrow(args[0],),),
  );
}

/**
 Stand-in for an OCR reader that works: writes the text of the file at its
 first argument to its second argument plus `.txt`, as `tesseract picture
 stem` names its input and its output. With {@link copyingDecoder} before
 it, the words a case hands in as a picture's bytes are the words read back.

 @param args - reader's arguments, picture first and output stem second

 @throws Whatever reading the picture or writing the text raises, which the
 code under test meets as a reader that failed

 @example
 ```ts
 const programs = new Map([['tesseract', transcribingReader,],],);
 ```
 */
export async function transcribingReader({ args, }: ProgramRun,): Promise<void> {
  await writeFile(
    `${nonNullishOrThrow(args[1],)}.txt`,
    await readFile(nonNullishOrThrow(args[0],),),
  );
}

/**
 Stand-in for a program that exits zero and leaves nothing behind.

 @example
 ```ts
 const programs = new Map([['tesseract', silentProgram,],],);
 ```
 */
export async function silentProgram(): Promise<void> {
  // Exits clean, having written no file.
}

/**
 Stand-in for a program that is not installed: rejects with
 {@link missingProgramFailure}'s error, as a spawn of a program that is not
 there does.

 @param run - program the code under test named, and its arguments

 @example
 ```ts
 const programs = new Map([['tesseract', missingProgram,],],);
 ```
 */
export function missingProgram(run: ProgramRun,): Promise<void> {
  return Promise.reject(missingProgramFailure(run,),);
}

/**
 Stand-in for a program that runs and exits unhappy.

 @param exitCode - code it exits with

 @param stderr - what it writes to standard error before it does

 @returns Stand-in, which rejects with {@link programExitFailure}'s error

 @example
 ```ts
 const programs = new Map([['dwebp', failingProgram({ exitCode: 1, stderr: 'not a picture\n', },),],],);
 ```
 */
export function failingProgram(
  {
    exitCode,
    stderr,
  }: {
    readonly exitCode: number;
    readonly stderr: string;
  },
): ProgramRunner {
  return function fail(run,): Promise<void> {
    return Promise.reject(programExitFailure({
      program: run.program,
      args: run.args,
      exitCode,
      stderr,
    },),);
  };
}

/**
 Stand-in for a decoder that starts writing its output and then gives up:
 leaves `garbage` at its last argument, the output path, and exits unhappy.

 @param garbage - bytes it leaves where the decoded picture should be

 @returns Stand-in, which rejects with {@link programExitFailure}'s error
 once the garbage is written

 @example
 ```ts
 const programs = new Map([['dwebp', damagingDecoder({ garbage, },),],],);
 ```
 */
export function damagingDecoder({ garbage, }: { readonly garbage: Uint8Array; },): ProgramRunner {
  return async function damageThenFail({
    program,
    args,
  },): Promise<void> {
    await writeFile(
      nonNullishOrThrow(args.at(-1,),),
      garbage,
    );
    throw programExitFailure({
      program,
      args,
      exitCode: 1,
      stderr: 'gave up partway through the picture\n',
    },);
  };
}

/**
 Builds a runner that starts no process: each run is recorded, then handed
 to the stand-in scripted for its program.

 @param programs - stand-in per program name; a program with none is
 recorded and then rejected, so a run no case scripted shows in `runs`
 whatever the code under test makes of the rejection

 @returns Runner, and every run it was asked for, in order

 @example
 ```ts
 const { runProgram, runs, } = scriptedPrograms({ programs: new Map([['dwebp', copyingDecoder,],],), },);
 ```
 */
export function scriptedPrograms(
  { programs, }: { readonly programs: ReadonlyMap<string, ProgramRunner>; },
): {
  readonly runProgram: ProgramRunner;
  readonly runs: readonly ProgramRun[];
} {
  /**
   Every run asked for so far, in order.
   */
  const runs: ProgramRun[] = [];

  return {
    runs,
    runProgram: async function runScripted(run,): Promise<void> {
      runs.push({
        program: run.program,
        args: [...run.args,],
      },);

      /**
       What the case scripted for this program, if anything.
       */
      const standIn = programs.get(run.program,);
      if (standIn === undefined)
        throw new Error(`no stand-in was scripted for ${run.program}`,);
      await standIn(run,);
    },
  };
}

//endregion Scripted programs
