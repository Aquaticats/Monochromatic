import type {
  BlockRange,
  TreeNode,
} from '@monochromatic-dev/translation-repair-document-structure/ts';
import {
  closesFlow,
  NO_MARK,
  openingMark,
} from './source-ships-open-flow.ts';
import { splitFlow, } from './source-ships-opener-flow.ts';
import type { OpenerInCode, } from './source-ships-opener-text.ts';
import {
  rangeOf,
  type TextEdit,
  type TextRange,
} from './source-ships-ranges.ts';

//region Source ships opener footnote
// A COMMENT THAT NEVER CLOSES, OPENED IN A FENCE OR A DISPLAY MATH THAT A FOOTNOTE DEFINITION HOLDS. The strict grammar
// disables indented code, so a fence or a display math indented four columns under a footnote definition is one of the
// footnote there, and the plain reading takes its lines for indented code, which `source-ships-opener-in-code.ts` does not
// look into. The lines that hold an opener are taken out of the block as they are for a fence at the top: the block is
// closed before the line, the line is written as a paragraph of the footnote, and the block is opened again with the same
// mark, each line carrying the four columns of indentation the footnote takes.

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
 Takes the lines that hold an opener out of the fences and display maths of one block of indented code under a footnote.

 @param source - text the parse was read from, with its comments as they stand

 @param range - where the block stands, from the start of its first line

 @param starts - index of the `<` of each opener of a comment that never closes

 @param closed - comments that close

 @returns Edits and the openers they write, none for a block that holds no fence or display math with an opener

 @example
 ```ts
 const split = splitFlowsOf({ source, range: { start: 9, end: 39, }, starts: [20,], closed: [], },);
 ```
 */
function splitFlowsOf(
  {
    source,
    range,
    starts,
    closed,
  }: {
    readonly source: string;
    readonly range: TextRange;
    readonly starts: readonly number[];
    readonly closed: readonly TextRange[];
  },
): OpenerInCode {
  /**
   The indentation the block's first line starts with.
   */
  const indent = FOOTNOTE_INDENTS.find(function startsFirstLine(candidate,): boolean {
    return source.startsWith(
      candidate,
      range.start,
    );
  },);
  if (indent === undefined) {
    return {
      edits: [],
      handled: [],
    };
  }
  /**
   Edits written.
   */
  const edits: TextEdit[] = [];
  /**
   Openers the edits write.
   */
  const handled: number[] = [];
  /**
   The flow being read: the mark that opened it and where its opening line starts, none while outside one.
   */
  const flow: {
    mark: string | typeof NO_MARK;
    from: number
  } = {
    mark: NO_MARK,
    from: range.start,
  };
  /**
   Takes the lines of the flow read so far, up to a place, out of it.

   @param end - where the flow ends: the start of its closing line, or the end of the block

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
      carried: indent,
      starts,
      closed,
    },);
    edits.push(...split.edits,);
    handled.push(...split.handled,);
    flow.mark = NO_MARK;
  }

  for (const line of linesOfRange({
    source,
    range,
  },)) {
    if (!source.startsWith(
      indent,
      line.start,
    ))
      continue;
    /**
     The line less the indentation the footnote takes.
     */
    const content = source.slice(
      line.start + indent.length,
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
 Finds the blocks of indented code under a footnote definition at the top of a text that hold a fence or a display math
 with an opener of a comment that never closes, and writes what holds one outside the block.

 @param source - text the parse was read from, with its comments as they stand

 @param root - parse of the text by the plain reading with the site's math

 @param starts - index of the `<` of each opener of a comment that never closes

 @param closed - comments that close

 @param footnotes - where the page reads a footnote definition

 @returns Edits that take the lines holding an opener out of their fence or display math, and the openers they write

 @example
 ```ts
 const split = openersInFootnoteFlowsOf({ source, root, starts: [20,], closed: [], footnotes: [{ start: 0, end: 39, },], },);
 ```
 */
export function openersInFootnoteFlowsOf(
  {
    source,
    root,
    starts,
    closed,
    footnotes,
  }: {
    readonly source: string;
    readonly root: TreeNode;
    readonly starts: readonly number[];
    readonly closed: readonly TextRange[];
    readonly footnotes: readonly BlockRange[];
  },
): OpenerInCode {
  if ((starts.length === 0) || (footnotes.length === 0) || (!('children' in root))) {
    return {
      edits: [],
      handled: [],
    };
  }
  /**
   What each block of indented code under a footnote comes to.
   */
  const splits = root.children
    .filter(function isIndentedCode(block,): boolean {
      return (block.type === 'code') && (block.position !== undefined);
    },)
    .map(function rangeOfBlock(block,): TextRange {
      return rangeOf({ node: block, },);
    },)
    .filter(function underFootnote(range,): boolean {
      return footnotes.some(function holds({
        start,
        end,
      },): boolean {
        return (start <= range.start) && (range.start < end);
      },);
    },)
    .map(function splitOne(range,): OpenerInCode {
      return splitFlowsOf({
        source,
        range,
        starts,
        closed,
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
