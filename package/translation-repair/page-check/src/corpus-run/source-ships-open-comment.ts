import type { TextRange, } from './source-ships-ranges.ts';

//region Source ships open comment
// A COMMENT THAT NEVER CLOSES, OPENED AT THE START OF A LINE, IS RAW HTML TO THE END OF THE TEXT to a reader of the source:
// the rest of the text is the block's content, so a quote mark, a label, a definition, a heading mark or a list marker in it
// is text, shown as it stands. The package's own parser hides everything after the four characters of such an opener, so the
// page cannot carry them, and the encoder writes the opener as a character reference (`source-ships-encoding.ts`). The parse
// that finds where the page needs rewriting is given the opener as it stands where it starts a block, so it reads the one
// raw HTML block a reader of the source reads, and the rest of the text is written as the text of such a block is. An opener
// after other text on its line is not a block start and is text to both readers, as one in a block quote or a list item is.

/**
 What opens an HTML comment.
 */
export const COMMENT_OPEN = '<!--';

/**
 How many spaces a line may open with before the opener still starts a block of raw HTML.
 */
const MOST_INDENT = 3;

/**
 Finds the first opener of a comment that never closes when it starts a block of raw HTML.

 @param text - whole text

 @param starts - index of the `<` of each opener of a comment that never closes, in text order

 @returns Index of the first opener when only spaces, at most three, stand before it on its line; -1 otherwise

 @example
 ```ts
 const start = blockOpenerStart({ text: '猫\n\n  <!-- a\n', starts: [5,], },);
 ```
 */
export function blockOpenerStart(
  {
    text,
    starts,
  }: {
    readonly text: string;
    readonly starts: readonly number[];
  },
): number {
  /**
   The first opener, absent when the list is empty.
   */
  const [first = -1,] = starts;
  if (first === (-1))
    return -1;
  /**
   Index of the first character of the opener's line.
   */
  const lineStart = (first === 0) ? 0 : (text.lastIndexOf(
    '\n',
    first - 1,
  ) + 1);
  if ((first - lineStart) > MOST_INDENT)
    return -1;
  for (let at = lineStart; at < first; at += 1) {
    if (text.charAt(at,) !== ' ')
      return -1;
  }
  return first;
}

/**
 What an opener of a comment that never closes is, in characters.
 */
const OPENER_LENGTH = COMMENT_OPEN.length;

/**
 Takes the characters of each opener out of the stretches that are written as raw text, since the opener is written whole
 as a character reference and a backslash before the rest of it would show.

 @param ranges - stretches written as raw text, in text order

 @param starts - index of the `<` of each opener of a comment that never closes, in text order

 @returns Stretches with the four characters of each opener cut out, the pieces in text order

 @example
 ```ts
 const cut = rangesWithoutOpeners({ ranges: [{ start: 0, end: 12, },], starts: [0,], },);
 ```
 */
export function rangesWithoutOpeners(
  {
    ranges,
    starts,
  }: {
    readonly ranges: readonly TextRange[];
    readonly starts: readonly number[];
  },
): readonly TextRange[] {
  return ranges.flatMap(function cut(range,): readonly TextRange[] {
    /**
     Pieces of the range between the openers it holds.
     */
    const pieces: TextRange[] = [];
    /**
     Where the piece being read starts.
     */
    const piece = { start: range.start, };
    for (const start of starts) {
      if ((start + OPENER_LENGTH) <= piece.start)
        continue;
      if (start >= range.end)
        break;
      if (start > piece.start) {
        pieces.push({
          start: piece.start,
          end: start,
        },);
      }
      piece.start = Math.max(
        piece.start,
        start + OPENER_LENGTH,
      );
    }
    if (piece.start < range.end) {
      pieces.push({
        start: piece.start,
        end: range.end,
      },);
    }
    return pieces;
  },);
}

/**
 Where each `<!--` of a text stands from an index on.

 @param text - text

 @param from - index to look from

 @returns Index of the `<` of each, in text order

 @example
 ```ts
 const starts = openerStarts({ text: '<!-- a <!-- b', from: 0, },);
 ```
 */
export function openerStarts(
  {
    text,
    from,
  }: {
    readonly text: string;
    readonly from: number;
  },
): readonly number[] {
  /**
   Indices found.
   */
  const starts: number[] = [];
  for (
    let at = text.indexOf(
      COMMENT_OPEN,
      from,
    );
    at !== (-1);
    at = text.indexOf(
      COMMENT_OPEN,
      at + COMMENT_OPEN.length,
    )
  )
    starts.push(at,);
  return starts;
}

//endregion Source ships open comment
