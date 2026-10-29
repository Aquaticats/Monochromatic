/**
 `falsyOrThrow`: assert falsy, return the narrowed value or throw.
 
 @module
 */

import { formatUnknownValue, } from './format-unknown-value.ts';

import type { ExtractOrUnknown, } from './extract-or-unknown.ts';
import type { Falsy, } from './falsy.ts';

/**
 Asserts that a value is falsy, returning it with truthy variants removed from the type.
 
 The runtime check is `value` (truthy in a boolean context), which inverts the truthy
 check used by {@link truthyOrThrow}. The static return type narrows unions cleanly
 (`string | undefined` becomes `undefined`) and recovers {@link Falsy} from an `unknown`
 input (via {@link ExtractOrUnknown}, avoiding plain `Extract<unknown, Falsy>` = `never`).
 `NaN` cannot be extracted from `number` because TypeScript has no `NaN` literal type.
 
 Use this when an invariant requires an absent or zero-shaped value: e.g. asserting
 that a sentinel slot is empty before populating it, or that a cleared field has been
 reset to its falsy default.
 
 @param value - Value to assert as falsy
 
 @returns Same value with truthy variants excluded from the type
 
 @throws Error when value is truthy
 
 @example
 Empty-slot invariant:
 ```ts
 falsyOrThrow(slot.error,);
 // slot.error confirmed null | undefined | '' etc.
 ```
 
 @example
 Union narrowing to the falsy side:
 ```ts
 const value: string | undefined = lookup();
 const cleared = falsyOrThrow(value,);
 // cleared is undefined (string excluded; '' would survive but other strings throw)
 ```
 */
export function falsyOrThrow<T,>(value: T,): ExtractOrUnknown<T, Falsy> {
  // oxlint-disable-next-line typescript/strict-boolean-expressions -- intentional truthiness check on generic T; the rule's narrower-type preference defeats the purpose of this assertion
  if (value)
    throw new Error(`Expected falsy value, got ${formatUnknownValue(value,)}`,);
  // oxlint-disable-next-line typescript-eslint/no-unsafe-type-assertion -- TypeScript cannot statically narrow generic T via `if (value)`, so the cast bridges the runtime check to the documented return type
  return value as ExtractOrUnknown<T, Falsy>;
}
