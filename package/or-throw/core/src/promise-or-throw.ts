/**
 `promiseOrThrow`: assert `Promise` instance, return the narrowed value or throw.
 
 @module
 */

import { formatUnknownValue, } from './format-unknown-value.ts';

import type { ExtractOrUnknown, } from './extract-or-unknown.ts';

/**
 Asserts that a value is a `Promise`, returning it with non-`Promise`
 variants removed from the type.
 
 Uses `instanceof Promise`, which is the correct predicate for native
 promises inside a single realm. Thenables (objects with a `.then` method
 but not native `Promise` instances) are intentionally rejected; callers
 that want to accept thenables should write a duck-type check instead.
 Cross-realm `Promise` values (an iframe's `Promise`) will fail this check.
 The return type uses {@link ExtractOrUnknown} so `unknown` inputs narrow to
 `Promise<unknown>` instead of collapsing to `never`.
 
 @param value - Value to assert as a `Promise`
 
 @returns Same value with non-`Promise` variants excluded from the type
 
 @throws Error when value is not a `Promise`
 
 @example
 ```ts
 const result: string | Promise<string> = lookup();
 const pending = promiseOrThrow(result,);
 // pending is Promise<string>
 const value = await pending;
 ```
 */
export function promiseOrThrow<T,>(value: T,): ExtractOrUnknown<T, Promise<unknown>> {
  if (!(value instanceof Promise))
    throw new Error(`Expected Promise, got ${typeof value} ${formatUnknownValue(value,)}`,);
  // oxlint-disable-next-line typescript-eslint/no-unsafe-type-assertion -- TypeScript cannot statically narrow generic T after instanceof
  return value as ExtractOrUnknown<T, Promise<unknown>>;
}
