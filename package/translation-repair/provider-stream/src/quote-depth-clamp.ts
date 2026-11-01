//region Quote depth clamp
// TIANQICHEN66621 SLICE 16, 2026-09-28. An editor's patch resolved six
// accepted issues and left the last quoted line as `> >`, a quote inside a
// quote that neither the original nor the archive has; the consolidation
// declined its slate for that standing, and the page shipped the nesting.
// Over every artifact ten repair-lane slices nested a quote two levels deep
// where neither side nests, and every one shipped; the translate lane did it
// none (`nested-quote-census.mjs`).
//
// CLAMPED, NOT REFUSED. Refusing the edit would throw away the fixes it
// carries for a marker the editor mistyped, so the replacement's lines are
// brought back to the depth their context allows, deterministically, before
// any gate reads them, as the typography restoration is.
//
// ONE LEVEL IS ALWAYS ALLOWED. This archive renders speech the original sets
// in 「」 as a blockquote, so an edit quoting a line is a rendering choice; a
// SECOND level is a quote inside a quote, which only the original's own
// nesting, or nesting the replaced text already has, licenses.

/**
 Most spaces a blockquote marker may be indented by.
 */
const MAX_MARKER_INDENT = 3;

/**
 How deep one line is quoted, and where its markers end.

 @param line - one line, without its line ending

 @returns Marker count, and the index of the first character after the markers
 and the one space each may take

 @example
 ```ts
 quotePrefix({ line: '> > She purred.', },); // { depth: 2, end: 4 }
 ```
 */
function quotePrefix({ line, }: { readonly line: string; },): {
  readonly depth: number;
  readonly end: number
} {
  /**
   Cursor over the line and the markers read so far.
   */
  const cursor = {
    at: 0,
    depth: 0,
  };
  for (;;) {
    /**
     Where the next marker would start after its indentation.
     */
    const indented = cursor.at + (line.slice(
      cursor.at,
      cursor.at + MAX_MARKER_INDENT,
    )
      .length
      - line.slice(
        cursor.at,
        cursor.at + MAX_MARKER_INDENT,
      )
        .trimStart()
        .length);
    if (line[indented] !== '>')
      return {
        depth: cursor.depth,
        end: cursor.at,
      };
    cursor.depth += 1;
    cursor.at = indented + ((line[indented + 1] === ' ') ? 2 : 1);
  }
}

/**
 Deepest quote any line of a text opens with.

 @param text - text read line by line

 @returns Deepest marker count, zero for unquoted text

 @example
 ```ts
 quoteDepth({ text: '> A.\n>\n> > B.', },); // 2
 ```
 */
export function quoteDepth({ text, }: { readonly text: string; },): number {
  return text.split('\n',)
    .reduce(
    function deepest(
      most,
      line,
    ): number {
      return Math.max(
        most,
        quotePrefix({ line, },)
          .depth,
      );
    },
    0,
    );
}

/**
 Deepest quote an edit may write in its region: one level always, the
 nesting of every whole line the region touches, and the original's.

 WHOLE LINES, read from their starts. A region may start mid-line, and its
 replaced text's first characters are then words, not markers: read as a line
 of its own, a region starting at an emoticon like `>>_<<` would itself allow
 two levels.

 @param targetText - whole text the region sits in

 @param startOffset - where the region starts

 @param endOffset - where the region ends

 @param sourceText - the original, empty where the caller has none

 @returns Deepest allowed marker count

 @example
 ```ts
 const bound = quoteDepthBound({ targetText, startOffset, endOffset, sourceText, },);
 ```
 */
export function quoteDepthBound(
  {
    targetText,
    startOffset,
    endOffset,
    sourceText,
  }: {
    readonly targetText: string;
    readonly startOffset: number;
    readonly endOffset: number;
    readonly sourceText: string;
  },
): number {
  /**
   Start of the line the region starts in.
   */
  const lineStart = targetText.lastIndexOf(
    '\n',
    startOffset - 1,
  ) + 1;

  /**
   End of the line the region ends in.
   */
  const lineEnd = targetText.indexOf(
    '\n',
    endOffset,
  );
  return Math.max(
    1,
    quoteDepth({ text: targetText.slice(
      lineStart,
      (lineEnd === (-1)) ? targetText.length : lineEnd,
    ), },),
    quoteDepth({ text: sourceText, },),
  );
}

/**
 A replacement whose lines quote no deeper than the bound, each over-deep
 line's markers rewritten to the bound in the plain `> ` form.

 @param replacement - an edit's new text for its region

 @param bound - deepest allowed marker count, from {@link quoteDepthBound}

 @param startsLine - whether the region starts at a line start; when it does
   not, the replacement's first line continues a line whose markers lie
   outside it

 @returns The replacement with over-deep lines clamped

 @example
 ```ts
 clampQuoteDepth({ replacement: 'A.\n> > B.', bound: 1, startsLine: false, },); // 'A.\n> B.'
 ```
 */
export function clampQuoteDepth(
  {
    replacement,
    bound,
    startsLine,
  }: {
    readonly replacement: string;
    readonly bound: number;
    readonly startsLine: boolean;
  },
): string {
  return replacement
    .split('\n',)
    .map(function clampLine(
      line,
      index,
    ): string {
      if ((index === 0) && (!startsLine))
        return line;

      /**
       The line's markers and where they end.
       */
      const prefix = quotePrefix({ line, },);
      if (prefix.depth <= bound)
        return line;

      /**
       The line's words after its markers.
       */
      const rest = line.slice(prefix.end,);

      /**
       Markers at the allowed depth.
       */
      const markers = Array.from(
        { length: bound, },
        function marker(): string {
          return '>';
        },
      )
        .join(' ',);
      return (rest === '') ? markers : `${markers} ${rest}`;
    },)
    .join('\n',);
}

//endregion Quote depth clamp
