import { CargoMutantsToolingError } from './error.ts';
import {
  type CapturedResult,
  runCaptured,
} from './run-captured.ts';

/**
 What a Podman call left behind.
 */
export type PodmanResult = CapturedResult;

/**
 Run Podman without a shell and wait for it.

 @param args - complete Podman arguments, each passed as one argument
 
 @param capture - true to collect standard output and error, false to pass them through to the caller's streams
 
 @param allowFailure - true when a nonzero exit is a result to inspect, false when it must stop the campaign
 
 @returns exit status, the signal that ended the call (empty when none), and the captured output (empty when not captured)
 
 @throws CargoMutantsToolingError when the call exits nonzero without `allowFailure`
 
 @throws Error when Podman cannot be started
 
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
   What the call left behind.
   */
  const result = await runCaptured({
    program: 'podman',
    args,
    capture,
  });
  if ((!allowFailure) && (result.status !== 0))
    throw new CargoMutantsToolingError(`Container command failed: ${args[0]} (status ${result.status}, signal ${result.signal === '' ? 'none' : result.signal}). ${result.stderr}`);
  return result;
}
