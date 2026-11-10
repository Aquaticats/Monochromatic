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
 Counts the steps of a repeat that runs while a counter stays below a bound, one number at a time.

 The count is rounded up, so a fractional bound keeps every step a `counter < bound` loop took, and a bound of zero or
 less gives no step at all. The walk is lazy: nothing is built before the first step is taken, so a bound larger than an
 array can hold still starts at zero. Each call gives its own walk, and one walk is read once; ask for a new one to
 step from zero again.

 @param count - exclusive upper bound of the numbers

 @returns the numbers from zero to `count`, exclusive, one at a time

 @example
 ```ts
 [...numbersBelow(4,),]; // [0, 1, 2, 3]
 ```

 @example
 ```ts
 numbersBelow(2.5,).next(); // { value: 0, done: false }
 ```
 */
export function* numbersBelow(count: number,): Generator<number> {
  /**
   Number of steps the bound asks for, rounded up so a fractional bound keeps every step a counted loop took.
   */
  const steps = Math.max(
    0,
    Math.ceil(count,),
  );

  /**
   Number the next step carries, in a one-slot holder, because it outlives every yield.
   */
  const cursor = { next: 0, };

  while (cursor.next < steps) {
    /**
     Number this step yields.
     */
    const { next, } = cursor;
    yield next;
    cursor.next = next + 1;
  }
}
