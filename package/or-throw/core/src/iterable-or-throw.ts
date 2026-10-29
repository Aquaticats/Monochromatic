/**
 `iterableOrThrow`: assert sync-iterable, return the narrowed value or throw.
 
 @module
 */

import { formatUnknownValue, } from './format-unknown-value.ts';

/**
 Asserts that a value implements the sync-iterable protocol, returning it
 intersected with `Iterable<unknown>`.
 
 The runtime check accepts:
 - strings (`typeof === 'string'`, always iterable in JS)
 - objects with a `Symbol.iterator` method (`Symbol.iterator in value`
   where `value` is a non-null object)
 
 Primitives other than strings, `null`, and `undefined` throw.
 
 The return type is `T & Iterable<unknown>`. For `T = unknown`, the
 intersection collapses to `Iterable<unknown>`. For a `T` that already
 extends `Iterable<unknown>` (`string`, arrays, `Set`, `Map`), the
 intersection is `T`. For a `T` that does not (`number`, `boolean`),
 the intersection is `never` and the helper will always throw.
 
 @param value - Value to assert as sync-iterable
 
 @returns Same value with the `Iterable<unknown>` interface intersected in
 
 @throws Error when value is not sync-iterable
 
 @example
 ```ts
 function joinAll(items: unknown,): string {
   const iter = iterableOrThrow(items,);
   return [...iter,].join(', ',);
 }
 ```
 */
export function iterableOrThrow<T,>(value: T,): T & Iterable<unknown> {
  if ((typeof value) === 'string') {
    return value as T & Iterable<unknown>;
  }
  if ((value === null) || (value === undefined)
    || ((typeof value) !== 'object'))
    throw new Error(`Expected iterable, got ${typeof value} ${formatUnknownValue(value,)}`,);
  if (!(Symbol.iterator
    in value))
    throw new Error(`Expected iterable, got object without Symbol.iterator`,);
  // oxlint-disable-next-line typescript-eslint/no-unsafe-type-assertion -- Symbol.iterator presence narrows the runtime shape; TypeScript does not propagate this
  return value as T & Iterable<unknown>;
}
