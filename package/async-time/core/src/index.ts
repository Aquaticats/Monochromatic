/**
 Async timing primitives.
 
 - {@link wait}`(ms)`: resolves after a delay; thin wrapper over `setTimeout`.
 - {@link withTimeout}`({ promise, ms, label })`: races a promise against a
   deadline and rejects with a labeled error if the deadline wins.
 - {@link settleWithin}`({ promise, ms })`: same race, but expiry is an
   ordinary outcome returned as a value rather than an error.
 
 Choose among them: use `wait` when the delay itself is the point, as in a
 polling loop that awaits it to completion. Use `withTimeout` when the caller
 needed the value and not getting it is a failure. Use `settleWithin` when the
 caller needed the wait and expiry is normal.
 
 Avoid `Promise.race([work, wait(ms,),],)`. `Promise.race` does nothing to the
 loser, and `wait` returns no handle, so a won race leaves a live `setTimeout`
 holding the event loop until it expires. Both other helpers clear their timer
 through a `using` disposer; that is most of why they exist.
 
 @example
 ```ts
 import { settleWithin, wait, withTimeout, } from '\@monochromatic-dev/async-time-core/ts';
 
 await wait(500,);
 
 const data = await withTimeout({
   promise: fetch('/api/data',),
   ms: 5000,
   label: 'fetch user data',
 },);
 
 const finished = await settleWithin({
   promise: Promise.allSettled(stragglers,),
   ms: 30_000,
 },);
 ```
 
 @packageDocumentation
 */

export { settleWithin, } from './settle-within.ts';

export { wait, } from './wait.ts';

export { withTimeout, } from './with-timeout.ts';
