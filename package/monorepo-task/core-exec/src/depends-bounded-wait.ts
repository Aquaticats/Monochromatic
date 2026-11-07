/**
 Time bounds for every child process task-depends waits for.

 Owner rule: "nothing should ever wait forever or indefinitely". A child that never exits (a Node process that could not
 start a thread under a full process-count ceiling sleeps in `futex_do_wait` for good, a stalled mount, a command waiting
 on input) would otherwise hold task-depends, and everything that waits for task-depends, open without end.

 @module
 */

import type { ChildProcess, } from 'node:child_process';

import spawn, {
  type Options,
  type Result,
} from 'nano-spawn';

import {
  MS_PER_HOUR,
  MS_PER_MINUTE,
  MS_PER_SECOND,
} from '@monochromatic-dev/const-core/ts';

//region Default bounds

/**
 How long one `sh:` source, output or time-strategy command may run before task-depends ends it, in milliseconds.

 These commands only print one timestamp: measured on 2026-11-07, `git log -1` took a few milliseconds and
 `podman image exists` 0.07 s, so one minute is nearly three orders above the commands the repository's tasks use.
 Raise it per call with `--query-timeout-seconds` for a slower probe.
 */
export const DEFAULT_QUERY_TIMEOUT_MS: number = MS_PER_MINUTE;

/**
 How long the command after `--` (the build or the task the checks guard) may run before task-depends ends it, in milliseconds.

 Provisional value awaiting the owner's pick: the longest task run in this repository's records is the 20 min 51 s toolchain
 rehearsal in `doc/handover/cli-git-rust-implementation.md`, so one hour keeps every recorded task whole while ending a
 stuck one. Raise or lower it per call with `--command-timeout-seconds`.
 */
export const DEFAULT_COMMAND_TIMEOUT_MS: number = MS_PER_HOUR;

//endregion Default bounds

//region Bounded spawn and failure wording

/**
 Raised when a child outlived its bound and was ended; the thrown spawn error stays in `cause`.

 @example
 ```ts
 throw new BoundedWaitTimeoutError({ timeoutMs: 1_000, cause: new Error('killed',), },);
 ```
 */
export class BoundedWaitTimeoutError extends Error {
  /**
   The bound that was in force, in milliseconds.
   */
  readonly timeoutMs: number;

  /**
   @param timeoutMs - Bound that was in force, in milliseconds

   @param cause - Spawn error raised after the child was ended
   */
  constructor({
    timeoutMs,
    cause,
  }: {
    readonly timeoutMs: number;
    readonly cause: unknown
  },) {
    super(
      `child timed out after ${String(timeoutMs / MS_PER_SECOND,)} seconds and was ended`,
      { cause, },
    );
    this.name = 'BoundedWaitTimeoutError';
    this.timeoutMs = timeoutMs;
  }
}

/**
 Ends a child and everything it started, by killing its process group with `SIGKILL`.

 The child leads its own group (it is spawned detached), so a shell and every member of its pipeline end together
 and no member keeps the output pipe open. Windows has no process groups to signal: only the child itself ends there.

 @param child - Child started by {@link spawnBounded}; its process id is also its group id

 @example
 ```ts
 endProcessGroup({ child, },);
 ```
 */
function endProcessGroup({ child, }: { readonly child: ChildProcess; },): void {
  if ((process.platform === 'win32') || (child.pid === undefined)) {
    child.kill('SIGKILL',);
    return;
  }
  try {
    process.kill(
      -child.pid,
      'SIGKILL',
    );
  }
  catch (error) {
    // ESRCH means the group already ended on its own, which is the wanted outcome.
    if (!((Error.isError(error,)) && ('code' in error)
      && (error.code === 'ESRCH')))
      console.error(`[task-depends] could not end the timed-out child (process group ${String(child.pid,)}): ${String(error,)}`,);
  }
}

/**
 Signals whose default action ends this process; each also ends the child's group first.
 */
const END_SIGNALS = [
  'SIGINT',
  'SIGTERM',
  'SIGHUP',
] as const;

/**
 Makes an interrupt of this process end the child's whole group too, then lets the interrupt take its default effect.

 The child's own group no longer receives the terminal's interrupt, so this process passes it on.

 @param child - Child started by {@link spawnBounded}

 @returns Removes the handlers again once the child is done

 @example
 ```ts
 const remove = endSignalsFor({ child, },);
 remove();
 ```
 */
function endSignalsFor({ child, }: { readonly child: ChildProcess; },): () => void {
  /**
   Handlers by signal, kept so each can be removed again.
   */
  const handlers = END_SIGNALS.map(function handlerFor(signal,) {
    /**
     Ends the group, removes this handler, and re-raises the signal so the default action ends this process.
     */
    function endGroupThenSelf(): void {
      endProcessGroup({ child, },);
      process.removeListener(
        signal,
        endGroupThenSelf,
      );
      process.kill(
        process.pid,
        signal,
      );
    }
    process.on(
      signal,
      endGroupThenSelf,
    );
    return {
      signal,
      handler: endGroupThenSelf,
    };
  },);
  return function removeHandlers(): void {
    for (const {
      signal,
      handler,
    } of handlers)
      process.removeListener(
        signal,
        handler,
      );
  };
}

/**
 What guards one running child: the bound's timer and the signal handlers.
 */
type ChildGuard = Disposable & {
  /**
   Whether the bound has ended the child.
   */
  readonly wasEnded: () => boolean;
};

/**
 Starts the bound's timer and the signal handlers for a running child.

 @param child - Running child

 @param timeoutMs - Bound in milliseconds

 @returns The guard; disposing it stops the timer and removes the signal handlers

 @example
 ```ts
 using guard = guardChild({ child, timeoutMs: 1_000, },);
 ```
 */
function guardChild({
  child,
  timeoutMs,
}: {
  readonly child: ChildProcess;
  readonly timeoutMs: number
},): ChildGuard {
  /**
   Set by the timer, so a spawn error caused by the kill is told from an ordinary failure.
   */
  const state = { timedOut: false, };
  /**
   Timer that ends the child's group when the bound passes.
   */
  const timer = setTimeout(
    function endChildAfterBound() {
      state.timedOut = true;
      endProcessGroup({ child, },);
    },
    timeoutMs,
  );
  /**
   Removes the signal handlers.
   */
  const removeSignalHandlers = endSignalsFor({ child, },);
  return {
    wasEnded() {
      return state.timedOut;
    },
    [Symbol.dispose]() {
      clearTimeout(timer,);
      removeSignalHandlers();
    },
  };
}

/**
 Runs a child with `nano-spawn` and ends it with its whole process group when it outlives the bound.

 `SIGKILL` guarantees the end: a child stuck before it installs any handler, or one that ignores `SIGTERM`, would
 otherwise keep the wait alive.

 @param file - Executable, or the shell command line when `options.shell` is set

 @param args - Arguments passed to the executable

 @param options - Other `nano-spawn` options; `detached` is set here

 @param timeoutMs - Bound in milliseconds

 @returns Captured result when the child finished inside the bound

 @throws {@link BoundedWaitTimeoutError} when the bound ended the child; the spawn error of an ordinary failure passes through

 @example
 ```ts
 await spawnBounded({ file: 'sleep 5 | cat', args: [], options: { shell: true, }, timeoutMs: 1_000, },); // rejects after about one second
 ```
 */
export async function spawnBounded(
  {
    file,
    args,
    options,
    timeoutMs,
  }: {
    readonly file: string;
    readonly args: readonly string[];
    readonly options: Options;
    readonly timeoutMs: number;
  },
): Promise<Result> {
  /**
   The running child; `detached` makes it the leader of a new process group that {@link endProcessGroup} can signal.
   */
  const subprocess = spawn(
    file,
    [...args,],
    {
      ...options,
      detached: process.platform !== 'win32',
    },
  );
  /**
   Outcome of starting the child; settled, not thrown, because a failed start is reported by awaiting `subprocess` below.
   */
  const [started,] = await Promise.allSettled([subprocess.nodeChildProcess,],);
  if (started.status === 'rejected')
    return await subprocess;
  /**
   Timer and signal handlers for the running child.
   */
  using guard = guardChild({
    child: started.value,
    timeoutMs,
  },);
  try {
    return await subprocess;
  }
  catch (error) {
    if (guard.wasEnded())
      throw new BoundedWaitTimeoutError({
        timeoutMs,
        cause: error,
      },);
    throw error;
  }
}

/**
 Says whether a thrown error is the bound ending the child, as one sentence for the failure message.

 @param error - Value thrown by {@link spawnBounded}

 @returns A sentence naming the bound, or an empty string when the child failed for another reason

 @example
 ```ts
 timeoutNote(new Error('x',),); // ''
 ```
 */
export function timeoutNote(error: unknown,): string {
  if (!(error instanceof BoundedWaitTimeoutError))
    return '';
  return `timed out after ${String(error.timeoutMs / MS_PER_SECOND,)} seconds and was ended`;
}

//endregion Bounded spawn and failure wording
