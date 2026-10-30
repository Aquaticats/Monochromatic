import { spawn } from 'node:child_process';
import { once } from 'node:events';

import { CargoMutantsToolingError } from './error.ts';

/**
 What a Podman call left behind.
 */
export type PodmanResult = {
  readonly status: number;
  readonly signal: string;
  readonly stdout: string;
  readonly stderr: string;
};

/**
 Signal text reported when the call ended with an exit status and no signal.
 */
const noSignal = '';

/**
 Run Podman without a shell and wait for it.

 @param args - complete Podman arguments, each passed as one argument
 
 @param capture - true to collect standard output and error, false to pass them through to the caller's streams
 
 @param allowFailure - true when a nonzero exit is a result to inspect, false when it must stop the campaign
 
 @returns exit status, the signal that ended the call (empty when none), and the captured output (empty when not captured)
 
 @throws CargoMutantsToolingError when the call cannot start, or exits nonzero without `allowFailure`
 
 @example
 ```ts
 await runPodman({ args: ['--version'], capture: true, allowFailure: false });
 ```
 */
export async function runPodman({
  args,
  capture,
  allowFailure,
}: {
  readonly args: readonly string[];
  readonly capture: boolean;
  readonly allowFailure: boolean;
}): Promise<PodmanResult> {
  /**
   The running call.
   */
  const child = spawn(
    'podman',
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
  /**
   Exit status, with a signal counted as a failure.
   */
  const status = (typeof code) === 'number' ? code : 1;
  /**
   Captured standard error.
   */
  const stderr = Buffer.concat(errorChunks)
    .toString('utf8');
  if ((!allowFailure) && (status !== 0))
    throw new CargoMutantsToolingError(`Container command failed: ${args[0]} (status ${status}, signal ${(typeof signal) === 'string' ? signal : 'none'}). ${stderr}`);
  return {
    status,
    signal: (typeof signal) === 'string' ? signal : noSignal,
    stdout: Buffer.concat(outChunks)
      .toString('utf8'),
    stderr,
  };
}
