/**
 `objectOrThrow`: assert non-null object, return the narrowed value or throw.
 
 @module
 */

import { formatUnknownValue, } from './format-unknown-value.ts';

import type { ExtractOrUnknown, } from './extract-or-unknown.ts';

/**
 Asserts that a value is a non-null object, returning it with primitive and
 `null` variants removed from the type.
 
 Uses `typeof === 'object' && value !== null`. The `!== null` check is
 mandatory: `typeof null === 'object'` is a long-standing JavaScript quirk.
 Functions are not considered objects here (`typeof === 'function'`); use
 {@link functionOrThrow} for those.
 
 The return type uses {@link ExtractOrUnknown} so `unknown` inputs narrow to
 TypeScript's `object` type instead of collapsing to `never`.
 
 @param value - Value to assert as a non-null object
 
 @returns Same value with primitive and `null` variants excluded from the type
 
 @throws Error when value is `null`, `undefined`, or a primitive
 
 @example
 ```ts
 const raw: unknown = JSON.parse(payload,);
 const obj = objectOrThrow(raw,);
 // obj is object; safe to key-access via narrowed lookups
 ```
 */
export function objectOrThrow<T,>(value: T,): ExtractOrUnknown<T, object> {
  if (((typeof value) !== 'object') || (value === null))
    throw new Error(`Expected non-null object, got ${typeof value} ${formatUnknownValue(value,)}`,);
  // oxlint-disable-next-line typescript-eslint/no-unsafe-type-assertion -- TypeScript cannot statically narrow generic T after typeof + null check
  return value as ExtractOrUnknown<T, object>;
}
