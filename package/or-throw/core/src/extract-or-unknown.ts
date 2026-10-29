/**
 Type-level `Extract` variant that handles `unknown` inputs.
 
 @module
 */

/**
 Variant of `Extract` that returns the target type `U` when the input `T`
 is `unknown`, instead of collapsing to `never`.
 
 Plain `Extract<unknown, string>` evaluates to `never`, because `unknown`
 does not extend `string`. That defeat is silent: a helper typed as
 `<T,>(value: T,): Extract<T, string>` returns `never` for the most common
 call shape (a value of type `unknown` from `JSON.parse`, a fetched API
 payload, generic property access), forcing callers to write awkward casts
 or losing the narrowing entirely. `ExtractOrUnknown` recovers `U` in that
 case while preserving the discriminating behavior of `Extract` for
 narrower input types.
 
 @example
 ```ts
 import type { ExtractOrUnknown, } from '\@monochromatic-dev/or-throw-core';
 
 type A = ExtractOrUnknown<unknown, string>;          // string
 type B = ExtractOrUnknown<string | number, string>;  // string
 type C = ExtractOrUnknown<string, string>;           // string
 type D = ExtractOrUnknown<number, string>;           // never
 ```
 */
export type ExtractOrUnknown<T, U,> = unknown extends T ? U : Extract<T, U>;
