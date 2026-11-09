/**
 How much time a bound leaves, for waits that must not outlast it.

 @module
 */

/**
 How long is left before a deadline.

 @param deadline - when the wait must be over, as milliseconds since the epoch

 @returns the milliseconds left, and zero once the deadline has passed

 @example
 ```ts
 const left = millisecondsLeftUntil(Date.now() + 500,);
 ```
 */
export function millisecondsLeftUntil(deadline: number,): number {
  return Math.max(
    0,
    deadline - Date.now(),
  );
}
