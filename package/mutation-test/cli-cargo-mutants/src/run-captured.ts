import { spawn } from 'node:child_process';
import { once } from 'node:events';

/**
 What a finished call left behind.
 */
export type CapturedResult = {
  readonly status: number;
  /**
   Signal that ended the call, or the empty text when it exited with a status.
   */
  readonly signal: string;
  readonly stdout: string;
  readonly stderr: string;
};

/**
 Exit status reported for a call that a signal ended.
 */
const signalStatus = 1;

/**
 Run a program without a shell, wait for it, and report how it ended without judging it.

 @param program - executable name or path
 
 @param args - complete arguments, each passed as one argument
 
 @param capture - true to collect standard output and error, false to pass them through to the caller's streams
 
 @returns exit status, the signal that ended the call (empty when none), and the captured output (empty when not captured)
 
 @throws Error when the program cannot be started, for example when it is not installed
 
 @example
 ```ts
 await runCaptured({ program: 'cargo', args: ['--version'], capture: true });
 ```
 */
export async function runCaptured({
  program,
  args,
  capture,
}: {
  readonly program: string;
  readonly args: readonly string[];
  readonly capture: boolean;
}): Promise<CapturedResult> {
  /**
   The running call.
   */
  const child = spawn(
    program,
    [...args],
    { stdio: capture ? [
      'ignore',
      'pipe',
      'pipe'
    ] : 'inherit' },
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
  /**
   Arguments of the close event: the exit code, then the signal.
   */
  const closed: readonly unknown[] = await once(
    child,
    'close',
  );
  /**
   Exit code and signal of the call.
   */
  const [code, signal] = closed;
  return {
    status: (typeof code) === 'number' ? code : signalStatus,
    signal: (typeof signal) === 'string' ? signal : '',
    stdout: Buffer.concat(outChunks)
      .toString('utf8'),
    stderr: Buffer.concat(errorChunks)
      .toString('utf8'),
  };
}
