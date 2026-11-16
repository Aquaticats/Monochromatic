import { spawn as spawnChild, } from 'node:child_process';
import { createInterface, } from 'node:readline';
import { text, } from 'node:stream/consumers';

import { wait, } from '@monochromatic-dev/async-time-core/ts';
import { isErrnoException, } from './errors.ts';
import { tagged, } from '@monochromatic-dev/logger-core/ts';

/**
 Module logger for the operation lock.
 */
const l = tagged({ tag: 'operation-lock', },);

/**
 Creates domain-specific error for operation-lock failures.
 */
export type OperationLockErrorFactory = (message: string) => Error;

/**
 Exit status distinguishing lock contention from command failure.
 */
const LOCK_CONFLICT_EXIT_CODE = 75;

/**
 Readiness line emitted only after `flock` acquires operation lock.
 */
const LOCK_READY_LINE = 'wg-quicker-operation-lock-ready';

/**
 Interval at which holder verifies owning process still exists.
 */
const LOCK_PARENT_PROBE_MS = 100;

/**
 Milliseconds the claim waits for the holder to write its readiness line.

 The holder is `flock` and a short Node program, so its readiness is its own start:
 measured on this host over ten rounds it answered in 16 to 18 ms,
 with one round at 309 ms while the machine was busy.
 Ten seconds is thirty-two times that slowest round,
 and it is a stall detector rather than a target:
 a holder that needs longer has not started, and the lifecycle that waits for it must fail and roll back.
 */
export const OPERATION_LOCK_READY_DEADLINE_MILLISECONDS = 10_000;

/**
 Milliseconds a release waits for the holder to end after `SIGTERM`,
 and again after `SIGKILL`.

 Measured on this host over ten rounds, the holder ended 0 to 1 ms after the polite signal.
 Two seconds is the point where it has not answered and must be killed,
 and the second wait is the point where even a kill has not been answered by the kernel.
 */
export const OPERATION_LOCK_RELEASE_DEADLINE_MILLISECONDS = 2_000;

/**
 Milliseconds between checks of whether the holder ended while the lock is released.
 */
const RELEASE_POLL_MILLISECONDS = 10;

/**
 JavaScript run after `flock --no-fork` acquires lock.
 
 Holder exits through uncaught error when parent disappears,
 releasing kernel lock after caller crash.
 */
const LOCK_HOLDER_SOURCE = `const ownerPid = ${String(process.pid,)}; process.stdout.write('${LOCK_READY_LINE}\\n'); setInterval(function verifyOwner() { if (process.ppid !== ownerPid) throw new Error('wg-quicker operation-lock owner disappeared'); }, ${String(LOCK_PARENT_PROBE_MS,)});`;

/**
 Acquires crash-safe kernel lock through readiness-signaled holder.
 
 @param lockPath - Stable advisory lock path.
 
 @param conflictMessage - Diagnostic when another holder owns lock.
 
 @param errorFactory - Domain error constructor for lock failures.
 
 @returns Guard terminating holder on disposal.
 
 @throws Error returned by `errorFactory` on conflict or holder failure.
 
 @example
 ```ts
 await using lock = await claimOperationLock({
   lockPath: '/run/x.lock',
   conflictMessage: 'busy',
   errorFactory: message => new Error(message),
 });
 ```
 */
export async function claimOperationLock(
  {
    lockPath,
    conflictMessage,
    errorFactory,
    deadlineMilliseconds = OPERATION_LOCK_READY_DEADLINE_MILLISECONDS,
    releaseDeadlineMilliseconds = OPERATION_LOCK_RELEASE_DEADLINE_MILLISECONDS,
  }: {
    readonly lockPath: string;
    readonly conflictMessage: string;
    readonly errorFactory: OperationLockErrorFactory;
    readonly deadlineMilliseconds?: number;
    readonly releaseDeadlineMilliseconds?: number;
  },
): Promise<AsyncDisposable> {
  /**
   Primitive path snapshot passed through unresolved spawn boundary.
   */
  const pathUnits: string[] = [];
  /**
   UTF-16 cursor preserving path exactly without sharing caller container.
   */
  const pathCursor = { index: 0, };
  while (pathCursor.index < lockPath.length) {
    pathUnits[pathCursor.index] = lockPath.charAt(pathCursor.index,);
    pathCursor.index += 1;
  }
  /**
   Isolated path reconstructed from primitive code units.
   */
  const commandLockPath = pathUnits.join('',);
  /**
   Lock process becoming Node after kernel lock acquisition.
   */
  const holder = spawnChild(
    'flock',
    [
      '--exclusive',
      '--nonblock',
      '--conflict-exit-code',
      String(LOCK_CONFLICT_EXIT_CODE,),
      '--no-fork',
      commandLockPath,
      process.execPath,
      '--input-type=module',
      '--eval',
      LOCK_HOLDER_SOURCE,
    ],
    {
      stdio: [
        'ignore',
        'pipe',
        'pipe',
      ],
    },
  );
  /**
   Spawn failures captured without unhandled process event.
   */
  const spawnFailureMessages: string[] = [];
  holder.once(
    'error',
    function captureSpawnFailure(error: Readonly<Error>,): void {
      spawnFailureMessages[0] = error.message;
    },
  );
  /**
   Reads the holder's diagnostics,
   or answers a sentence saying they could not be read,
   so a failed read never becomes an unhandled rejection.

   @returns The holder's standard error, or the sentence saying it could not be read.
   */
  async function readHolderDiagnostics(): Promise<string> {
    try {
      return await text(holder.stderr,);
    }
    catch (error: unknown) {
      return `the holder's standard error could not be read: ${String(error,)}`;
    }
  }
  /**
   Diagnostics of the holder, read while the readiness line is awaited.
   */
  const stderr = readHolderDiagnostics();
  /**
   Sends a signal to the holder, tolerating one that has already ended.
   */
  function signalHolder({ signal, }: { readonly signal: NodeJS.Signals; },): void {
    try {
      holder.kill(signal,);
    }
    catch (error: unknown) {
      // A holder that is already gone needs no signal.
      if ((!isErrnoException(error,)) || (error.code !== 'ESRCH'))
        throw error;
    }
  }
  /**
   Waits, bounded, for the holder to end.

   @param milliseconds - Longest time to wait.

   @returns Whether the holder ended within that time.
   */
  async function holderEndsWithin({ milliseconds, }: { readonly milliseconds: number; },): Promise<boolean> {
    /**
     When the wait gives up.
     */
    const deadline = performance.now() + milliseconds;
    while (performance.now() < deadline) {
      if ((holder.exitCode !== null) || (holder.signalCode !== null))
        return true;
      // oxlint-disable-next-line eslint/no-await-in-loop -- The holder's end is polled within one bound.
      await wait(RELEASE_POLL_MILLISECONDS,);
    }
    return (holder.exitCode !== null) || (holder.signalCode !== null);
  }
  /**
   Readiness lines from acquired lock holder.
   */
  const lines = createInterface({ input: holder.stdout, },);
  /**
   Whether the readiness deadline passed,
   which is what ends a wait that nothing else ends.
   */
  const waited = { pastDeadline: false, };
  /**
   The readiness deadline: a holder that has not answered by then has not started,
   and the lifecycle waiting for it fails instead of waiting for ever.
   */
  const deadline = setTimeout(
    function giveUp(): void {
      waited.pastDeadline = true;
      signalHolder({ signal: 'SIGKILL', },);
      lines.close();
    },
    deadlineMilliseconds,
  );
  /**
   The refusal for a holder that passed its readiness bound.

   @returns The refusal to report.
   */
  function deadlineRefusal(): Error {
    return errorFactory(
      `The operation-lock holder did not write its readiness line within ${String(deadlineMilliseconds,)} ms for ${lockPath}, so it was killed and the lifecycle did not wait for it.`,
    );
  }
  for await (const line of lines) {
    if (line !== LOCK_READY_LINE) {
      signalHolder({ signal: 'SIGKILL', },);
      throw errorFactory(`Unexpected operation-lock readiness output: ${line}`,);
    }
    // A line that arrives after the bound belongs to a holder that was already killed,
    // and a lock nobody holds is not a lock this call may report.
    if (waited.pastDeadline)
      throw deadlineRefusal();
    clearTimeout(deadline,);
    lines.close();
    return {
      /**
       Terminates holder so kernel releases lock.
       */
      async [Symbol.asyncDispose](): Promise<void> {
        if ((holder.exitCode !== null) || (holder.signalCode !== null))
          return;
        signalHolder({ signal: 'SIGTERM', },);
        if (await holderEndsWithin({ milliseconds: releaseDeadlineMilliseconds, },))
          return;
        signalHolder({ signal: 'SIGKILL', },);
        if (await holderEndsWithin({ milliseconds: releaseDeadlineMilliseconds, },))
          return;
        l.error(
          `operation-lock holder PID ${String(holder.pid,)} did not end within ${String(releaseDeadlineMilliseconds,)} ms after SIGKILL; it may still hold ${lockPath}`,
        );
      },
    };
  }
  clearTimeout(deadline,);
  if (waited.pastDeadline) {
    // The holder's own diagnostic is not waited for here: the wait has already passed its bound.
    throw deadlineRefusal();
  }
  /**
   Diagnostic after holder exits without readiness.
   */
  const diagnostic = await stderr;
  /**
   First spawn failure when child could not start.
   */
  const [spawnFailureMessage,] = spawnFailureMessages;
  if (spawnFailureMessage !== undefined)
    throw errorFactory(`Cannot start operation-lock holder: ${spawnFailureMessage}`,);
  if (holder.exitCode === LOCK_CONFLICT_EXIT_CODE)
    throw errorFactory(conflictMessage,);
  throw errorFactory(
    `Operation-lock holder exited ${String(holder.exitCode,)} before readiness: ${diagnostic}`,
  );
}
