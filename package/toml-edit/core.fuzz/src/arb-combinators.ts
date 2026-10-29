/**
 Small fast-check combinators shared by the document generators.
 
 @module
 */

import {
  type Arbitrary,
  constant,
} from 'fast-check';

/**
 Draw one dependent arbitrary per input item and collect the results in order.
 
 fast-check has no built-in "list of items to arbitrary of results" combinator;
 folding `chain` keeps each draw independent while preserving input order, and
 avoids the rest-parameter spread that `tuple(...arbs)` would need.
 
 @param items - Inputs to expand, one arbitrary each.
 
 @param make - Produces the arbitrary for one item at its index.
 
 @returns Arbitrary of the drawn results in `items` order.
 
 @mutates make - Invoking caller-supplied factory can change captured or otherwise reachable state.
 
 @example
 ```ts
 drawEach({ items: ['a', 'b',], make: (name,) => keyBlockArbitrary({ owner: name, },), },);
 ```
 */
export function drawEach<T, R,>(
  {
    items,
    make,
  }: {
    readonly items: readonly T[];
    readonly make: (
      item: T,
      index: number
    ) => Arbitrary<R>;
  },
): Arbitrary<readonly R[]> {
  /**
   Arbitrary accumulated across input items.
   */
  let accumulated = constant([] as readonly R[],);
  for (const [index, item,] of items.entries()) {
    /**
     Prior arbitrary captured independently of reassignment.
     */
    const prior = accumulated;
    accumulated = prior.chain(function append(collected,) {
      return make(
        item,
        index,
      )
        .map(function push(next,) { return [
          ...collected,
          next,
        ]; },);
    },);
  }
  return accumulated;
}
