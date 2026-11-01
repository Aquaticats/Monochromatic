/**
 `setOrThrow`: assert `Set` instance, return the narrowed value or throw.
 
 @module
 */

import { formatUnknownValue, } from './format-unknown-value.ts';

import type { ExtractOrUnknown, } from './extract-or-unknown.ts';

/**
 Asserts that a value is a `Set`, returning it with non-`Set` variants
 removed from the type.
 
 Uses `instanceof Set`, which is the correct predicate inside a single realm.
 Cross-realm `Set` values (an iframe's `Set`) will fail this check;
 callers that need cross-realm support should write their own check.
 The return type uses {@link ExtractOrUnknown} so `unknown` inputs narrow to
 `Set<unknown>` instead of collapsing to `never`.
 
 @param value - Value to assert as a `Set`
 
 @returns Same value with non-`Set` variants excluded from the type
 
 @throws Error when value is not a `Set`
 
 @example
 ```ts
 const maybeSet: Set<string> | string[] = lookup();
 const onlySet = setOrThrow(maybeSet,);
 // onlySet is Set<string>
 ```
 */
export function setOrThrow<T,>(value: T,): ExtractOrUnknown<T, Set<unknown>> {
  if (!(value instanceof Set))
    throw new Error(`Expected Set, got ${typeof value} ${formatUnknownValue(value,)}`,);
  // oxlint-disable-next-line typescript-eslint/no-unsafe-type-assertion -- TypeScript cannot statically narrow generic T after instanceof
  return value as ExtractOrUnknown<T, Set<unknown>>;
}
