/**
 Branded CSS document source string.
 
 The brand is a compile-time marker only; at runtime a `StringCss` is an
 ordinary string. Callers assert the brand at the boundary where they know a
 string holds CSS, so downstream signatures can demand proven-CSS input
 instead of any string.
 
 @example
 ```ts
 const source = '.btn { color: red; }' as StringCss;
 ```
 */
export type StringCss = string & {
  readonly __brand: 'css-document';
};

/**
 Brands arbitrary text as CSS source at a boundary where the caller vouches
 for it: file contents, bundler-inlined strings, user-provided text.
 Runtime identity; `parseCss` performs the real validation and throws on
 text that is not CSS.
 
 @param source - Text the caller asserts holds CSS.
 
 @returns Same string, branded.
 
 @example
 ```ts
 parseCss({ source: asCssSource('.a { top: 0; }',), },);
 ```
 */
export function asCssSource(source: string,): StringCss {
  // oxlint-disable-next-line typescript/no-unsafe-type-assertion -- boundary branding; parseCss validates for real
  return source as StringCss;
}
