import type { BlockRange, } from '@monochromatic-dev/translation-repair-document-structure/ts';
import {
  closesFlow,
  NO_MARK,
  openingMark,
} from './source-ships-open-flow.ts';
import { footnoteContainerPrefixOf, } from './source-ships-footnote-container.ts';
import { footnoteOpensRawHtml, } from './source-ships-footnote-html.ts';
import { splitFlow, } from './source-ships-opener-flow.ts';
import type { OpenerInCode, } from './source-ships-opener-text.ts';
import type {
  TextEdit,
  TextRange,
} from './source-ships-ranges.ts';

//region Source ships opener footnote
// A COMMENT THAT NEVER CLOSES, OPENED IN A FENCE OR A DISPLAY MATH THAT A FOOTNOTE DEFINITION HOLDS. The strict grammar
// disables indented code, so a fence or a display math indented four columns under a footnote definition is one of the
// footnote there, wherever the footnote stands: at the top, in a block quote or in a list item, with a blank line between
// the definition and the block or none. The plain reading takes those lines for indented code, or for the lazy lines of
// the paragraph before the footnote, which `source-ships-opener-in-code.ts` does not look into. The lines that hold an opener are
// taken out of the block as they are for a fence at the top: the block is closed before the line, the line is written as
// a paragraph of the footnote, and the block is opened again with the same mark, each line carrying the marks and the
// indentation of its container and the four columns the footnote takes. The lines are read from the footnote's range,
// not from the nodes of the plain parse, which differ by shape (indented code, a paragraph with a code span in it).

/**
 The indentation a footnote takes from the blocks under it, as the text writes it.
 */
const FOOTNOTE_INDENTS: readonly string[] = [
  '    ',
  '\t',
];

/**
 Where each line of a stretch of the text stands.

 @param source - text

 @param range - stretch to cut into lines

 @returns Start and end of each line, the end before its line ending and its carriage return

 @example
 ```ts
 const lines = linesOfRange({ source: 'a\nb', range: { start: 0, end: 3, }, },);
 ```
 */
function linesOfRange(
  {
    source,
    range,
  }: {
    readonly source: string;
    readonly range: TextRange;
  },
): readonly TextRange[] {
  /**
   Lines found.
   */
  const lines: TextRange[] = [];
  /**
   Where the line being read starts.
   */
  const line = { start: range.start, };
  for (let reading = true; reading;) {
    /**
     Index of the line feed that ends the line, none for the last line of the stretch.
     */
    const feed = source.indexOf(
      '\n',
      line.start,
    );
    /**
     Whether the line is the last of the stretch.
     */
    const last = (feed === (-1)) || (feed >= range.end);
    /**
     Where the line ends, before its line ending.
     */
    const end = last ? range.end : feed;
    lines.push({
      start: line.start,
      end: ((end > line.start) && (source.charAt(end - 1,) === '\r')) ? (end - 1) : end,
    },);
    line.start = feed + 1;
    reading = !last;
  }
  return lines;
}

/**
 Takes the lines that hold an opener out of the fences and display maths of the blocks under one footnote definition.

 @param source - text the parse was read from, with its comments as they stand

 @param range - where the footnote stands, from its label

 @param starts - index of the `<` of each opener of a comment that never closes

 @param closed - comments that close

 @returns Edits and the openers they write, none for a footnote that holds no fence or display math with an opener

 @example
 ```ts
 const split = splitFlowsOf({ source, range: { start: 0, end: 39, }, starts: [20,], closed: [], },);
 ```
 */
function splitFlowsOf(
  {
    source,
    range,
    starts,
    closed,
    htmlRanges,
  }: {
    readonly source: string;
    readonly range: TextRange;
    readonly starts: readonly number[];
    readonly closed: readonly TextRange[];
    readonly htmlRanges: readonly TextRange[];
  },
): OpenerInCode {
  /**
   What its container puts before the footnote's label, the marks of which every later line of the footnote carries.
   */
  const prefix = source.slice(
    source.lastIndexOf(
      '\n',
      range.start - 1,
    ) + 1,
    range.start,
  );
  /**
   Edits written.
   */
  const edits: TextEdit[] = [];
  /**
   Openers the edits write.
   */
  const handled: number[] = [];
  /**
   The flow being read: the mark that opened it, where its opening line starts and what every line of it carries before its
   content; no mark while outside one.
   */
  const flow: {
    mark: string | typeof NO_MARK;
    from: number;
    carried: string
  } = {
    mark: NO_MARK,
    from: range.start,
    carried: '',
  };

  /**
   Takes the lines of the flow read so far, up to a place, out of it.

   @param end - where the flow ends: the start of its closing line, or the end of the footnote

   @example
   ```ts
   close({ end: 30, });
   ```
   */
  function close({ end, }: { readonly end: number; },): void {
    if (flow.mark === NO_MARK)
      return;
    /**
     What taking the lines out of the flow comes to.
     */
    const split = splitFlow({
      source,
      range: {
        start: flow.from,
        end,
      },
      mark: flow.mark,
      carried: flow.carried,
      starts,
      closed,
    },);
    edits.push(...split.edits,);
    handled.push(...split.handled,);
    flow.mark = NO_MARK;
  }

  /**
   Whether the lines read are still the block of raw HTML that the definition opens with, if it does: they are text up to
   the first blank line.
   */
  const html = { open: footnoteOpensRawHtml({
    source,
    range,
  },), };
  for (const line of linesOfRange({
    source,
    range,
  },)) {
    if (line.start === range.start)
      continue;
    // A line in a block of raw HTML of the source is text to its reader, and the page writes it as text.
    if (htmlRanges.some(function holds(block,): boolean {
      return (block.start <= line.start) && (line.start < block.end);
    },))
      continue;
    /**
     What its container takes of the line.
     */
    const container = footnoteContainerPrefixOf({
      source,
      at: line.start,
      pattern: prefix,
    },);
    // A line without the marks of a block quote is a lazy line of a paragraph, whatever it starts with.
    if (!container.marked)
      continue;
    /**
     Where the indentation of the footnote starts on the line, after the marks and white space of its container.
     */
    const indentAt = line.start + container.length;
    if (html.open) {
      // A line that holds nothing but the marks of its container is a blank line, which ends the block of raw HTML.
      /**
       What the line holds after the marks of its container.
       */
      const rest = source.slice(
        indentAt,
        line.end,
      );
      html.open = rest.trim() !== '';
      continue;
    }
    /**
     The indentation the footnote takes, when the line has it.
     */
    const indent = FOOTNOTE_INDENTS.find(function startsLine(candidate,): boolean {
      return source.startsWith(
        candidate,
        indentAt,
      );
    },);
    if (indent === undefined)
      continue;
    /**
     The line less the marks of its container and the four columns the footnote takes.
     */
    const content = source.slice(
      indentAt + indent.length,
      line.end,
    );
    if (flow.mark === NO_MARK) {
      /**
       What the line opens, when it opens a fence or a display math.
       */
      const mark = openingMark({ line: content, },);
      if (mark !== NO_MARK) {
        flow.mark = mark;
        flow.from = line.start;
        flow.carried = source.slice(
          line.start,
          indentAt + indent.length,
        );
      }
      continue;
    }
    if (closesFlow({
      line: content,
      mark: flow.mark,
    },))
      close({ end: line.start, },);
  }
  close({ end: range.end, },);
  return {
    edits,
    handled,
  };
}

/**
 Finds the fences and display maths that a footnote definition holds, at the top or in a container, with an opener of a
 comment that never closes, and writes what holds one outside the block.

 @param source - text the parse was read from, with its comments as they stand

 @param starts - index of the `<` of each opener of a comment that never closes

 @param closed - comments that close

 @param footnotes - where the page reads a footnote definition, and where a reader of the source does

 @param htmlRanges - where a reader of the source reads a block of raw HTML, whose lines are text to the reader and are not
 read as the blocks of a footnote; none when absent

 @returns Edits that take the lines holding an opener out of their fence or display math, and the openers they write

 @example
 ```ts
 const split = openersInFootnoteFlowsOf({ source, starts: [20,], closed: [], footnotes: [{ start: 0, end: 39, },], },);
 ```
 */
export function openersInFootnoteFlowsOf(
  {
    source,
    starts,
    closed,
    footnotes,
    htmlRanges = [],
  }: {
    readonly source: string;
    readonly starts: readonly number[];
    readonly closed: readonly TextRange[];
    readonly footnotes: readonly BlockRange[];
    readonly htmlRanges?: readonly TextRange[];
  },
): OpenerInCode {
  /**
   Where each footnote stands that holds an opener: for the footnotes that start at one place, the longest reach, since the
   lines a footnote has in the page's reading are those it has in the reader's.
   */
  const reaches = new Map<number, number>();
  for (const {
    start,
    end,
  } of footnotes) {
    if (starts.some(function inside(opener,): boolean {
      return (opener >= start) && (opener < end);
    },)) {
      reaches.set(
        start,
        Math.max(
          end,
          reaches.get(start,) ?? 0,
        ),
      );
    }
  }
  /**
   The same footnotes as ranges.
   */
  const ranges: TextRange[] = [];
  for (const [start, end,] of reaches) {
    ranges.push({
      start,
      end,
    },);
  }
  /**
   What each of them comes to.
   */
  const splits = ranges.map(function splitOne(range,): OpenerInCode {
    return splitFlowsOf({
      source,
      range,
      starts,
      closed,
      htmlRanges,
    },);
  },);
  return {
    edits: splits.flatMap(function editsOf({ edits, },): OpenerInCode['edits'] {
      return edits;
    },),
    handled: splits.flatMap(function handledOf({ handled, },): OpenerInCode['handled'] {
      return handled;
    },),
  };
}

//endregion Source ships opener footnote
