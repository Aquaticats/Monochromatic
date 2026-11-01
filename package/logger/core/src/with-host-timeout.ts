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
