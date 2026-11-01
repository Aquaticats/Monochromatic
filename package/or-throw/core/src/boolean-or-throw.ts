/**
 `booleanOrThrow`: assert `typeof === 'boolean'`, return the narrowed value or throw.
 
 @module
 */

import { formatUnknownValue, } from './format-unknown-value.ts';

import type { ExtractOrUnknown, } from './extract-or-unknown.ts';

/**
 Asserts that a value is a primitive boolean, returning it with non-boolean
 variants removed from the type.
 
 Uses `typeof === 'boolean'`. Truthy and falsy values that are not literal
 `true` or `false` (`0`, `1`, `'true'`, `'false'`) are rejected; this is a
 type check, not a coercion. Boxed-boolean wrappers (`new Boolean(false)`,
 `typeof === 'object'`) are intentionally rejected.
 
 The return type uses {@link ExtractOrUnknown} so `unknown` inputs narrow to
 `boolean` instead of collapsing to `never`.
 
 @param value - Value to assert as a primitive boolean
 
 @returns Same value with non-boolean variants excluded from the type
 
 @throws Error when value is not a primitive boolean
 
 @example
 ```ts
 const flag: unknown = config.enabled;
 const enabled = booleanOrThrow(flag,);
 // enabled is boolean
 ```
 */
export function booleanOrThrow<T,>(value: T,): ExtractOrUnknown<T, boolean> {
  if ((typeof value) !== 'boolean')
    throw new Error(`Expected boolean, got ${typeof value} ${formatUnknownValue(value,)}`,);
  // oxlint-disable-next-line typescript-eslint/no-unsafe-type-assertion -- TypeScript cannot statically narrow generic T after typeof
  return value as ExtractOrUnknown<T, boolean>;
}
