/**
 Shared structural type helpers for the page-weight pipeline.
 */

/**
 Recursively marks every property and array element of `T` readonly.
 
 only when it is deeply readonly. The hast nodes this package walks are plain
 data trees (no methods), so a homomorphic deep map describes them without
 dropping any field the walkers read; passing a mutable hast node into a
 `DeepReadonly`-typed parameter is sound because the body only reads.
 
 @typeParam T - value type to deeply freeze at the type level
 */
export type DeepReadonly<T,> = T extends readonly (infer U)[]
  ? readonly DeepReadonly<U>[]
  : T extends object ? { readonly [K in keyof T]: DeepReadonly<T[K]>; }
    : T;
