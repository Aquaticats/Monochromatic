/**
 Runtime size detection for container-shaped values.
 
 @module
 */

import { formatUnknownValue, } from './format-unknown-value.ts';

/**
 Returns the element count of a recognized container shape.
 
 Recognized:
 - strings (`.length`)
 - arrays (`.length`)
 - `Set` and `Map` (`.size`)
 - plain objects (`Object.keys(value).length`)
 
 Intentionally rejected:
 - `null` and `undefined`
 - `WeakSet` and `WeakMap` (no enumerable size)
 - bare iterables and async iterables (sizing requires consumption)
 - primitives without `.length` (numbers, booleans, symbols, bigints, functions)
 
 @param value - Value to size
 
 @returns Element count for the matched shape
 
 @throws Error when value lacks a recognized size shape
 
 @example
 ```ts
 getSize('hello',);          // 5
 getSize([1, 2, 3,],);       // 3
 getSize(new Set([1, 2,],),); // 2
 getSize({ a: 1, },);        // 1
 getSize(null,);             // throws
 getSize(42,);               // throws
 ```
 */
export function getSize(value: unknown,): number {
  if ((value === null) || (value === undefined))
    throw new Error(`Expected sized container, got ${formatUnknownValue(value,)}`,);
  if ((typeof value) === 'string')
    return value.length;
  if (Array.isArray(value,))
    return value.length;
  if ((value instanceof Set) || (value instanceof Map))
    return value.size;
  if ((typeof value) === 'object')
    return Object.keys(value as object,)
      .length;
  throw new Error(`Expected sized container, got ${typeof value}`,);
}
