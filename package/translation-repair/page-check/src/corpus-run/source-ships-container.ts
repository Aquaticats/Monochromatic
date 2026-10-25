import { withoutQuoteMarks, } from './source-ships-footnote.ts';
import {
  joinedSpans,
  nestedSpans,
  type SpansReader,
} from './source-ships-nested.ts';
import {
  indicesOf,
  type SourceSpans,
  type TextRange,
} from './source-ships-ranges.ts';

//region Source ships container
// A PARAGRAPH OF A BLOCK QUOTE OR A LIST ITEM IN WHICH A FOOTNOTE DEFINITION STARTS ON A LATER LINE, read as the strict
// grammar reads it. The plain reading keeps the paragraph whole, so the inline constructs that start before the
// definition line (a code span, math, a link) go on past it; the strict grammar starts the definition on that line,
// whether the line carries the marks of the quote or the hanging indent of the item, or is a lazy line indented up to
// three spaces, and ends the construct there. The lines before the first definition are read as a text of their own,
// and the text from each definition to the next as another. A definition that carries the marks of its container has
// them taken off every line after it, since they are no part of what the footnote holds; a lazy definition line is one
// at which the strict grammar closes the container, so the text from it on is read as it stands, a line with a mark
// after it opening a quote of its own.

/**
 Where the line that holds an index starts.

 @param source - text

 @param at - index of a character of the line

 @returns Index of the line's first character

 @example
 ```ts
 const start = lineStartOf({ source: 'a\nb', at: 2, },);
 ```
 */
function lineStartOf(
  {
    source,
    at,
  }: {
    readonly source: string;
    readonly at: number;
  },
): number {
  return source.lastIndexOf(
    '\n',
    at - 1,
  ) + 1;
}

/**
 Columns a tab counts for in the prefix of a line.
 */
const TAB_COLUMNS = 4;

/**
 How wide a prefix of white space is, a tab counting for several columns.

 @param prefix - characters before the content of a line

 @returns Width in columns

 @example
 ```ts
 const width = widthOf({ prefix: '\t ', },);
 ```
 */
function widthOf({ prefix, }: { readonly prefix: string; },): number {
  /**
   Columns counted so far.
   */
  const columns = { count: 0, };
  for (const character of prefix)
    columns.count += (character === '\t') ? TAB_COLUMNS : 1;
  return columns.count;
}

/**
 How many block quote marks a prefix holds.

 @param prefix - characters before the content of a line

 @returns Count of marks

 @example
 ```ts
 const marks = quoteMarksIn({ prefix: '> > ', },);
 ```
 */
function quoteMarksIn({ prefix, }: { readonly prefix: string; },): number {
  return Array
    .from(prefix,)
    .filter(function isMark(character,): boolean {
      return character === '>';
    },)
    .length;
}

/**
 Whether the line of a footnote definition stays in the container its paragraph stands in. A line that carries every
 quote mark of the paragraph's first line, or at least its width where that line has no mark (the hanging indent of a
 list item), belongs to the container; a lazy line, with fewer marks or less indent, is one at which the strict grammar
 closes the container, and the footnote stands at the level above.

 @param source - whole text

 @param range - where the paragraph stands

 @param split - where the footnote definition starts in the paragraph

 @returns Whether the footnote is inside the container

 @example
 ```ts
 const inside = staysInContainer({ source: '> a\n> [^1]: b', range: { start: 2, end: 13, }, split: 6, },);
 ```
 */
function staysInContainer(
  {
    source,
    range,
    split,
  }: {
    readonly source: string;
    readonly range: TextRange;
    readonly split: number;
  },
): boolean {
  /**
   What stands before the paragraph on its first line.
   */
  const opening = source.slice(
    lineStartOf({
      source,
      at: range.start,
    },),
    range.start,
  );
  /**
   What stands before the definition on its line.
   */
  const definition = source.slice(
    lineStartOf({
      source,
      at: split,
    },),
    split,
  );
  if (quoteMarksIn({ prefix: opening, },) > 0)
    return quoteMarksIn({ prefix: definition, },) >= quoteMarksIn({ prefix: opening, },);
  return widthOf({ prefix: definition, },) >= widthOf({ prefix: opening, },);
}

/**
 Reads a paragraph of the plain reading, in a block quote or a list item, in which a footnote definition starts on a
 later line: the lines before the first definition as a text of their own, and each definition with the lines up to
 the next one as another.

 @param source - whole text

 @param readable - whole text with its comments blanked, the parse's own input

 @param readSpans - what reads a block's spans, handed in so this module and the reader do not import each other

 @param footnotes - where the strict grammar reads a footnote definition in the whole text

 @param range - where the paragraph stands

 @param split - where the first footnote definition starts in the paragraph

 @param end - where the lines of the last footnote end: the end of the paragraph, or beyond it when the footnote reached
 blocks of the top

 @returns Spans of every part at the offsets of the whole text

 @throws {@link Error} when a node of any part has no position, which the reading used never builds

 @example
 ```ts
 const spans = footnoteInContainerSpans({ source, readable, readSpans: spansOf, range, split: 8, end: 20, },);
 ```
 */
export function footnoteInContainerSpans(
  {
    source,
    readable,
    readSpans,
    footnotes,
    range,
    split,
    end,
  }: {
    readonly source: string;
    readonly readable: string;
    readonly readSpans: SpansReader;
    readonly footnotes: readonly TextRange[];
    readonly range: TextRange;
    readonly split: number;
    readonly end: number;
  },
): SourceSpans {
  /**
   Where each footnote definition of the paragraph starts, the first at the split.
   */
  const starts = footnotes
    .map(function startOf({ start, },): number {
      return start;
    },)
    .filter(function inParagraph(start,): boolean {
      return (start >= split) && (start < range.end);
    },);
  /**
   The lines before the first definition, whose line with its marks belongs to no part.
   */
  const before = nestedSpans({
    source,
    readable,
    readSpans,
    indices: withoutQuoteMarks({
      source,
      range: {
        start: range.start,
        end: lineStartOf({
          source,
          at: split,
        },),
      },
    },),
  },);
  /**
   The parts from each footnote line to the next one, in text order.
   */
  const parts = starts.map(function partFrom(
    start,
    index,
  ): SourceSpans {
    /**
     Where the next definition starts, absent after the last.
     */
    const next = starts.at(index + 1,);
    /**
     The lines of this footnote: up to the line of the next definition, or the end of the footnote's lines.
     */
    const lines: TextRange = {
      start,
      end: (next === undefined)
        ? end
        : lineStartOf({
          source,
          at: next,
        },),
    };
    return nestedSpans({
      source,
      readable,
      readSpans,
      indices: staysInContainer({
        source,
        range,
        split: start,
      },)
        ? withoutQuoteMarks({
          source,
          range: lines,
        },)
        : indicesOf({ range: lines, },),
    },);
  },);
  /**
   The lines before the first definition with the parts joined to it so far.
   */
  const joined = { spans: before, };
  for (const part of parts) {
    joined.spans = joinedSpans({
      first: joined.spans,
      second: part,
    },);
  }
  return joined.spans;
}

//endregion Source ships container
