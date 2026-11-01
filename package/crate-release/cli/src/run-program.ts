import { spawn } from 'node:child_process';
import { once } from 'node:events';

import { caughtValueText } from '@monochromatic-dev/caught-value-core/ts';

import { failureExitCode } from './constants.ts';
import { CrateReleaseError } from './error.ts';

/**
 One program call: what to start, with which arguments, and where.
 */
export type ProgramCall = {
  readonly program: string;
  /**
   Arguments, each passed as one argument; no shell reads them.
   */
  readonly args: readonly string[];
  /**
   Absolute working directory.
   */
  readonly cwd: string;
};

/**
 What a finished program left behind.
 */
export type ProgramResult = {
  /**
   Exit status; the failure status when a signal ended the program.
   */
  readonly status: number;
  readonly stdout: string;
  readonly stderr: string;
};

/**
 Run a program to completion without a shell and report how it ended without judging it.

 @param program - executable name or path

 @param args - arguments, each passed as one argument

 @param cwd - absolute working directory

 @param capture - true to collect standard output and error, false to pass the caller's streams through

 @returns the exit status and the captured text (empty when not captured)

 @throws CrateReleaseError when the program cannot be started, for example because it is not installed

 @example
 ```ts
 await spawnProgram({ program: 'cargo', args: ['--version'], cwd: '/repo', capture: true });
 ```
 */
export async function spawnProgram({
  program,
  args,
  cwd,
  capture,
}: ProgramCall & {
  readonly capture: boolean;
}): Promise<ProgramResult> {
  /**
   The running child.
   */
  const child = spawn(
    program,
    [...args],
    {
      cwd,
      stdio: capture ? [
        'ignore',
        'pipe',
        'pipe',
      ] : 'inherit',
    },
  );
  /**
   Standard output chunks as they arrive.
   */
  const outChunks: Buffer[] = [];
  /**
   Standard error chunks as they arrive.
   */
  const errorChunks: Buffer[] = [];
  child.stdout
    ?.on(
    'data',
    function collectOut(chunk: Buffer) {
      outChunks.push(chunk);
    },
    );
  child.stderr
    ?.on(
    'data',
    function collectError(chunk: Buffer) {
      errorChunks.push(chunk);
    },
    );
  try {
    /**
     Arguments of the close event: the exit code, then the signal, once the streams have closed.
     */
    const closed: readonly unknown[] = await once(
      child,
      'close',
    );
    /**
     Exit code, which is not a number when a signal ended the child.
     */
    const [code] = closed;
    return {
      status: (typeof code) === 'number' ? code : failureExitCode,
      stdout: Buffer.concat(outChunks)
        .toString('utf8'),
      stderr: Buffer.concat(errorChunks)
        .toString('utf8'),
    };
  }
  catch (error) {
    throw new CrateReleaseError(
      `Could not run ${program} in ${cwd}: ${caughtValueText(error)}. Is ${program} installed and on the search path?`,
      { cause: error },
    );
  }
}
