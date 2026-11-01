/**
 `maybeAsyncIterableOrThrow`: assert sync- or async-iterable, return the value or throw.
 
 @module
 */

import { formatUnknownValue, } from './format-unknown-value.ts';

/**
 Asserts that a value implements either the sync-iterable or the
 async-iterable protocol, returning it intersected with the union of both
 protocols.
 
 The runtime check accepts:
 - strings (always sync-iterable)
 - objects with `Symbol.iterator` or `Symbol.asyncIterator`
 
 Primitives other than strings, `null`, and `undefined` throw. Objects with
 neither symbol throw.
 
 Use this when a consumer iterates with `for await`, which works on both
 sync iterables (each value is awaited as-is) and async iterables; this
 matches `Symbol.asyncIterator || Symbol.iterator` resolution at runtime.
 
 @param value - Value to assert as iterable in either direction
 
 @returns Same value with the iterable-or-async-iterable union intersected in
 
 @throws Error when value is not iterable in either direction
 
 @example
 ```ts
 async function consume(source: unknown,): Promise<void> {
   const iter = maybeAsyncIterableOrThrow(source,);
   for await (const item of iter) {
     // ...
   }
 }
 ```
 */
export function maybeAsyncIterableOrThrow<T,>(
  value: T,
): T & (Iterable<unknown> | AsyncIterable<unknown>) {
  if ((typeof value) === 'string') {
    return value as T & (Iterable<unknown> | AsyncIterable<unknown>);
  }
  if ((value === null) || (value === undefined)
    || ((typeof value) !== 'object')) {
    throw new Error(
      `Expected iterable or async iterable, got ${typeof value} ${formatUnknownValue(value,)}`,
    );
  }
  if ((!(Symbol.iterator
    in value)) && (!(Symbol.asyncIterator
      in value))) {
    throw new Error(
      `Expected iterable or async iterable, got object without either Symbol.iterator or Symbol.asyncIterator`,
    );
  }
  // oxlint-disable-next-line typescript-eslint/no-unsafe-type-assertion -- protocol-symbol presence narrows the runtime shape; TypeScript does not propagate this
  return value as T & (Iterable<unknown> | AsyncIterable<unknown>);
}
