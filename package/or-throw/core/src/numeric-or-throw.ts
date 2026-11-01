/**
 `numericOrThrow`: assert `typeof === 'number'` or `typeof === 'bigint'`, return the value or throw.
 
 @module
 */

import { formatUnknownValue, } from './format-unknown-value.ts';

import type { ExtractOrUnknown, } from './extract-or-unknown.ts';

/**
 Asserts that a value is a primitive number or bigint, returning it with
 non-numeric variants removed from the type.
 
 "Numeric" matches the `module-es` type vocabulary (`t numeric/`): the
 union of `number` and `bigint`. Use this when arithmetic is valid on
 either kind; reach for {@link numberOrThrow} or {@link bigintOrThrow} when the kind
 specifically matters.
 
 The return type uses {@link ExtractOrUnknown} so `unknown` inputs narrow to
 `number | bigint` instead of collapsing to `never`.
 
 @param value - Value to assert as numeric (number or bigint)
 
 @returns Same value with non-numeric variants excluded from the type
 
 @throws Error when value is not a primitive number or bigint
 
 @example
 ```ts
 const mixed: number | bigint | string = lookup();
 const n = numericOrThrow(mixed,);
 // n is number | bigint
 ```
 */
export function numericOrThrow<T,>(value: T,): ExtractOrUnknown<T, number | bigint> {
  if (((typeof value) !== 'number') && ((typeof value) !== 'bigint'))
    throw new Error(`Expected number or bigint, got ${typeof value} ${formatUnknownValue(value,)}`,);
  // oxlint-disable-next-line typescript-eslint/no-unsafe-type-assertion -- TypeScript cannot statically narrow generic T after typeof
  return value as ExtractOrUnknown<T, number | bigint>;
}
