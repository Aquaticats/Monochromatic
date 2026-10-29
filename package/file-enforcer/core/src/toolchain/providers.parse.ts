/**
 Probe output parsers: `bin list` and `cargo binstall --list` tables read
 into name to version maps, by index scans over whitespace-separated columns.
 */

import { basename, } from 'node:path';

//region Whitespace scanning

/**
 Result of `indexOf` when the searched value is absent.
 */
const NOT_FOUND = -1;

/**
 Words a `cargo binstall --list` row carries before its method column
 counts as present: crate, cargo version, binstall version, method.
 */
const CARGO_ROW_WORD_COUNT = 4;

/**
 Shortest whitespace run that separates two `bin list` columns; a single
 space belongs to the column text (paths may contain one).
 */
const MIN_COLUMN_GAP = 2;

/**
 Whether one character is whitespace in the sense of the `\s` class (white
 space and line terminators).

 @param char - Single character to classify

 @returns True when the character is whitespace

 @example
 ```ts
 isWhitespace('\t',); // true
 ```
 */
function isWhitespace(char: string,): boolean {
  return char.trim() === '';
}

/**
 Characters a path column cannot contain: the line terminators.
 */
const LINE_TERMINATORS: ReadonlySet<string> = new Set([
  '\n',
  '\r',
  '\u2028',
  '\u2029',
],);

/**
 Splits text into its whitespace-separated words, dropping empty words.

 @param text - Text to split

 @returns Words in order

 @example
 ```ts
 splitWords('a  b\tc',); // ['a', 'b', 'c']
 ```
 */
function splitWords(text: string,): readonly string[] {
  return Array.from(
    text,
    function spaceOut(char,): string {
      return isWhitespace(char,)
        ? ' '
        : char;
    },
  )
    .join('',)
    .split(' ',)
    .filter(
    function isWord(word,): boolean {
      return word !== '';
    },
    );
}

/**
 Splits provider output into rows at every line terminator `readBinRow` guards,
 in one left-to-right pass,
 so rows separated by a carriage return or a Unicode line terminator
 survive instead of being swallowed by the row before them.

 @param text - Provider output to split

 @returns Rows in order, empty rows included

 @example
 ```ts
 splitRows('a\r\nb',); // ['a', '', 'b']
 ```
 */
function splitRows(text: string,): readonly string[] {
  /**
   Rows found so far.
   */
  const rows: string[] = [];
  /**
   Offset the current row starts at.
   */
  let start = 0;
  for (let index = 0; index < text.length; index += 1) {
    if (LINE_TERMINATORS.has(text.charAt(index,),)) {
      /**
       Row between the previous terminator and this one.
       */
      const row = text.slice(
        start,
        index,
      );
      rows.push(row,);
      start = index + 1;
    }
  }
  rows.push(text.slice(start,),);
  return rows;
}

/**
 Removes a leading `v` from a version word when something remains after it.

 @param word - Version word as printed

 @returns The word without a leading `v`; a lone `v` stays

 @example
 ```ts
 withoutVersionPrefix('v1.2.3',); // '1.2.3'
 ```
 */
function withoutVersionPrefix(word: string,): string {
  return (word.length > 1) && word.startsWith('v',)
    ? word.slice(1,)
    : word;
}

//endregion Whitespace scanning

//region bin list

/**
 One whitespace run inside a table row, as character offsets.
 */
type Gap = {
  readonly start: number;
  readonly end: number;
};

/**
 One `Path  Version  URL  Status` row reduced to the columns read.
 */
type BinRow = {
  readonly path: string;
  readonly version: string;
};

/**
 Finds the whitespace runs of two or more characters in a row,
 in one left-to-right pass (each character is visited once).

 @param isSpace - Per-character whitespace flags of the row

 @returns Runs in order, as offsets into the row

 @example
 ```ts
 findColumnGaps([false, true, true, false,],); // [{ start: 1, end: 3, }]
 ```
 */
function findColumnGaps(isSpace: readonly boolean[],): readonly Gap[] {
  /**
   Runs found so far.
   */
  const gaps: Gap[] = [];
  /**
   Start of the run being read, or NOT_FOUND between runs.
   */
  const run = { start: NOT_FOUND, };
  for (let index = 0; index <= isSpace.length; index += 1) {
    /**
     Whether this position is whitespace; the position past the end closes any open run.
     */
    const space = (index < isSpace.length) && (isSpace[index] === true);
    if (space && (run.start === NOT_FOUND))
      run.start = index;
    if ((!space) && (run.start !== NOT_FOUND)) {
      if ((index - run.start) >= MIN_COLUMN_GAP) {
        gaps.push({
          start: run.start,
          end: index,
        },);
      }
      run.start = NOT_FOUND;
    }
  }
  return gaps;
}

/**
 Reads the path and version columns of one `bin list` row. The path is the
 shortest prefix followed by a column gap, a version word, and another column
 gap; a row that begins with whitespace is not a table row.

 @param line - One line of `bin list` output

 @returns The row's columns, or an empty list when the line is not a row

 @example
 ```ts
 readBinRow('/bin/tool  v1.2.3  https://example.test  ok',); // [{ path: '/bin/tool', version: '1.2.3', }]
 ```
 */
function readBinRow(line: string,): readonly BinRow[] {
  /**
   Characters of the row.
   */
  const chars = Array.from(line,);
  /**
   Per-character whitespace flags.
   */
  const isSpace = chars.map(isWhitespace,);
  if ((chars.length === 0) || (isSpace[0] === true))
    return [];
  /**
   Column gaps of the row.
   */
  const gaps = findColumnGaps(isSpace,);
  /**
   Offset of the first line terminator; the path column ends before it.
   */
  const terminatorAt = chars.findIndex(
    function isLineTerminator(char,): boolean {
      return LINE_TERMINATORS.has(char,);
    },
  );
  /**
   Offsets where a column gap starts, for constant-time lookup.
   */
  const gapStarts = new Set(gaps.map(function toStart(gap,): number {
    return gap.start;
  },),);
  /**
   First gap whose following word is itself followed by a column gap.
   The word scans after successive gaps cover disjoint stretches of the row,
   so the whole search stays linear in the row length.
   */
  const versionGap = gaps.find(
    function followedByVersionColumn(gap,): boolean {
      if ((terminatorAt !== NOT_FOUND) && (gap.start > terminatorAt))
        return false;
      /**
       Offset where the word after the gap ends, or -1 at the end of the row.
       */
      const wordEnd = isSpace.indexOf(
        true,
        gap.end,
      );
      if ((gap.end >= chars.length) || (wordEnd === NOT_FOUND))
        return false;
      return gapStarts.has(wordEnd,);
    },
  );
  if (versionGap === undefined)
    return [];
  /**
   Version word as printed, possibly marked with `*` and prefixed with `v`.
   */
  const word = chars.slice(
    versionGap.end,
    isSpace.indexOf(
      true,
      versionGap.end,
    ),
  )
    .join('',);
  return [{
    path: chars.slice(
      0,
      versionGap.start,
    )
      .join('',),
    version: withoutVersionPrefix(
      (word.length > 1) && word.startsWith('*',)
        ? word.slice(1,)
        : word,
    ),
  }];
}

/**
 Reads one `bin list` output table into name to version pairs.

 @param output - `bin list` stdout

 @returns Map from binary file name to reported version

 @example
 ```ts
 const versions = parseBinList('/bin/tool  v1.2.3  https://example.test  ok',);
 ```
 */
export function parseBinList(output: string,): ReadonlyMap<string, string> {
  /**
   Parsed name to version pairs.
   */
  const versions = new Map<string, string>();
  for (const line of splitRows(output,)) {
    for (const row of readBinRow(line,)) {
      versions.set(
        basename(row.path,),
        row.version,
      );
    }
  }
  return versions;
}

//endregion bin list

//region cargo binstall list

/**
 Reads `cargo binstall --list` output into crate to version pairs.

 @param output - `cargo binstall --list` stdout

 @returns Map from crate name to installed version

 @example
 ```ts
 const versions = parseCargoBinstallList('ripgrep  v14.1.0  14.1.0  prebuilt',);
 ```
 */
export function parseCargoBinstallList(
  output: string,
): ReadonlyMap<string, string> {
  /**
   Parsed crate to version pairs.
   */
  const versions = new Map<string, string>();
  for (const line of splitRows(output,)) {
    /**
     Words of one `Package  Cargo Ver  Binstall Ver  Method` row.
     */
    const words = splitWords(line,);
    /**
     Crate, cargo version and binstall version columns of the row.
     */
    const [
      crate = '',
      cargoVer = '',
      binstallVer = '',
    ] = words;
    if ((words.length < CARGO_ROW_WORD_COUNT) || isWhitespace(line.charAt(0,),))
      continue;
    if ((crate === 'Package') || (crate === 'Total:'))
      continue;
    versions.set(
      crate,
      binstallVer === '-'
        ? withoutVersionPrefix(cargoVer,)
        : withoutVersionPrefix(binstallVer,),
    );
  }
  return versions;
}

//endregion cargo binstall list
