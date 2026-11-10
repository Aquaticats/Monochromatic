/**
 The numbers from zero up to a bound, for a repeat that runs a fixed number of times.

 @example
 ```ts
 import { numbersBelow, } from './numbers-below.ts';

 for (const frame of numbersBelow(3,))
   console.log(frame,); // 0, 1, 2
 ```

 @packageDocumentation
 */

/**
 Counts the steps of a repeat that runs while a counter stays below a bound.

 The count is rounded up, so a fractional bound keeps every step a `counter < bound` loop took, and a bound of zero or
 less gives no step at all. A bound of `Infinity` throws from the array length, where a counted loop would never end.

 @param count - exclusive upper bound of the numbers

 @returns the numbers from zero to `count`, exclusive

 @throws RangeError when the rounded count is larger than an array can hold

 @example
 ```ts
 numbersBelow(4,); // [0, 1, 2, 3]
 ```

 @example
 ```ts
 numbersBelow(2.5,); // [0, 1, 2]
 ```
 */
export function numbersBelow(count: number,): readonly number[] {
  return Array.from(
    { length: Math.max(
      0,
      Math.ceil(count,),
    ), },
    function indexOfStep(
      _unused: unknown,
      index: number,
    ): number {
      return index;
    },
  );
}
