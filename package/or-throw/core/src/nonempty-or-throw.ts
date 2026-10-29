/**
 `nonemptyOrThrow`: assert nonempty container, return the value or throw.
 
 @module
 */

import { getSize, } from './size.ts';

/**
 Asserts that a recognized container is nonempty (size greater than zero),
 returning the same value.
 
 Recognized container shapes (see {@link getSize} in `size.ts`): strings, arrays,
 `Set`, `Map`, plain objects. Values without a recognized size shape throw,
 as do values whose size is zero.
 
 Type narrowing is intentionally pass-through; TypeScript cannot express
 "array with at least one element" as a refinement of the parameter type
 without literal-type tricks that hurt more than they help. The check is at
 runtime.
 
 @param value - Container expected to have at least one element
 
 @returns Same value
 
 @throws Error when value lacks a recognized size shape, or when size is zero
 
 @example
 ```ts
 nonemptyOrThrow([1, 2, 3,],);    // returns [1, 2, 3]
 nonemptyOrThrow('hello',);       // returns 'hello'
 nonemptyOrThrow(new Set([1,],),); // returns the Set
 nonemptyOrThrow([],);            // throws (size 0)
 nonemptyOrThrow('',);            // throws (size 0)
 nonemptyOrThrow(42,);            // throws (not a container)
 ```
 */
export function nonemptyOrThrow<T,>(value: T,): T {
  /**
   Container length or undefined for non-container inputs; zero triggers the throw branch.
   */
  const size = getSize(value,);
  if (size === 0)
    throw new Error(`Expected nonempty container, got size 0`,);
  return value;
}
