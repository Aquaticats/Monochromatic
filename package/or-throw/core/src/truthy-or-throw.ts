/**
 `truthyOrThrow`: assert truthy, return the narrowed value or throw.
 
 @module
 */

import { formatUnknownValue, } from './format-unknown-value.ts';

import type { Falsy, } from './falsy.ts';

/**
 Asserts that a value is truthy, returning it with falsy variants removed from the type.
 
 The runtime check is `!value`, which catches every falsy value JavaScript recognizes:
 `false`, `0`, `0n`, `''`, `null`, `undefined`, and `NaN`. The static return type is
 `Exclude<T, Falsy>` (see {@link Falsy} for the excluded union); this narrows unions
 cleanly (`string | undefined` becomes `string`)
 but cannot exclude `NaN` from a wide `number` (TypeScript has no `NaN` literal type),
 so `truthyOrThrow<number>(NaN,)` throws at runtime while still typing as `number`.
 
 @param value - Value to assert as truthy
 
 @returns Same value with falsy variants excluded from the type
 
 @throws Error when value is falsy
 
 @example
 Required string field:
 ```ts
 const name = truthyOrThrow(user.displayName,);
 // name is non-empty string (empty string was excluded)
 ```
 
 @example
 Required positive count:
 ```ts
 const count = truthyOrThrow(parsedCount,);
 // count is non-zero (0 was excluded)
 ```
 
 @example
 Truthy union narrowing:
 ```ts
 const value: string | undefined = lookup();
 const present = truthyOrThrow(value,);
 // present is string (undefined and '' both excluded)
 ```
 */
export function truthyOrThrow<T,>(value: T,): Exclude<T, Falsy> {
  // oxlint-disable-next-line typescript/strict-boolean-expressions -- intentional truthiness check on generic T; the rule's narrower-type preference defeats the purpose of this assertion
  if (!value)
    throw new Error(`Expected truthy value, got ${formatUnknownValue(value,)}`,);
  // oxlint-disable-next-line typescript-eslint/no-unsafe-type-assertion -- TypeScript cannot statically narrow generic T via `if (!value)`, so the cast bridges the runtime check to the documented return type
  return value as Exclude<T, Falsy>;
}
