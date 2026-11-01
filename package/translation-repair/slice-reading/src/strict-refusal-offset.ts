import {
  maskHtmlComments,
  maskInvisibleLines,
  parseMdxBody,
  requireMdxRefusal,
  splitFrontMatter,
  withMdxExhaustionRefused,
} from '@monochromatic-dev/translation-repair-document-structure/ts';

//region Strict refusal offset
// WHERE THE STRICT GRAMMAR STOPPED, as an offset into the whole text. The
// assembly guard's one-slice counterfactual proves a repair only when one
// withdrawal leaves the page whole; a page with two breaks (XingZ607: two lone
// closing tags in two slices) fails every single withdrawal and the guard
// withdrew all 88 replacements. The parser names the first break's line and
// column, so a withdrawal that moves the first refusal later is progress the
// guard can act on one slice at a time. Read the way `parseDocument` reads:
// front matter split away, invisible lines and comments masked at their
// length, so the offset indexes the text the guard spliced.

/**
 Offset of the first character on a one-based line and column.

 @param text - text the line and column index

 @param line - one-based line

 @param column - one-based column

 @returns Offset into the text, clamped to its end

 @example
 ```ts
 const at = offsetOf({ text, line: 3, column: 2, },);
 ```
 */
function offsetOf(
  {
    text,
    line,
    column,
  }: {
    readonly text: string;
    readonly line: number;
    readonly column: number;
  },
): number {
  /**
   Offset where the named line starts: the lines before it, each with its
   line break.
   */
  const lineStart = text
    .split('\n',)
    .slice(
    0,
    line - 1,
    )
    .reduce(
    function pastLine(
      sum: number,
      earlier: string,
    ): number {
        /**
         Characters of the earlier line, with its line break.
         */
      const withBreak = earlier.length + 1;
      return sum + withBreak;
    },
    0,
    );
  /**
   Characters into that line.
   */
  const intoLine = column - 1;
  /**
   Offset the line and column name.
   */
  const at = lineStart + intoLine;
  return Math.min(
    at,
    text.length,
  );
}

/**
 What the strict grammar made of a document: accepted, or refused at an
 offset.

 @example
 ```ts
 const reading: StrictParseReading = { refused: true, offset: 458, };
 ```
 */
export type StrictParseReading =
  | {
    /**
     The grammar accepted the whole document.
     */
    readonly refused: false;
  }
    | {
    /**
     The grammar stopped.
     */
      readonly refused: true;

    /**
     Offset into the whole text where it stopped.
     */
      readonly offset: number;
    };

/**
 Where the strict MDX grammar first refuses a document, if anywhere.

 @param text - whole document, front matter included when present

 @returns Acceptance, or the offset of the first refusal

 @example
 ```ts
 const reading = strictRefusalOffset({ text: assembledText, },);
 ```
 */
export function strictRefusalOffset(
  { text, }: { readonly text: string; },
): StrictParseReading {
  /**
   Front matter split with body offset for absolute anchoring.
   */
  const split = splitFrontMatter({ text, },);
  /**
   Body with invisible lines blanked, as the document reader reads it.
   */
  const { masked: unwelded, } = maskInvisibleLines({ text: split.body, },);
  /**
   Body with comments blanked too, at the same length.
   */
  const { masked, } = maskHtmlComments({ text: unwelded, },);
  try {
    withMdxExhaustionRefused({
      parse: parseMdxBody,
      input: { body: masked, },
    },);
    return { refused: false, };
  }
  catch (error) {
    /**
     The grammar's refusal, which names where it stopped when the parser did.
     */
    const refusal = requireMdxRefusal({ error, },);
    // NO PLACE MEANS THE END OF THE BODY (ledger B86): the one refusal the
    // parser names no place for is an element still open when the document
    // ended (`mdast-util-mdx-jsx` `onErrorRightIsTag` with nothing left to
    // close), so the grammar read the whole body before it stopped. Read as the
    // body's start, it ranked before every other break on the page.
    if ((refusal.line === undefined) || (refusal.column === undefined))
      return {
        refused: true,
        offset: split.bodyOffset + masked.length,
      };
    return {
      refused: true,
      offset: split.bodyOffset + offsetOf({
        text: masked,
        line: refusal.line,
        column: refusal.column,
      },),
    };
  }
}

//endregion Strict refusal offset
