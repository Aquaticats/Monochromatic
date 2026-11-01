/**
 `bigintOrThrow`: assert `typeof === 'bigint'`, return the narrowed value or throw.
 
 @module
 */

import { formatUnknownValue, } from './format-unknown-value.ts';

import type { ExtractOrUnknown, } from './extract-or-unknown.ts';

/**
 Asserts that a value is a primitive bigint, returning it with non-bigint
 variants removed from the type.
 
 Uses `typeof === 'bigint'`. Number values whose magnitudes are
 representable as bigint are still rejected: this helper checks the type,
 not numeric equivalence.
 
 The return type uses {@link ExtractOrUnknown} so `unknown` inputs narrow to
 `bigint` instead of collapsing to `never`.
 
 @param value - Value to assert as a primitive bigint
 
 @returns Same value with non-bigint variants excluded from the type
 
 @throws Error when value is not a primitive bigint
 
 @example
 ```ts
 const mixed: bigint | number = lookup();
 const big = bigintOrThrow(mixed,);
 // big is bigint
 ```
 */
export function bigintOrThrow<T,>(value: T,): ExtractOrUnknown<T, bigint> {
  if ((typeof value) !== 'bigint')
    throw new Error(`Expected bigint, got ${typeof value} ${formatUnknownValue(value,)}`,);
  // oxlint-disable-next-line typescript-eslint/no-unsafe-type-assertion -- TypeScript cannot statically narrow generic T after typeof
  return value as ExtractOrUnknown<T, bigint>;
}
