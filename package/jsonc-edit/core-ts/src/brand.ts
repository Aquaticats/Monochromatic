/**
 Branded type for a complete JSONC document string.
 
 The brand is a compile-time marker only; at runtime a `StringJsonc` is an
 ordinary string. It exists so callers cannot pass an arbitrary string where a
 value known to be JSONC is required, without an explicit assertion at the
 boundary. Defined locally rather than derived from the retired `module-es`
 string taxonomy.
 
 @example
 ```ts
 const source = '{ "a": 1 } // ok' as StringJsonc;
 ```
 */
export type StringJsonc = string & {
  readonly __brand: 'jsonc-document';
};

/**
 Branded type for a JSONC fragment: a substring of a document being parsed,
 such as the content remaining after a leading comment is stripped.
 
 Distinct brand from {@link StringJsonc} so a fragment is not mistaken for a
 whole document at a type boundary.
 
 @example
 ```ts
 const rest = '[1, 2, 3]' as FragmentStringJsonc;
 ```
 */
export type FragmentStringJsonc = string & {
  readonly __brand: 'jsonc-fragment';
};
