import { spawn } from 'node:child_process';
import { once } from 'node:events';

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
 Run a program to completion without a shell.

 @param program - executable name or path
 
 @param args - arguments, each passed as one argument
 
 @param inheritStdio - true to pass the program the caller's standard streams, false to capture its standard error
 
 @returns the exit status, and the captured standard error text (empty when inherited)
 
 @throws Error when the program cannot be started, for example when it is not installed
 
 @example
 ```ts
 const { status } = await runProgram({ program: 'true', args: [], inheritStdio: false });
 ```
 */
export async function runProgram({
  program,
  args,
  inheritStdio,
}: {
  readonly program: string;
  readonly args: readonly string[];
  readonly inheritStdio: boolean;
}): Promise<ProgramResult> {
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
    status: (typeof code) === 'number' ? code : signalExitStatus,
    stderr: Buffer.concat(chunks)
      .toString('utf8'),
  };
}
