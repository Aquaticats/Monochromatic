/**
 `arrayOrThrow`: assert array, return the narrowed value or throw.
 
 @module
 */

import { formatUnknownValue, } from './format-unknown-value.ts';

import type { ExtractOrUnknown, } from './extract-or-unknown.ts';

/**
 Asserts that a value is an array, returning it with non-array variants
 removed from the type.
 
 The runtime check is `Array.isArray`, which is the only correct array
 predicate in JavaScript: it works across realms (iframe, worker) and
 across proxy wrappers, where `instanceof Array` fails. The return type
 uses {@link ExtractOrUnknown} so `unknown` inputs (e.g. `JSON.parse` results)
 narrow to `readonly unknown[]` instead of collapsing to `never`.
 
 @param value - Value to assert as an array
 
 @returns Same value with non-array variants excluded from the type
 
 @throws Error when value is not an array
 
 @example
 ```ts
 const parsed: unknown = JSON.parse(payload,);
 const items = arrayOrThrow(parsed,);
 // items is readonly unknown[]
 
 const mixed: string | string[] = lookup();
 const list = arrayOrThrow(mixed,);
 // list is string[]
 ```
 */
export function arrayOrThrow<T,>(value: T,): ExtractOrUnknown<T, readonly unknown[]> {
  if (!Array.isArray(value,))
    throw new Error(`Expected array, got ${typeof value} ${formatUnknownValue(value,)}`,);
  // oxlint-disable-next-line typescript-eslint/no-unsafe-type-assertion -- TypeScript cannot statically narrow generic T after Array.isArray
  return value as ExtractOrUnknown<T, readonly unknown[]>;
}
