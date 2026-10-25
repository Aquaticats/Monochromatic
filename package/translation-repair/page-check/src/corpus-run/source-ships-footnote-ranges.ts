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
  return [
    ...source,
    ...page.filter(function startsOutsideTheSource({ start, },): boolean {
      return !source.some(function holds(range,): boolean {
        return (start >= range.start) && (start < range.end);
      },);
    },),
  ].toSorted(function byStart(
    left,
    right,
  ): number {
    return left.start - right.start;
  },);
}

//endregion Source ships footnote ranges
