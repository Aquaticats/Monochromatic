/**
 `symbolOrThrow`: assert `typeof === 'symbol'`, return the narrowed value or throw.
 
 @module
 */

import { formatUnknownValue, } from './format-unknown-value.ts';

import type { ExtractOrUnknown, } from './extract-or-unknown.ts';

/**
 Asserts that a value is a symbol, returning it with non-symbol variants
 removed from the type.
 
 Uses `typeof === 'symbol'`. Both registered (`Symbol.for('key')`) and
 unregistered (`Symbol('desc')`) symbols pass.
 
 The return type uses {@link ExtractOrUnknown} so `unknown` inputs narrow to
 `symbol` instead of collapsing to `never`.
 
 @param value - Value to assert as a symbol
 
 @returns Same value with non-symbol variants excluded from the type
 
 @throws Error when value is not a symbol
 
 @example
 ```ts
 const key: unknown = lookupKey();
 const sym = symbolOrThrow(key,);
 // sym is symbol; safe to use as a property key
 ```
 */
export function symbolOrThrow<T,>(value: T,): ExtractOrUnknown<T, symbol> {
  if ((typeof value) !== 'symbol')
    throw new Error(`Expected symbol, got ${typeof value} ${formatUnknownValue(value,)}`,);
  // oxlint-disable-next-line typescript-eslint/no-unsafe-type-assertion -- TypeScript cannot statically narrow generic T after typeof
  return value as ExtractOrUnknown<T, symbol>;
}
