/**
 Span splicing: applying one mutant to source text.
 
 @example
 ```ts
 spliceReplacement({ source: 'a + b', start: 2, end: 3, text: '-' });
 // 'a - b'
 ```
 */

/**
 Applies one replacement to source text by UTF-16 string slicing.
 
 yuku-parser's JS bindings return UTF-16 string offsets (probe-verified
 on multibyte and astral characters), so plain string slicing is exact;
 Buffer-based byte slicing at these offsets would corrupt output.
 
 @param options - Source text and replacement span.
 
 @returns Mutated source text.
 
 @throws Error when span is out of bounds or inverted.
 
 @example
 ```ts
 spliceReplacement({ source: 'true && x', start: 5, end: 7, text: '||' });
 // 'true || x'
 ```
 */
export function spliceReplacement(options: {
  readonly source: string;
  readonly start: number;
  readonly end: number;
  readonly text: string;
},): string {
  if ((options.start < 0)
    || (options.end
      > options.source
        .length)
    || (options.start > options.end))
    throw new Error(
      `replacement span ${String(options.start,)}..${String(options.end,)} out of bounds for source of length ${String(options.source
        .length,)}`,
    );

  return `${options.source
    .slice(
    0,
    options.start,
  )}${options.text}${options.source
    .slice(options.end,)}`;
}
