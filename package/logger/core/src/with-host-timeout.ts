import { withTimeout, } from '@monochromatic-dev/async-time-core/ts';

/**
 Awaits a sink operation under a deadline when the host has timers.
 QuickJS-ng exposes `queueMicrotask` but no global `setTimeout`, so a timer
 cannot be scheduled there. When either timer primitive is missing, the
 operation is awaited directly: this preserves working sinks but cannot
 bound a custom sink that never settles. When both primitives exist, the
 original deadline remains intact.

 @param promise - Sink operation to await.

 @param ms - Deadline used when the host exposes timers.

 @param label - Operation named in a timeout diagnostic.

 @returns Sink operation's result.

 @throws Error when a host with timers reaches the deadline.

 @example
 ```ts
 await withHostTimeout({ promise: sink.verify(), ms: 5_000, label: 'sink verify' });
 ```
 */
export async function withHostTimeout<const Result>({
  promise,
  ms,
  label,
}: {
  readonly label: string;
  readonly ms: number;
  readonly promise: Promise<Result>;
},): Promise<Result> {
  if ((typeof globalThis.setTimeout) !== 'function')
    return await promise;
  if ((typeof globalThis.clearTimeout) !== 'function')
    return await promise;

  return await withTimeout({
    label,
    ms,
    promise,
  },);
}

/**
 Tells a host deadline apart from the awaited operation's own failure.

 The timeout utility builds its message from the same label and limit, so a
 caller that has to react differently to a deadline, such as keeping a slow
 probe's answer instead of retiring its sink, recognizes it without duplicating
 the race. A caught value is the deadline when its message is exactly the one
 that label and limit produce.

 @param error - Caught value from {@link withHostTimeout}.

 @param ms - Deadline the caller passed.

 @param label - Operation label the caller passed.

 @returns Whether the caught value is this deadline elapsing.

 @example
 ```ts
 const label = 'sink 0 verify';
 try { await withHostTimeout({ promise: probe, ms: 5_000, label }); }
 catch (error) { if (isHostDeadlineElapsed({ error, label, ms: 5_000 })) { /* probe still running *\/ } }
 ```
 */
export function isHostDeadlineElapsed({
  error,
  label,
  ms,
}: {
  readonly error: unknown;
  readonly label: string;
  readonly ms: number;
},): boolean {
  return Error.isError(error,)
    && (error.message === `Timed out after ${String(ms,)}ms: ${label}`);
}
