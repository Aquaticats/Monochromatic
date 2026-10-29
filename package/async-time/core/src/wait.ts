/**
 Resolves after a specified delay in milliseconds.
 Wraps `setTimeout` in a promise.
 
 @param ms - delay before the returned promise resolves
 
 @returns promise that resolves to `undefined` after `ms` milliseconds
 
 @example
 ```ts
 import { wait, } from '\@monochromatic-dev/async-time-core';
 
 await wait(1000,);
 ```
 
 @example
 Throttle a loop:
 ```ts
 for (const item of items) {
   await process(item,);
   await wait(200,);
 }
 ```
 */
export function wait(ms: number,): Promise<undefined> {
  /**
   Promise capability created locally rather than borrowed from executor input.
   */
  const {
    promise,
    resolve,
  } = Promise.withResolvers<undefined>();
  setTimeout(
    function resolveAfterDelay(): void {
      resolve(undefined,);
    },
    ms,
  );
  return promise;
}
