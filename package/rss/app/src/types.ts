/**
 Shared structural type helpers for the RSS pipeline.
 */

/**
 Recursively marks every property and array element of `T` readonly.
 
 only when it is deeply readonly. The feed, outline, and item values this
 package threads through its pipeline are plain parsed data, so a homomorphic
 deep map describes them without dropping any field the renderers read;
 passing a mutable value into a `DeepReadonly`-typed parameter is sound
 because every consumer only reads. `Date` short-circuits first so its
 instance methods stay callable and the rule's allow-list still recognises
 it; the function branch (contravariant `never[]` matches any signature,
 including methods carrying typed parameters such as `Dirent.isFile`)
 preserves callable members instead of mangling them into property bags.
 
 @typeParam T - value type to deeply freeze at the type level
 
 @example
 ```ts
 function render(items: readonly DeepReadonly<Item>[]): string { ... }
 ```
 */
export type DeepReadonly<T,> = T extends Date ? T
  : T extends (...args: readonly never[]) => unknown ? T
    : T extends readonly (infer U)[] ? readonly DeepReadonly<U>[]
      : T extends object ? { readonly [K in keyof T]: DeepReadonly<T[K]>; }
        : T;
