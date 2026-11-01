import {
  MARK_RANGE_LINES,
  WIDE_RANGE_LINES,
} from './east-asian-width-table.test-fixture.ts';
import {
  inRanges,
  rangesOfLines,
} from './east-asian-width-parse.test-fixture.ts';

//region Display columns
// HOW MANY COLUMNS A LINE OF TEXT FILLS (ledger B1376, B1413). A character of East_Asian_Width Wide or Fullwidth fills two
// columns, a mark fills none of its own, the zero width joiner fills none, and every other character fills one. The
// classes come from the table generated from Unicode's `EastAsianWidth.txt` (`east-asian-width-table.test-fixture.ts`),
// with no pattern and no guess about which blocks the docs hold.
//
// THE LIMITS. Characters are counted one code point at a time, as `wcwidth` counts them, and not by grapheme cluster: an
// emoji sequence joined by the zero width joiner counts its emoji (two columns each) where a terminal draws one glyph, and
// a code point whose width a terminal decides by its font (the East_Asian_Width values A, H, Na and N) counts one. A
// regional indicator is Neutral in the file, so a flag pair counts two columns, which is what it renders in.

/**
 Ranges of code points that fill two columns, read once from the generated table.
 */
const TWO_COLUMN_RANGES = rangesOfLines({ lines: WIDE_RANGE_LINES, },);

/**
 Ranges of code points that are marks, read once from the generated table.
 */
const MARK_RANGES = rangesOfLines({ lines: MARK_RANGE_LINES, },);

/**
 The zero width joiner, a format character of no width that joins the emoji of a sequence.
 */
const ZERO_WIDTH_JOINER = 0x20_0D;

/**
 Columns one code point fills.

 @param codePoint - code point to measure

 @returns None for a mark or the zero width joiner, two for East_Asian_Width Wide or Fullwidth, one for the rest

 @example
 ```ts
 const columns = columnsOfCodePoint({ codePoint: 0x23_29, },);
 ```
 */
export function columnsOfCodePoint({ codePoint, }: { readonly codePoint: number; },): number {
  if ((codePoint === ZERO_WIDTH_JOINER) || inRanges({
    ranges: MARK_RANGES,
    codePoint,
  },))
    return 0;
  return inRanges({
    ranges: TWO_COLUMN_RANGES,
    codePoint,
  },) ? 2 : 1;
}

/**
 Number of display columns of a line.

 @param line - one line of text

 @returns Columns the line fills: two for each wide character, none for a mark or a joiner, one for the rest

 @example
 ```ts
 // Three narrow characters fill three columns, and each wide one fills two.
 const width = displayColumnsOf({ line: 'cat', },);
 const wideWidth = displayColumnsOf({ line: '猫猫', },);
 ```
 */
export function displayColumnsOf({ line, }: { readonly line: string; },): number {
  /**
   Columns counted so far.
   */
  let width = 0;
  for (const character of line) {
    width += columnsOfCodePoint({ codePoint: character.codePointAt(0,) ?? 0, },);
  }
  return width;
}

//endregion Display columns
