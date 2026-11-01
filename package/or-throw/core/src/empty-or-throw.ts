/**
 `emptyOrThrow`: assert empty container, return the value or throw.
 
 @module
 */

import { getSize, } from './size.ts';

/**
 Asserts that a recognized container is empty (zero size), returning the same value.
 
 Recognized container shapes (see {@link getSize} in `size.ts`): strings, arrays,
 `Set`, `Map`, plain objects. Values without a recognized size shape throw,
 as do values whose size is non-zero.
 
 Type narrowing is intentionally pass-through. TypeScript cannot represent
 "empty array" or "empty string" usefully without literal-type tricks, so the
 return type is `T`. The check is at runtime; the type just records "this
 value continues to be `T`, just verified empty."
 
 @param value - Container expected to have zero size
 
 @returns Same value
 
 @throws Error when value lacks a recognized size shape, or when size is non-zero
 
 @example
 ```ts
 emptyOrThrow([],);          // returns []
 emptyOrThrow('',);          // returns ''
 emptyOrThrow(new Set(),);   // returns the Set
 emptyOrThrow({},);          // returns {}
 emptyOrThrow([1],);         // throws (size 1)
 emptyOrThrow(42,);          // throws (not a container)
 ```
 */
export function emptyOrThrow<T,>(value: T,): T {
  /**
   Container length or undefined for non-container inputs; non-zero triggers the throw branch.
   */
  const size = getSize(value,);
  if (size !== 0)
    throw new Error(`Expected empty container, got size ${String(size,)}`,);
  return value;
}
