/**
 Shared `Falsy` type alias used by {@link truthyOrThrow} and {@link falsyOrThrow}
 to express the values that coerce to `false` in a boolean context.
 
 @module
 */

/* oxlint-disable no-restricted-syntax/no-nullish-union, no-restricted-syntax/no-optional-escape -- this type enumerates JavaScript's falsy set as its domain definition, not as a fake-optional encoding; `false`, `0`, `''`, `null`, and `undefined` are the subject matter, so the union is incomplete without them */
/**
 Union of the values that JavaScript evaluates as `false` in a boolean context.
 
 `NaN` is falsy at runtime but cannot be represented as a TypeScript literal type,
 so it is not in this union. Helpers in this package check for falsiness at runtime
 (via `!value` or its negation), which catches `NaN`; the static type just cannot
 exclude `NaN` from a `number`.
 
 @example
 ```ts
 import type { Falsy, } from '\@monochromatic-dev/or-throw-core';
 
 function isFalsy(value: unknown,): value is Falsy {
   return !value;
 }
 ```
 */
export type Falsy = false | 0 | 0n | '' | null | undefined;
/* oxlint-enable no-restricted-syntax/no-nullish-union, no-restricted-syntax/no-optional-escape */
