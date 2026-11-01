/**
 `dateOrThrow`: assert `Date` instance, return the narrowed value or throw.
 
 @module
 */

import { formatUnknownValue, } from './format-unknown-value.ts';

import type { ExtractOrUnknown, } from './extract-or-unknown.ts';

/**
 Asserts that a value is a `Date`, returning it with non-`Date` variants
 removed from the type.
 
 Uses `instanceof Date`. Invalid dates (`new Date('garbage')`,
 `getTime() === NaN`) still pass: this helper checks the type, not the
 validity. Callers that need a valid date should chain a separate check
 (e.g. `Number.isNaN(date.getTime())`).
 The return type uses {@link ExtractOrUnknown} so `unknown` inputs narrow to
 `Date` instead of collapsing to `never`.
 
 @param value - Value to assert as a `Date`
 
 @returns Same value with non-`Date` variants excluded from the type
 
 @throws Error when value is not a `Date`
 
 @example
 ```ts
 const maybeDate: Date | string = parseField(input,);
 const date = dateOrThrow(maybeDate,);
 // date is Date
 ```
 */
export function dateOrThrow<T,>(value: T,): ExtractOrUnknown<T, Date> {
  if (!(value instanceof Date))
    throw new Error(`Expected Date, got ${typeof value} ${formatUnknownValue(value,)}`,);
  // oxlint-disable-next-line typescript-eslint/no-unsafe-type-assertion -- TypeScript cannot statically narrow generic T after instanceof
  return value as ExtractOrUnknown<T, Date>;
}
