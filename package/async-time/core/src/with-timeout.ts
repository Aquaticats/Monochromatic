/**
 Races a promise against a deadline.
 Resolves with the promise's value if it settles in time;
 rejects with a labeled `Error` otherwise.
 
 Uses `Promise.race` with a `using`-disposed timer so the
 `setTimeout` handle is cleared whether the race wins or loses.
 
 @param promise - promise to race against the timeout
 
 @param ms - timeout duration in milliseconds
 
 @param label - human-readable label for the timeout error message
 
 @returns same value as the input promise when it settles before the timeout
 
 @mutates promise - `Promise.race` performs caller-owned promise assimilation through its `then` capability.
 
 @throws Error when the timeout expires before the promise settles
 
 @example
 ```ts
 import { withTimeout, } from '\@monochromatic-dev/async-time-core';
 
 const data = await withTimeout({
   promise: fetch(url,),
   ms: 5000,
   label: 'fetch user data',
 },);
 ```
 */
export async function withTimeout<T,>({
  promise,
  ms,
  label,
}: {
  label: string;
  ms: number;
  promise: Promise<T>;
},): Promise<T> {
  /**
   Rejecter and its promise captured up front via `Promise.withResolvers`,
   so the timer can reject the deadline promise and the race can await it
   without a `new Promise` executor or a shared mutable timer binding.
   */
  const {
    promise: timeoutPromise,
    reject,
  } = Promise.withResolvers<never>();

  /**
   setTimeout handle read by the `using` disposer. Assigned synchronously
   here so the disposer never observes an unset handle and the binding stays
   a `const` free of a banned `T | undefined` union.
   */
  const timer = setTimeout(
    function onTimeout() {
      reject(new Error(`Timed out after ${String(ms,)}ms: ${label}`,),);
    },
    ms,
  );

  /**
   Clears the pending setTimeout when the race resolves before the timeout fires.
   
   Without this, the timer would still call `reject` after `Promise.race` has
   already settled, producing an unhandled rejection on a promise no caller awaits.
   */
  using _cleanup = {
    [Symbol.dispose](): void {
      clearTimeout(timer,);
    },
  };

  return await Promise.race([
    promise,
    timeoutPromise,
  ],);
}
