/**
 `mapOrThrow`: assert `Map` instance, return the narrowed value or throw.
 
 @module
 */

import { formatUnknownValue, } from './format-unknown-value.ts';

import type { ExtractOrUnknown, } from './extract-or-unknown.ts';

/**
 Asserts that a value is a `Map`, returning it with non-`Map` variants
 removed from the type.
 
 Uses `instanceof Map`, which is the correct predicate inside a single realm.
 Cross-realm `Map` values (an iframe's `Map`) will fail this check;
 callers that need cross-realm support should write their own check.
 The return type uses {@link ExtractOrUnknown} so `unknown` inputs narrow to
 `Map<unknown, unknown>` instead of collapsing to `never`.
 
 @param value - Value to assert as a `Map`
 
 @returns Same value with non-`Map` variants excluded from the type
 
 @throws Error when value is not a `Map`
 
 @example
 ```ts
 const maybeMap: Map<string, number> | Record<string, number> = lookup();
 const onlyMap = mapOrThrow(maybeMap,);
 // onlyMap is Map<string, number>
 ```
 */
export function mapOrThrow<T,>(value: T,): ExtractOrUnknown<T, Map<unknown, unknown>> {
  if (!(value instanceof Map))
    throw new Error(`Expected Map, got ${typeof value} ${formatUnknownValue(value,)}`,);
  // oxlint-disable-next-line typescript-eslint/no-unsafe-type-assertion -- TypeScript cannot statically narrow generic T after instanceof
  return value as ExtractOrUnknown<T, Map<unknown, unknown>>;
}
