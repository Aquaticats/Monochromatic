import { spawn as spawnChild, } from 'node:child_process';
import { once, } from 'node:events';
import { createInterface, } from 'node:readline';
import { text, } from 'node:stream/consumers';

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
  }: {
    readonly lockPath: string;
    readonly conflictMessage: string;
    readonly errorFactory: OperationLockErrorFactory;
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
   Diagnostics consumed while readiness line is awaited.
   */
  const stderr = text(holder.stderr,);
  /**
   Readiness lines from acquired lock holder.
   */
  const lines = createInterface({ input: holder.stdout, },);
  for await (const line of lines) {
    if (line !== LOCK_READY_LINE) {
      holder.kill('SIGKILL',);
      throw errorFactory(`Unexpected operation-lock readiness output: ${line}`,);
    }
    lines.close();
    return {
      /**
       Terminates holder so kernel releases lock.
       */
      async [Symbol.asyncDispose](): Promise<void> {
        if (holder.exitCode !== null)
          return;
        /**
         Close observation registered before termination signal.
         */
        const closed = once(
          holder,
          'close',
        );
        holder.kill('SIGTERM',);
        await closed;
      },
    };
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
