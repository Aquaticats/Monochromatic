/**
 Matches a replacement's line-ending edges to the span it replaces.

 A content span in the archive ends where its block ends, and the block
 separator after it belongs to the document, not to the span. A lane that
 returns its text with a trailing line ending (a writer closing its answer
 with a newline, which the deterministic floor has no reason to refuse) would
 have that ending written verbatim ahead of the separator, and the page would
 carry one more blank line than the archive did at that boundary. Class
 seventy-five (CuspariaKLSY2, 2026-09-21): a list-intro slice shipped
 "…transitioning:\n" and the page carried a double blank line between the
 intro and its list where the archive and the original both carry one.

 The rule: the written text carries exactly the leading and trailing
 line-ending runs the replaced span carried, whatever its lane wrapped it in.
 Inside the text nothing is touched, so hard breaks, verse lines and fenced
 blocks stay as the lane wrote them.

 @module
 */

//region Line-ending runs
/**
 Whether one character is a line-ending byte.

 @param character - one code unit

 @returns Whether it is CR or LF

 @example
 ```ts
 isLineEnding({ character: '\n', },) // true
 ```
 */
function isLineEnding({ character, }: { readonly character: string; },): boolean {
  return (character === '\n') || (character === '\r');
}

/**
 Run of line-ending characters at the start of a text.

 @param text - text to read

 @returns Leading CR and LF characters, possibly empty

 @example
 ```ts
 leadingLineEndings({ text: '\n\nThe cat.', },) // '\n\n'
 ```
 */
export function leadingLineEndings({ text, }: { readonly text: string; },): string {
  for (let index = 0; index < text.length; index += 1) {
    if (!isLineEnding({ character: text.charAt(index,), },)) {
      return text.slice(
        0,
        index,
      );
    }
  }
  return text;
}

/**
 Run of line-ending characters at the end of a text.

 @param text - text to read

 @returns Trailing CR and LF characters, possibly empty

 @example
 ```ts
 trailingLineEndings({ text: 'The cat.\n', },) // '\n'
 ```
 */
export function trailingLineEndings({ text, }: { readonly text: string; },): string {
  for (let index = text.length - 1; index >= 0; index -= 1) {
    if (!isLineEnding({ character: text.charAt(index,), },)) {
      return text.slice(index + 1,);
    }
  }
  return text;
}
//endregion Line-ending runs

//region Edge matching
/**
 Rewrites a replacement so its leading and trailing line-ending runs are the
 replaced span's own.

 @param replaced - text the span held in the document, whose edges are kept

 @param text - replacement its lane produced, whose edges are discarded

 @returns Replacement carrying the span's edges around its own interior

 @example
 ```ts
 matchSpanEdges({ replaced: 'The cat sleeps.', text: '\nThe cat naps.\n', },)
 // 'The cat naps.'
 ```
 */
export function matchSpanEdges(
  {
    replaced,
    text,
  }: {
    readonly replaced: string;
    readonly text: string;
  },
): string {
  /**
   Leading run the replacement itself carries, to be dropped.
   */
  const ownLeading = leadingLineEndings({ text, },);
  if (ownLeading === text) {
    // The replacement is nothing but line endings (or empty): the span's own
    // edges are all that remains, which for an ordinary block span is nothing,
    // the same deletion an empty replacement makes.
    return leadingLineEndings({ text: replaced, },)
      + trailingLineEndings({ text: replaced, },);
  }

  /**
   Trailing run the replacement itself carries, to be dropped.
   */
  const ownTrailing = trailingLineEndings({ text, },);

  /**
   The replacement between its own runs, untouched.
   */
  const interior = text.slice(
    ownLeading.length,
    text.length - ownTrailing.length,
  );
  return leadingLineEndings({ text: replaced, },)
    + interior
    + trailingLineEndings({ text: replaced, },);
}
//endregion Edge matching
