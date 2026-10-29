import {
  type BlockRange,
  footnoteDefinitionRangesOf,
  pageFootnoteDefinitionRangesOf,
  tableRangesOf,
  type TreeNode,
} from '@monochromatic-dev/translation-repair-document-structure/ts';
import {
  endLineOf,
  hasFootnoteLabel,
} from './source-ships-footnote.ts';
import {
  type RawHtmlRegion,
  rawHtmlRegionsOf,
} from './source-ships-footnote-html-region.ts';
import type { FootnoteState, } from './source-ships-pending.ts';
import {
  closingBracket,
  rangeOf,
  type TextEdit,
  type TextRange,
} from './source-ships-ranges.ts';

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
 indented four columns or more), and so does the plain reading, which reads the table: the line is no part of a paragraph,
 so no span runs across it. Joining the line to the row before it would write the line into the row, so such a start is
 cut as a start in any other paragraph is, and a plain reading that took the rows and the line for one paragraph would
 not join it.

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
 What {@link pageOnlyDefinitionWritten} answers for a node that is no such definition.
 */
export const NO_PAGE_ONLY_DEFINITION: unique symbol = Symbol(
  'no GFM footnote start of the page at this link reference definition',
);

/**
 How a link reference definition is written that the page reads as the start of a footnote definition.
 */
export type PageOnlyDefinition = {
  /**
   The edit that writes the caret of the label with a backslash.
   */
  readonly edit: TextEdit;

  /**
   The rest of the label, which is text.
   */
  readonly label: TextRange;

  /**
   The state of the container once the definition is read: no footnote is open, since the page reads none.
   */
  readonly state: FootnoteState;
};

/**
 Writes a link reference definition that the page reads as the start of a footnote definition and a reader of the source
 does not.

 CommonMark and GFM take a line `[^label]:` indented four columns or more right after a link reference definition for a
 second definition, which shows nothing: the first keeps the paragraph going, and a definition may start on any line of it.
 The strict grammar disables indented code, so it starts a footnote at the line, and the footnote shows its label and its
 text. The caret of the label is written with a backslash, so that the page reads a definition of another label.

 @param source - text the node was parsed from

 @param node - node of the plain reading

 @param starts - offsets where a footnote definition starts on the page and not for a reader of the source

 @param endLine - line the block read last in the container ends on, which a definition without a position leaves

 @returns Edit, label and state of the container; {@link NO_PAGE_ONLY_DEFINITION} when the node is no link
 reference definition with a footnote label or the page starts no footnote at it

 @example
 ```ts
 const written = pageOnlyDefinitionWritten({ source: '[r]: u\n    [^1]: y', node, starts: [11,], endLine: 1, },);
 ```
 */
export function pageOnlyDefinitionWritten(
  {
    source,
    node,
    starts,
    endLine,
  }: {
    readonly source: string;
    readonly node: TreeNode;
    readonly starts: readonly number[];
    readonly endLine: number;
  },
): PageOnlyDefinition | typeof NO_PAGE_ONLY_DEFINITION {
  if (!hasFootnoteLabel({
    source,
    node,
  },))
    return NO_PAGE_ONLY_DEFINITION;
  /**
   Where the definition starts and ends.
   */
  const {
    start: from,
    end: to,
  } = rangeOf({ node, },);
  /**
   Where the bracket that opens the label stands, behind any indentation.
   */
  const open = source.indexOf(
    '[',
    from,
  );
  if (!starts.includes(open,))
    return NO_PAGE_ONLY_DEFINITION;
  return {
    edit: {
      start: open + 1,
      end: open + 2,
      replacement: String.raw`\^`,
    },
    label: {
      start: open + 2,
      end: closingBracket({
        source,
        from: open + 1,
        limit: to,
      },),
    },
    state: {
      open: false,
      endLine: endLineOf({
        node,
        fallback: endLine,
      },),
      endsInLiteral: false,
      prefix: '',
      opensRawHtml: false,
      opensFootnote: false,
      goneOnLazily: false,
    },
  };
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

/**
 What a reader of the source and the page read of a text's tables and footnote definitions.
 */
export type FootnoteReadings = {
  /**
   Where the strict grammar reads a table, which cuts what it holds at a pipe.
   */
  readonly tables: readonly BlockRange[];

  /**
   Where the page ends a footnote definition, which a line of raw HTML does not end: the page writes it as text.
   */
  readonly pageFootnoteRanges: readonly BlockRange[];

  /**
   Where a reader of the source reads a footnote definition, and the definitions only the page starts.
   */
  readonly footnoteRanges: readonly BlockRange[];

  /**
   Where the lines of a block of raw HTML that a footnote definition opens with stand, which are text on the page.
   */
  readonly rawHtmlRegions: readonly RawHtmlRegion[];

  /**
   Where a footnote definition starts, for the reader of the source and for the page.
   */
  readonly footnoteStarts: readonly number[];

  /**
   Where the page starts a footnote definition and a reader of the source does not: a line indented four columns or more
   after a paragraph or a link reference definition, less the lines that follow a row of a table.
   */
  readonly pageOnlyStarts: readonly number[];
};

/**
 Reads where a text holds tables and footnote definitions for the reader of the source and for the page.

 @param readable - text as the parse reads it, with its comments blanked

 @param source - text itself, whose label lines hold the comment openers that `readable` writes as text, so that a
 footnote definition that opens with one is found to open with raw HTML; `readable` when absent

 @returns Tables, footnote definitions of the page and of the two readings together, lines of the raw HTML
 blocks the definitions open with, and the starts only the page reads

 @example
 ```ts
 const { pageOnlyStarts, } = footnoteReadingsOf({ readable: 'a\n    [^1]: y', },);
 ```
 */
export function footnoteReadingsOf(
  {
    readable,
    source = readable,
  }: {
    readonly readable: string;
    readonly source?: string;
  },
): FootnoteReadings {
  /**
   Where the strict grammar reads a table.
   */
  const tables = tableRangesOf({ body: readable, },);
  /**
   Where the page ends a footnote definition.
   */
  const pageFootnoteRanges = pageFootnoteDefinitionRangesOf({ body: readable, },);
  /**
   Where a reader of the source reads a footnote definition, which ends where the plain reading's paragraph goes on.
   */
  const sourceFootnoteRanges = footnoteDefinitionRangesOf({ body: readable, },);
  return {
    tables,
    pageFootnoteRanges,
    footnoteRanges: withPageOnlyStarts({
      source: sourceFootnoteRanges,
      page: pageFootnoteRanges,
    },),
    rawHtmlRegions: rawHtmlRegionsOf({
      source,
      footnotes: [
        ...sourceFootnoteRanges,
        ...pageFootnoteRanges,
      ],
    },),
    footnoteStarts: [
      ...sourceFootnoteRanges,
      ...pageFootnoteRanges,
    ].map(function startOf({ start, },): number {
      return start;
    },),
    pageOnlyStarts: startsOffTables({
      source: readable,
      starts: pageOnlyStartsOf({
        source: sourceFootnoteRanges,
        page: pageFootnoteRanges,
      },),
      tables,
    },),
  };
}

//endregion Source ships footnote ranges
