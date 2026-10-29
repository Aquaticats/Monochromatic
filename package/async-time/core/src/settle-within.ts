/**
 Waits for a promise to settle, or for a deadline, whichever comes first,
 WITHOUT treating the deadline as an error.
 
 The difference from `withTimeout` is the whole reason this exists.
 `withTimeout` rejects when the deadline wins, which is right when the caller
 needed the value. This resolves either way, which is right when the caller
 needed the WAIT: a grace window granted to stragglers, a settle-or-move-on
 step, anything where expiry is an ordinary outcome rather than a failure.
 Expressing that with `withTimeout` means catching an error on the expected
 path and then proving it was the timeout's error and not a real one.
 
 WHY NOT `Promise.race([promise, wait(ms,),],)`, which is what this replaces:
 `Promise.race` settles on the first result and does nothing whatever to the
 loser. `wait` is a bare `setTimeout` that returns no handle, so when the
 promise wins the timer is left pending, unclearable, and holding the Node
 event loop. A process doing that finishes its work and then sits there until
 the last timer expires. Measured at three minutes on a 180-second grace
 window, which was long enough to look like a hang and to stall anything
 chained behind it.
 
 @param promise - work to wait on
 
 @param ms - longest this will wait
 
 @returns Which side finished first, so a caller that cares can tell
 
 @mutates promise - `Promise.race` performs caller-owned promise assimilation through its `then` capability.
 
 @example
 ```ts
 import { settleWithin, } from '\@monochromatic-dev/async-time-core';
 
 const finished = await settleWithin({
   promise: Promise.allSettled(asks,),
   ms: graceMs,
 },);
 if (finished === 'expired') l.warn('grace ran out',);
 ```
 */
export async function settleWithin({
  promise,
  ms,
}: {
  readonly ms: number;
  readonly promise: Promise<unknown>;
},): Promise<'settled' | 'expired'> {
  /**
   Deadline's own promise, resolved by the timer rather than rejected, so
   expiry travels as a value and no caller has to catch anything.
   */
  const {
    promise: deadline,
    resolve,
  } = Promise.withResolvers<'expired'>();

  /**
   Timer handle, assigned synchronously so the disposer never observes an
   unset binding and the `const` stays free of a banned nullish union.
   */
  const timer = setTimeout(
    function onDeadline(): void {
      resolve('expired',);
    },
    ms,
  );

  /**
   Clears the timer however the race ends.
   
   This is the entire point of the function. Without it a won race leaves a
   live `setTimeout` that keeps the event loop alive for the remainder of
   `ms`, which is invisible until something waits on the process exiting.
   */
  using _cleanup = {
    [Symbol.dispose](): void {
      clearTimeout(timer,);
    },
  };

  return await Promise.race([
    (async function settled(): Promise<'settled'> {
      await promise;
      return 'settled';
    })(),
    deadline,
  ],);
}
