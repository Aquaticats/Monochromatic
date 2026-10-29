/**
 `weakSetOrThrow`: assert `WeakSet` instance, return the narrowed value or throw.
 
 @module
 */

import { formatUnknownValue, } from './format-unknown-value.ts';

import type { ExtractOrUnknown, } from './extract-or-unknown.ts';

/**
 Asserts that a value is a `WeakSet`, returning it with non-`WeakSet` variants
 removed from the type.
 
 Uses `instanceof WeakSet`, which is the correct predicate inside a single
 realm. `WeakSet` has no enumerable size or iteration, so this helper is
 useful mainly for type-narrowing branches that want to distinguish weak
 containers from regular ones.
 The return type uses {@link ExtractOrUnknown} so `unknown` inputs narrow to
 `WeakSet<object>` instead of collapsing to `never`.
 
 @param value - Value to assert as a `WeakSet`
 
 @returns Same value with non-`WeakSet` variants excluded from the type
 
 @throws Error when value is not a `WeakSet`
 
 @example
 ```ts
 const maybeWeak: WeakSet<object> | Set<object> = lookup();
 const onlyWeak = weakSetOrThrow(maybeWeak,);
 // onlyWeak is WeakSet<object>
 ```
 */
export function weakSetOrThrow<T,>(value: T,): ExtractOrUnknown<T, WeakSet<object>> {
  if (!(value instanceof WeakSet))
    throw new Error(`Expected WeakSet, got ${typeof value} ${formatUnknownValue(value,)}`,);
  // oxlint-disable-next-line typescript-eslint/no-unsafe-type-assertion -- TypeScript cannot statically narrow generic T after instanceof
  return value as ExtractOrUnknown<T, WeakSet<object>>;
}
