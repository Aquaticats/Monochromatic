/**
 `stringOrThrow`: assert `typeof === 'string'`, return the narrowed value or throw.
 
 @module
 */

import { formatUnknownValue, } from './format-unknown-value.ts';

import type { ExtractOrUnknown, } from './extract-or-unknown.ts';

/**
 Asserts that a value is a primitive string, returning it with non-string
 variants removed from the type.
 
 Uses `typeof === 'string'`. Boxed-string wrappers (`new String('x')`,
 `typeof === 'object'`) are intentionally rejected; production code should
 not be constructing boxed primitives, and accepting them would mask bugs.
 
 The return type uses {@link ExtractOrUnknown} so `unknown` inputs (e.g. parsed
 JSON, fetched API payloads, generic property access) narrow to `string`
 instead of collapsing to `never`.
 
 @param value - Value to assert as a primitive string
 
 @returns Same value with non-string variants excluded from the type
 
 @throws Error when value is not a primitive string
 
 @example
 ```ts
 const raw: unknown = JSON.parse(payload,);
 const greeting = stringOrThrow(raw,);
 // greeting is string
 
 const mixed: string | number = lookup();
 const text = stringOrThrow(mixed,);
 // text is string
 ```
 */
export function stringOrThrow<T,>(value: T,): ExtractOrUnknown<T, string> {
  if ((typeof value) !== 'string')
    throw new Error(`Expected string, got ${typeof value} ${formatUnknownValue(value,)}`,);
  // oxlint-disable-next-line typescript-eslint/no-unsafe-type-assertion -- TypeScript cannot statically narrow generic T after typeof
  return value as ExtractOrUnknown<T, string>;
}
