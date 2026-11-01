//region Readonly data views
// A recursively readonly VIEW of structural data this package borrows but does
// not own.
//
// `Readonly<T>` is shallow. `Readonly<{ children: Node[] }>` stops the property
// being reassigned and leaves the array and every element mutable, so a deep
// readonly checker still reports the nested writable path. That is why
// annotating a borrowed value with `Readonly` moves a finding rather than
// clearing it.
//
// These are VIEWS, not copies. A mutable external value is structurally
// assignable to a readonly projection of itself, so nothing is cloned at
// runtime and the external discriminated unions survive intact. That matters
// for mdast, whose consumers narrow on `node.type` across many variants.

/**
 Recursively exposes structural data through readonly properties.

 Intended for records, unions, arrays and tuples: plain parsed or borrowed
 data. Not for stateful class instances, whose methods and private state this
 would misdescribe.

 Homomorphic on purpose. Mapping over `keyof Value` preserves arrays and
 tuples as readonly arrays and readonly tuples rather than flattening them
 into objects keyed by index.

 @internal

 @example
 ```ts
 type Reading = DeepReadonlyData<{ entries: { id: string; }[]; }>;
 ```
 */
export type DeepReadonlyData<Value,> = Value extends object ? {
  readonly [Key in keyof Value]: DeepReadonlyData<Value[Key]>;
}
  : Value;

//endregion Readonly data views
