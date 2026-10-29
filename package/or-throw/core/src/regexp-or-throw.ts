/**
 `regExpOrThrow`: assert `RegExp` instance, return the narrowed value or throw.
 
 @module
 */

import { formatUnknownValue, } from './format-unknown-value.ts';

import type { ExtractOrUnknown, } from './extract-or-unknown.ts';

/**
 Asserts that a value is a `RegExp`, returning it with non-`RegExp` variants
 removed from the type.
 
 Uses `instanceof RegExp`. Regex-shaped strings ("^abc$") and objects with a
 `.test()` method are intentionally rejected; callers that want to accept
 either should normalize first (`new RegExp(maybe)`).
 The return type uses {@link ExtractOrUnknown} so `unknown` inputs narrow to
 `RegExp` instead of collapsing to `never`.
 
 @param value - Value to assert as a `RegExp`
 
 @returns Same value with non-`RegExp` variants excluded from the type
 
 @throws Error when value is not a `RegExp`
 
 @example
 ```ts
 const maybePattern: RegExp | string = readPattern();
 const re = regExpOrThrow(maybePattern,);
 // re is RegExp
 ```
 */
export function regExpOrThrow<T,>(value: T,): ExtractOrUnknown<T, RegExp> {
  if (!(value instanceof RegExp))
    throw new Error(`Expected RegExp, got ${typeof value} ${formatUnknownValue(value,)}`,);
  // oxlint-disable-next-line typescript-eslint/no-unsafe-type-assertion -- TypeScript cannot statically narrow generic T after instanceof
  return value as ExtractOrUnknown<T, RegExp>;
}
