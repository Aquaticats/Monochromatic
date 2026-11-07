/**
 Time bounds for every child process task-depends waits for.

 Owner rule: "nothing should ever wait forever or indefinitely". A child that never exits (a Node process that could not
 start a thread under a full process-count ceiling sleeps in `futex_do_wait` for good, a stalled mount, a command waiting
 on input) would otherwise hold task-depends, and everything that waits for task-depends, open without end.

 @module
 */

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

//region Spawn options and failure wording

/**
 Options that make a child end after the bound, as a spread for `nano-spawn` options.

 `SIGKILL` guarantees the end: a child stuck before it installs any handler, or one that ignores `SIGTERM`, would
 otherwise keep the wait alive. `isCanceled` on the thrown error then tells the bound from an ordinary failure.

 @param timeoutMs - Bound in milliseconds

 @returns Options carrying an abort signal that fires after the bound, and the signal that ends the child

 @example
 ```ts
 await spawn('sleep', ['5',], { ...boundedSpawnOptions({ timeoutMs: 1_000, },), },); // rejects after about one second
 ```
 */
export function boundedSpawnOptions(
  { timeoutMs, }: { readonly timeoutMs: number; },
): {
  readonly signal: AbortSignal;
  readonly killSignal: 'SIGKILL'
} {
  return {
    signal: AbortSignal.timeout(timeoutMs,),
    killSignal: 'SIGKILL',
  };
}

/**
 Says whether a thrown spawn error is the bound ending the child, as one sentence for the failure message.

 @param error - Value thrown by `nano-spawn`

 @param timeoutMs - Bound that was in force

 @returns A sentence naming the bound and the option that changes it, or an empty string when the child failed for another reason

 @example
 ```ts
 timeoutNote({ error: new Error('x',), timeoutMs: 1_000, },); // ''
 ```
 */
export function timeoutNote(
  {
    error,
    timeoutMs,
  }: {
    readonly error: unknown;
    readonly timeoutMs: number;
  },
): string {
  if (
    (error === null)
      || ((typeof error) !== 'object')
      || (!('isCanceled' in error))
      || (error.isCanceled !== true)
  )
    return '';
  return `timed out after ${String(timeoutMs / MS_PER_SECOND,)} seconds and was ended`;
}

//endregion Spawn options and failure wording
