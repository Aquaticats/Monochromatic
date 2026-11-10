/**
 Waits for a sink operation under the host deadline, reporting the deadline as
 a value rather than as an error. @module

 QuickJS-ng exposes `queueMicrotask` but no global `setTimeout`, so a timer
 cannot be scheduled there. When either timer primitive is missing, the
 operation is awaited directly: this preserves working sinks but cannot bound a
 custom sink that never settles.

 On a host with timers the wait has two possible endings, and callers need both:
 the operation settled, or the deadline won. Reporting the second as a returned
 value keeps a caller from having to recognize a rejection, which would tie this
 package's classification to the wording another package builds its timeout
 message from: a renamed label or a changed unit there would silently turn every
 deadline into an ordinary failure.
 */

import { settleWithin, } from '@monochromatic-dev/async-time-core/ts';

/**
 Awaits a sink operation under the host deadline.
 
 @param promise - Sink operation to await.
 
 @param ms - Deadline used when the host exposes both timer primitives.
 
 @returns `'settled'` when the operation settled first, `'expired'` when the deadline did.
 
 @throws The operation's own rejection. Reaching the deadline is never a throw,
 because it travels as the returned value; anything caught here is the
 operation's own failure.
 
 @mutates promise - `settleWithin` races it, which assimilates it through its `then` capability.
 
 @example
 ```ts
 const outcome = await settleWithinHostTimeout({ ms: 5_000, promise: sink.verify() });
 if (outcome === 'expired') { /* startup stops waiting, the probe keeps running *\/ }
 ```
 */
export async function settleWithinHostTimeout(
  {
    ms,
    promise,
  }: {
    readonly ms: number;
    readonly promise: Promise<unknown>;
  },
): Promise<'settled' | 'expired'> {
  if (((typeof globalThis.setTimeout) !== 'function') || ((typeof globalThis.clearTimeout) !== 'function')) {
    await promise;
    return 'settled';
  }

  return await settleWithin({
    ms,
    promise,
  },);
}
