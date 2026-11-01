//region East Asian Width parse
// THE READING OF UNICODE'S `EastAsianWidth.txt` INTO THE RANGES OF CODE POINTS THAT FILL TWO COLUMNS OR NONE (ledger B1413).
// The file is a normative data file of the Unicode Character Database
// (`https://www.unicode.org/Public/<version>/ucd/EastAsianWidth.txt`), and one line of it reads
// `2329           ; W  # Ps       LEFT-POINTING ANGLE BRACKET`: a code point or a range of them, a semicolon, the
// East_Asian_Width value, and after the number sign the General_Category and the name. A character of the value W (Wide)
// or F (Fullwidth) fills two columns (UAX 11), and a character of General_Category M (Mn, Mc, Me) fills none of its own.
//
// NO PATTERN IS USED. A line is cut at the number sign, the semicolon and the two dots with string methods.

/**
 A run of code points from one to another, both included.
 */
export type CodeRange = {
  /**
   First code point of the run.
   */
  readonly from: number;

  /**
   Last code point of the run.
   */
  readonly to: number;
};

/**
 What the file gives: the Unicode version it names and the two sets of ranges.
 */
export type EastAsianWidthTable = {
  /**
   Version the first line of the file names, such as `17.0.0`.
   */
  readonly version: string;

  /**
   Ranges of code points of East_Asian_Width W or F, in order and without overlap.
   */
  readonly wide: readonly CodeRange[];

  /**
   Ranges of code points whose General_Category is a mark, in order and without overlap.
   */
  readonly marks: readonly CodeRange[];
};

/**
 Radix of the code points the file writes.
 */
const HEX_RADIX = 16;

/**
 Digits a range is written with in the generated table at least, so every range lines up.
 */
const RANGE_DIGITS = 4;

/**
 The East_Asian_Width values that fill two columns: Wide and Fullwidth.
 */
const TWO_COLUMN_VALUES: ReadonlySet<string> = new Set([
  'W',
  'F',
]);

/**
 The first letter of the General_Category of a mark.
 */
const MARK_CATEGORY_LETTER = 'M';

/**
 Blocks whose UNASSIGNED code points default to W, which the file's header states and its lines do not list:
 CJK Unified Ideographs Extension A, CJK Unified Ideographs, CJK Compatibility Ideographs, and the undesignated code
 points of Planes 2 and 3 (`EastAsianWidth.txt`, the header paragraph that begins "The unassigned code points in the
 following blocks default to W").
 */
const DEFAULT_WIDE_BLOCKS: readonly CodeRange[] = [
  {
    from: 0x34_00,
    to: 0x4D_BF,
  },
  {
    from: 0x4E_00,
    to: 0x9F_FF,
  },
  {
    from: 0xF9_00,
    to: 0xFA_FF,
  },
  {
    from: 0x2_00_00,
    to: 0x2_FF_FD,
  },
  {
    from: 0x3_00_00,
    to: 0x3_FF_FD,
  },
];

/**
 Prefix of the first line of the file, before the version.
 */
const NAME_PREFIX = '# EastAsianWidth-';

/**
 Suffix of the first line of the file, after the version.
 */
const NAME_SUFFIX = '.txt';

/**
 Merges ranges that overlap or touch.

 @param ranges - ranges in any order

 @returns Ranges in order, none overlapping or adjacent to another

 @example
 ```ts
 const merged = mergeRanges({ ranges: [{ from: 1, to: 2, }, { from: 3, to: 4, },], },);
 ```
 */
export function mergeRanges({ ranges, }: { readonly ranges: readonly CodeRange[]; },): readonly CodeRange[] {
  /**
   Ranges closed so far.
   */
  const merged: CodeRange[] = [];
  for (const range of ranges.toSorted(function byStart(
    left,
    right,
  ): number {
    return left.from - right.from;
  },)) {
    /**
     The range last closed.
     */
    const last = merged.at(-1,);
    if ((last !== undefined) && (range.from <= (last.to + 1))) {
      merged[merged.length - 1] = {
        from: last.from,
        to: Math.max(
          last.to,
          range.to,
        ),
      };
    }
    else {
      merged.push(range,);
    }
  }
  return merged;
}

/**
 Reads the range a line of the file names in its first field.

 @param field - text before the semicolon, such as `2329` or `231A..231B`

 @returns Range of the field

 @example
 ```ts
 const range = rangeOfField({ field: '231A..231B', },);
 ```
 */
function rangeOfField({ field, }: { readonly field: string; },): CodeRange {
  /**
   The two ends, which are one where the line names a single code point.
   */
  const [first = '', last = first,] = field.trim()
    .split('..',);
  return {
    from: Number.parseInt(
      first,
      HEX_RADIX,
    ),
    to: Number.parseInt(
      last,
      HEX_RADIX,
    ),
  };
}

/**
 Reads the file into the ranges of two columns and of marks.

 @param text - whole text of `EastAsianWidth.txt`

 @returns Version, ranges of East_Asian_Width W or F with the defaults the header states, and ranges of marks

 @throws {@link Error} when the first line does not name the file and its version

 @example
 ```ts
 const table = parseEastAsianWidth({ text, },);
 ```
 */
export function parseEastAsianWidth({ text, }: { readonly text: string; },): EastAsianWidthTable {
  /**
   The lines of the file.
   */
  const lines = text.split('\n',);
  /**
   The first line, which names the file and its version.
   */
  const [heading = '',] = lines;
  if ((!heading.startsWith(NAME_PREFIX,)) || (!heading.endsWith(NAME_SUFFIX,))) {
    throw new Error(
      `the first line of the file is "${heading}", not "${NAME_PREFIX}<version>${NAME_SUFFIX}": this is not EastAsianWidth.txt`,
    );
  }
  /**
   Ranges of two columns found so far.
   */
  const wide: CodeRange[] = [...DEFAULT_WIDE_BLOCKS,];
  /**
   Ranges of marks found so far.
   */
  const marks: CodeRange[] = [];
  for (const line of lines) {
    if (line.startsWith('#',) || (line.trim() === ''))
      continue;
    /**
     The data and the comment of the line.
     */
    const [data = '', comment = '',] = line.split('#',);
    /**
     The range and the East_Asian_Width value.
     */
    const [field = '', value = '',] = data.split(';',);
    /**
     The range the line names.
     */
    const range = rangeOfField({ field, },);
    if (TWO_COLUMN_VALUES.has(value.trim(),))
      wide.push(range,);
    /**
     The General_Category, the first word of the comment.
     */
    const [category = '',] = comment.trim()
      .split(' ',);
    if (category.startsWith(MARK_CATEGORY_LETTER,))
      marks.push(range,);
  }
  return {
    version: heading.slice(
      NAME_PREFIX.length,
      -NAME_SUFFIX.length,
    ),
    wide: mergeRanges({ ranges: wide, },),
    marks: mergeRanges({ ranges: marks, },),
  };
}

/**
 Writes a range as its first and last code point in hexadecimal, uppercase, joined by a hyphen.

 @param range - range to write

 @returns Text such as `231A-231B`

 @example
 ```ts
 const text = textOfRange({ range: { from: 0x23_1A, to: 0x23_1B, }, },);
 ```
 */
export function textOfRange({ range, }: { readonly range: CodeRange; },): string {
  return [
    range.from,
    range.to,
  ].map(function hexOf(codePoint,): string {
    return codePoint.toString(HEX_RADIX,)
      .toUpperCase()
      .padStart(
      RANGE_DIGITS,
      '0',
      );
  },)
    .join('-',);
}

/**
 Reads ranges back from the text `textOfRange` writes, one or more to a line, separated by spaces.

 @param lines - text lines of ranges

 @returns Ranges in the order written

 @example
 ```ts
 const ranges = rangesOfLines({ lines: ['231A-231B 2329-232A',], },);
 ```
 */
export function rangesOfLines({ lines, }: { readonly lines: readonly string[]; },): readonly CodeRange[] {
  return lines.flatMap(function rangesOfLine(line,): readonly CodeRange[] {
    return line.split(' ',)
      .filter(function isRange(word,): boolean {
        return word !== '';
      },)
      .map(function rangeOf(word,): CodeRange {
        /**
         The two ends.
         */
        const [first = '', last = '',] = word.split('-',);
        return {
          from: Number.parseInt(
            first,
            HEX_RADIX,
          ),
          to: Number.parseInt(
            last,
            HEX_RADIX,
          ),
        };
      },);
  },);
}

/**
 Whether a code point lies in one of the ranges.

 @param ranges - ranges in order, none overlapping

 @param codePoint - code point to find

 @returns True when a range holds it; the search halves the ranges at each step, so a long line costs little

 @example
 ```ts
 const found = inRanges({ ranges, codePoint: 0x23_29, },);
 ```
 */
export function inRanges(
  {
    ranges,
    codePoint,
  }: {
    readonly ranges: readonly CodeRange[];
    readonly codePoint: number;
  },
): boolean {
  return (function search(): boolean {
    /**
     Index of the first range not yet ruled out.
     */
    let low = 0;
    /**
     Index one past the last range not yet ruled out.
     */
    let high = ranges.length;
    while (low < high) {
      /**
       Middle of what is left.
       */
      const middle = Math.floor((low + high) / 2,);
      /**
       The range at the middle.
       */
      const range = ranges[middle];
      if (range === undefined)
        return false;
      if (codePoint < range.from)
        high = middle;
      else if (codePoint > range.to)
        low = middle + 1;
      else
        return true;
    }
    return false;
  })();
}

//endregion East Asian Width parse
