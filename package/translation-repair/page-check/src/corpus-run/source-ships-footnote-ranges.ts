import type { BlockRange, } from '@monochromatic-dev/translation-repair-document-structure/ts';

//region Source ships footnote ranges
// THE FOOTNOTE DEFINITIONS THE ENCODER CUTS BY AT THE TOP OF A TEXT: where a reader of the source in GFM ends each one,
// which the reading tests of the encoder are made against, and the definitions only the page starts. The strict grammar
// disables indented code, so it starts a footnote at a line `[^label]:` indented four spaces or more after a paragraph,
// where CommonMark and GFM read a lazy line of that paragraph and see no definition. The code span the original pairs
// across that line is cut there on the page, so the encoder has to cut there too, though the reader of the source sees
// no footnote.

/**
 Says where the page starts a footnote definition and a reader of the source does not.

 @param source - ranges where a reader of the source in GFM reads a footnote definition, in text order

 @param page - ranges where the page reads one, in text order

 @returns Starts of the page's ranges that are outside every range of the source, in text order

 @example
 ```ts
 const starts = pageOnlyStartsOf({ source: [], page: [{ start: 9, end: 23, },], },);
 ```
 */
export function pageOnlyStartsOf(
  {
    source,
    page,
  }: {
    readonly source: readonly BlockRange[];
    readonly page: readonly BlockRange[];
  },
): readonly number[] {
  return page
    .map(function startOf({ start, },): number {
      return start;
    },)
    .filter(function startsOutsideTheSource(start,): boolean {
      return !source.some(function holds(range,): boolean {
        return (start >= range.start) && (start < range.end);
      },);
    },);
}

/**
 Takes out the starts whose line follows a row of a table, where the line is no part of a paragraph.

 A reader of the source takes the line after the last row of a table for a block of its own (indented code, for a line
 indented four columns or more), and the plain reading, which has no tables, takes the rows and the line for one
 paragraph with a code span across them. Joining the line to the row before it writes the line into the row, so such a
 start is cut as a start in any other paragraph is.

 @param source - whole text, or the text the parse was given with its comments blanked, of the same length

 @param starts - offsets where a footnote definition starts on the page and not for a reader of the source

 @param tables - where a reader of the source reads a table, the end being the end of its last row

 @returns Starts whose line starts after a line ending outside every table, in the order given

 @example
 ```ts
 const joinable = startsOffTables({ source, starts: [35,], tables: [{ start: 0, end: 30, },], },);
 ```
 */
export function startsOffTables(
  {
    source,
    starts,
    tables,
  }: {
    readonly source: string;
    readonly starts: readonly number[];
    readonly tables: readonly BlockRange[];
  },
): readonly number[] {
  return starts.filter(function followsNoTableRow(start,): boolean {
    /**
     Index of the line ending that ends the line before the start's line.
     */
    const ending = source.lastIndexOf(
      '\n',
      start - 1,
    );
    return !tables.some(function holds(table,): boolean {
      return (table.start <= ending) && (ending <= table.end);
    },);
  },);
}

/**
 Joins the source's footnote ranges with the page's that start where no range of the source stands.

 @param source - ranges where a reader of the source in GFM reads a footnote definition, in text order

 @param page - ranges where the page reads one, in text order

 @returns Source's ranges and the page's whose start is outside every range of the source, in text order

 @example
 ```ts
 const joined = withPageOnlyStarts({ source: [], page: [{ start: 9, end: 23, },], },);
 ```
 */
export function withPageOnlyStarts(
  {
    source,
    page,
  }: {
    readonly source: readonly BlockRange[];
    readonly page: readonly BlockRange[];
  },
): readonly BlockRange[] {
  /**
   Starts of the page's ranges that stand outside every range of the source.
   */
  const outside = pageOnlyStartsOf({
    source,
    page,
  },);
  return [
    ...source,
    ...page.filter(function startsOutside({ start, },): boolean {
      return outside.includes(start,);
    },),
  ].toSorted(function byStart(
    left,
    right,
  ): number {
    return left.start - right.start;
  },);
}

//endregion Source ships footnote ranges
