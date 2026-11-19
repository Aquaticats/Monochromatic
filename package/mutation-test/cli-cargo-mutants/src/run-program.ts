import { spawn } from 'node:child_process';
import { once } from 'node:events';

import { CargoMutantsToolingError } from './error.ts';

/**
 What a finished program left behind.
 */
export type ProgramResult = {
  readonly status: number;
  readonly stderr: string;
};

/**
 Exit status reported for a program that ended without one, because a signal stopped it.
 */
export const signalExitStatus = 1;

/**
 Run a program without a shell, optionally bounding the wait for exit and stderr.

 @param program - executable name or path
 
 @param args - arguments, each passed as one argument
 
 @param inheritStdio - true to pass the program the caller's standard streams, false to capture its standard error
 
 @param deadlineMilliseconds - stall limit, absent for programs the caller wants to wait for until exit

 @returns the exit status, and the captured standard error text (empty when inherited)
 
 @throws Error when the program cannot be started, for example when it is not installed

 @throws CargoMutantsToolingError when the deadline passes without observing the program and its stderr close
 
 @example
 ```ts
 const { status } = await runProgram({ program: 'true', args: [], inheritStdio: false });
 ```
 */
export async function runProgram({
  program,
  args,
  inheritStdio,
  deadlineMilliseconds,
}: {
  readonly program: string;
  readonly args: readonly string[];
  readonly inheritStdio: boolean;
  readonly deadlineMilliseconds?: number;
}): Promise<ProgramResult> {
  /**
   Cancel the wait itself, not just the child: a kernel wait can keep a killed process and its stderr open.
   */
  const deadline = deadlineMilliseconds === undefined ? undefined : AbortSignal.timeout(deadlineMilliseconds);
  /**
   The running child.
   */
  const child = spawn(
    program,
    [...args],
    { stdio: inheritStdio ? 'inherit' : [
      'ignore',
      'ignore',
      'pipe'
    ] },
  );
  /**
   Chunks of standard error as they arrive.
   */
  const chunks: Buffer[] = [];
  child.stderr
    ?.on(
    'data',
    function collect(chunk: Buffer) {
      chunks.push(chunk);
    },
    );
  try {
    /**
     Arguments of the close event: the exit code, then the signal, once the streams have closed.
     */
    const closed: readonly unknown[] = await once(
      child,
      'close',
      deadline === undefined ? {} : { signal: deadline },
    );
    /**
     Exit code, which is not a number when a signal ended the child.
     */
    const [code] = closed;
    return {
      status: (typeof code) === 'number' ? code : signalExitStatus,
      stderr: Buffer.concat(chunks)
        .toString('utf8'),
    };
  }
  catch (error) {
    if (deadline?.aborted !== true)
      throw error;
    /**
     Stop the wait without relying on the direct child to acknowledge SIGKILL.
     */
    using release = {
      [Symbol.dispose](): void {
        child
          .stderr
          ?.destroy();
        child.unref();
      },
    };
    /**
     Diagnostic for a timeout, including one where the kernel does not immediately end the child.
     */
    const reason = `${program} was still running after ${String(deadlineMilliseconds)} ms`;
    try {
      // GNU sync needs no subprocesses; this runner stops the direct child, not a process group.
      child.kill('SIGKILL');
    }
    catch (killError) {
      throw new CargoMutantsToolingError(
        `${reason}, and sending SIGKILL failed: ${String(killError)}`,
        { cause: killError },
      );
    }
    throw new CargoMutantsToolingError(
      `${reason}; stopped waiting and requested SIGKILL.`,
      { cause: error },
    );
  }
}
