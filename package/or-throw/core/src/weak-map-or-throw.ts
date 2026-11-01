/**
 `weakMapOrThrow`: assert `WeakMap` instance, return the narrowed value or throw.
 
 @module
 */

import { formatUnknownValue, } from './format-unknown-value.ts';

import type { ExtractOrUnknown, } from './extract-or-unknown.ts';

/**
 Asserts that a value is a `WeakMap`, returning it with non-`WeakMap`
 variants removed from the type.
 
 Uses `instanceof WeakMap`, which is the correct predicate inside a single
 realm. `WeakMap` has no enumerable size or iteration, so this helper is
 useful mainly for type-narrowing branches that want to distinguish weak
 containers from regular ones.
 The return type uses {@link ExtractOrUnknown} so `unknown` inputs narrow to
 `WeakMap<object, unknown>` instead of collapsing to `never`.
 
 @param value - Value to assert as a `WeakMap`
 
 @returns Same value with non-`WeakMap` variants excluded from the type
 
 @throws Error when value is not a `WeakMap`
 
 @example
 ```ts
 const maybeWeak: WeakMap<object, string> | Map<object, string> = lookup();
 const onlyWeak = weakMapOrThrow(maybeWeak,);
 // onlyWeak is WeakMap<object, string>
 ```
 */
export function weakMapOrThrow<T,>(
  value: T,
): ExtractOrUnknown<T, WeakMap<object, unknown>> {
  if (!(value instanceof WeakMap))
    throw new Error(`Expected WeakMap, got ${typeof value} ${formatUnknownValue(value,)}`,);
  // oxlint-disable-next-line typescript-eslint/no-unsafe-type-assertion -- TypeScript cannot statically narrow generic T after instanceof
  return value as ExtractOrUnknown<T, WeakMap<object, unknown>>;
}
