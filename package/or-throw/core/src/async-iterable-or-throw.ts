/**
 `asyncIterableOrThrow`: assert async-iterable, return the narrowed value or throw.
 
 @module
 */

import { formatUnknownValue, } from './format-unknown-value.ts';

/**
 Asserts that a value implements the async-iterable protocol, returning it
 intersected with `AsyncIterable<unknown>`.
 
 The runtime check accepts non-null objects with a `Symbol.asyncIterator`
 method. Primitives (including strings), `null`, and `undefined` throw.
 Sync-iterable values that do not also implement `Symbol.asyncIterator`
 throw; use {@link maybeAsyncIterableOrThrow} to accept either.
 
 The return type is `T & AsyncIterable<unknown>`. For `T = unknown`, the
 intersection collapses to `AsyncIterable<unknown>`. For a `T` that already
 extends `AsyncIterable<unknown>`, the intersection is `T`. For a `T` that
 does not, the intersection is `never` and the helper will always throw.
 
 @param value - Value to assert as async-iterable
 
 @returns Same value with the `AsyncIterable<unknown>` interface intersected in
 
 @throws Error when value is not async-iterable
 
 @example
 ```ts
 async function drain(stream: unknown,): Promise<void> {
   const iter = asyncIterableOrThrow(stream,);
   for await (const chunk of iter) {
     // ...
   }
 }
 ```
 */
export function asyncIterableOrThrow<T,>(value: T,): T & AsyncIterable<unknown> {
  if ((value === null) || (value === undefined)
    || ((typeof value) !== 'object'))
    throw new Error(`Expected async iterable, got ${typeof value} ${formatUnknownValue(value,)}`,);
  if (!(Symbol.asyncIterator
    in value))
    throw new Error(`Expected async iterable, got object without Symbol.asyncIterator`,);
  // oxlint-disable-next-line typescript-eslint/no-unsafe-type-assertion -- Symbol.asyncIterator presence narrows the runtime shape; TypeScript does not propagate this
  return value as T & AsyncIterable<unknown>;
}
