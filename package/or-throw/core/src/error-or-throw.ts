/**
 `errorOrThrow`: assert genuine `Error` instance, return the narrowed value or throw.
 
 @module
 */

import { formatUnknownValue, } from './format-unknown-value.ts';

import type { ExtractOrUnknown, } from './extract-or-unknown.ts';

/**
 Asserts that a value is a genuine `Error`, returning it with non-`Error`
 variants removed from the type.
 
 Uses `Error.isError`, which tests the internal error data rather than the
 prototype chain. It matches `Error` and every subclass (`TypeError`,
 `RangeError`, and any user-defined `class FooError extends Error`) from any
 realm (iframe, worker, `node:vm`). Plain objects with a `.message` property,
 `Error`-shaped fakes, and `Object.create(Error.prototype)` impostors are
 rejected; for those, use a duck-type check or `errorLike` from `module-es`.
 
 @param value - Value to assert as an `Error`
 
 @returns Same value with non-`Error` variants excluded from the type
 
 @throws Error when value is not an `Error`
 
 @example
 ```ts
 try {
   doWork();
 }
 catch (caught: unknown) {
   const error = errorOrThrow(caught,);
   // error is Error; safe to read error.message
 }
 ```
 */
export function errorOrThrow<T,>(value: T,): ExtractOrUnknown<T, Error> {
  if (!(Error.isError(value,)))
    throw new Error(`Expected Error, got ${typeof value} ${formatUnknownValue(value,)}`,);
  // oxlint-disable-next-line typescript-eslint/no-unsafe-type-assertion -- TypeScript cannot statically narrow generic T after instanceof
  return value as ExtractOrUnknown<T, Error>;
}
