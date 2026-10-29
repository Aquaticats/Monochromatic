/**
 Text splices on an original source string.

 Every catalog edit is computed against one parse of the untouched source and
 applied as a range replacement, so text outside the edited ranges (comments,
 spacing, key order) stays byte-identical.

 @module
 */

//region Types

/**
 Replace `source.slice(start, end)` with `text`.

 @example
 ```ts
 const splice: Splice = { start: 10, end: 13, text: '">=1"', };
 ```
 */
export type Splice = {
  readonly start: number;
  readonly end: number;
  readonly text: string;
};

//endregion Types

//region Positions

/**
 Offset of the first character of the line holding `offset`.

 @param source - full text

 @param offset - any offset on the line

 @returns offset just after the previous newline, or 0

 @example
 ```ts
 lineStart({ source: 'a\nbc', offset: 3, }); // 2
 ```
 */
export function lineStart({ source, offset, }: { readonly source: string; readonly offset: number; },): number {
  return source.lastIndexOf('\n', offset - 1,) + 1;
}

/**
 Offset just past the newline ending the line holding `offset`.

 @param source - full text

 @param offset - any offset on the line

 @returns offset after the line's newline, or the text length on the last line

 @example
 ```ts
 lineEndInclusive({ source: 'a\nbc', offset: 0, }); // 2
 ```
 */
export function lineEndInclusive({ source, offset, }: { readonly source: string; readonly offset: number; },): number {
  /**
   Position of the newline ending this line.
   */
  const newline = source.indexOf('\n', offset,);
  return newline === -1 ? source.length : newline + 1;
}

/**
 Start of the run of comment-only lines directly above the line holding `offset`.

 @param source - full text

 @param offset - offset on the line whose leading comments are wanted

 @returns start of the first contiguous `#` line above, or the line's own start when there is none

 @example
 ```ts
 leadingCommentStart({ source: '# why\nx = 1\n', offset: 6, }); // 0
 ```
 */
export function leadingCommentStart({ source, offset, }: { readonly source: string; readonly offset: number; },): number {
  /**
   Start of the line currently being extended upward.
   */
  let start = lineStart({ source, offset, },);
  while (start > 0) {
    /**
     Start of the line above.
     */
    const above = lineStart({ source, offset: start - 1, },);
    if (!source.slice(above, start,).trimStart().startsWith('#',))
      break;
    start = above;
  }
  return start;
}

//endregion Positions

//region Apply

/**
 Applies non-overlapping splices to `source`.

 @param source - original text the splice offsets refer to

 @param splices - replacements in any order

 @returns text with every splice applied

 @throws RangeError when two splices overlap, which would make the result depend on application order

 @example
 ```ts
 applySplices({ source: 'a = "1"', splices: [{ start: 4, end: 7, text: '">=1"', },], }); // 'a = ">=1"'
 ```
 */
export function applySplices({ source, splices, }: { readonly source: string; readonly splices: readonly Splice[]; },): string {
  /**
   Splices from last to first, so earlier offsets stay valid while applying.
   */
  const ordered = splices.toSorted(function byStartDescending(left, right,): number {
    return (right.start - left.start) || (right.end - left.end);
  },);
  /**
   Text built from the end backwards.
   */
  let result = source;
  /**
   Start of the splice applied just before, to detect overlaps.
   */
  let previousStart = Number.POSITIVE_INFINITY;
  for (const splice of ordered) {
    if (splice.end > previousStart) {
      throw new RangeError(
        `Overlapping catalog edits at offsets ${String(splice.start,)}..${String(splice.end,)} and ${String(previousStart,)}; this is a file-enforcer bug.`,
      );
    }
    result = `${result.slice(0, splice.start,)}${splice.text}${result.slice(splice.end,)}`;
    previousStart = splice.start;
  }
  return result;
}

//endregion Apply
