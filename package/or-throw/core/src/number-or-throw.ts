/**
 `numberOrThrow`: assert `typeof === 'number'`, return the narrowed value or throw.
 
 @module
 */

import { formatUnknownValue, } from './format-unknown-value.ts';

import type { ExtractOrUnknown, } from './extract-or-unknown.ts';

/**
 Asserts that a value is a primitive number, returning it with non-number
 variants removed from the type.
 
 Uses `typeof === 'number'`. `NaN` and `Infinity` pass: they are still
 `number`-typed values in JavaScript. Callers that need a finite or integer
 number should compose a separate check (`Number.isFinite`,
 `Number.isInteger`). Boxed-number wrappers (`new Number(1)`,
 `typeof === 'object'`) are intentionally rejected.
 
 The return type uses {@link ExtractOrUnknown} so `unknown` inputs narrow to
 `number` instead of collapsing to `never`.
 
 @param value - Value to assert as a primitive number
 
 @returns Same value with non-number variants excluded from the type
 
 @throws Error when value is not a primitive number
 
 @example
 ```ts
 const raw: unknown = JSON.parse(payload,);
 const count = numberOrThrow(raw,);
 // count is number
 ```
 */
export function numberOrThrow<T,>(value: T,): ExtractOrUnknown<T, number> {
  if ((typeof value) !== 'number')
    throw new Error(`Expected number, got ${typeof value} ${formatUnknownValue(value,)}`,);
  // oxlint-disable-next-line typescript-eslint/no-unsafe-type-assertion -- TypeScript cannot statically narrow generic T after typeof
  return value as ExtractOrUnknown<T, number>;
}
