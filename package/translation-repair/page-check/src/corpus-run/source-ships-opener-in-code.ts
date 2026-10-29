import type {
  BlockRange,
  TreeNode,
} from '@monochromatic-dev/translation-repair-document-structure/ts';
import { readsAsDisplayMath, } from './source-ships-display-math.ts';
import { rawHtmlRegionsOf, } from './source-ships-footnote-html-region.ts';
import { splitFlow, } from './source-ships-opener-flow.ts';
import { openersInFootnoteFlowsOf, } from './source-ships-opener-footnote.ts';
import {
  CARRIED,
  type OpenerInCode,
  writtenAsText,
} from './source-ships-opener-text.ts';
import {
  NO_MARK,
  openingMark,
} from './source-ships-open-flow.ts';
import {
  rangeOf,
  type TextEdit,
  type TextRange,
} from './source-ships-ranges.ts';

//region Source ships opener in code
// A COMMENT THAT NEVER CLOSES, OPENED INSIDE CODE OR MATH. The package's own parser masks a comment by looking for the four
// characters of its opener and knows nothing of code, so it hides the rest of the page behind an opener that stands in a
// fence, a code span or math, and no backslash stops that. The character reference that stands for the opener elsewhere
// (`source-ships-encoding.ts`) shows as written inside code, so it cannot stand there either. What holds the opener is
// therefore taken out of the code, and a reader of the page sees the same characters in the code before it and in the text
// that carries it.
//
// A FENCE OR A DISPLAY MATH is closed before the line that holds the opener, the line is written as a paragraph with the
// opener as the character reference, and the block is opened again with the same mark; the info string of the opening is
// not repeated, which no reader of the text sees. A CODE SPAN OR INLINE MATH is closed before the opener, and the rest of
// its content is written as text, the closing mark dropped; where nothing but white space stands before the opener the
// whole span is written as text, the white space before the opener after the closing mark. An opener in a fence or display
// math that a footnote definition holds, at the top or in a container, is found in `source-ships-opener-footnote.ts`,
// since the plain reading takes the lines of that fence for indented code or for a code span. An opener first on a line
// right after a paragraph is the start of a block of raw HTML that ends the paragraph to a reader of the source, and
// `source-ships-html-interrupt.ts` puts a blank line before it, as before any such block.

/**
 What a code span or inline math starts with: the run of its delimiter.

 @param source - text

 @param range - where the node stands

 @returns Delimiter run at the start of the node

 @example
 ```ts
 const run = delimiterRun({ source: 'a ``x`` b', range: { start: 2, end: 7, }, },); // '``'
 ```
 */
function delimiterRun({
  source,
  range,
}: {
  readonly source: string;
  readonly range: TextRange
},): string {
  /**
   The character the run is made of.
   */
  const character = source.charAt(range.start,);
  /**
   Where the run ends.
   */
  const end = { at: range.start, };
  while ((end.at < range.end) && (source.charAt(end.at,) === character))
    end.at += 1;
  return source.slice(
    range.start,
    end.at,
  );
}

/**
 What a line of a container starts with before its content: white space and the marks of a block quote.
 */
const MARKS_AND_WHITE_SPACE = ' \t\r>';

/**
 Whether a line holds nothing but white space and the marks of a block quote.

 @param line - line of text, without its line feed

 @returns Whether every character is one of them; true for an empty line

 @example
 ```ts
 const marks = holdsOnlyMarks({ line: '> > ', },); // true
 ```
 */
function holdsOnlyMarks({ line, }: { readonly line: string; },): boolean {
  for (let at = 0; at < line.length; at += 1) {
    if (!MARKS_AND_WHITE_SPACE.includes(line.charAt(at,),))
      return false;
  }
  return true;
}

/**
 The code a span keeps before an opener: what stood before the opener less the white space and the marks of containers
 between the code and the opener, which a reader of the page would take off the span with the white space at its other
 end, and which are written after the closing delimiter.

 A line after the first that holds nothing but marks and white space is that gap, and the trailing white space of the last
 line that holds code goes with it; an angle bracket that ends code on a line is code, a closed comment's among them.

 @param before - what stands in the span before the opener

 @returns Code with the gap after it cut off; the whole of it when nothing but gap stands in it

 @example
 ```ts
 const kept = codeBefore({ before: 'x\n> ', },); // 'x'
 ```
 */
function codeBefore({ before, }: { readonly before: string; },): string {
  /**
   The lines of the stretch, the first of which starts at the span and so has no marks.
   */
  const lines = before.split('\n',);
  /**
   How many lines hold code: a line after the first that is nothing but marks and white space is not one.
   */
  const code = { lines: lines.length, };
  while ((code.lines > 1) && holdsOnlyMarks({ line: lines[code.lines - 1] ?? '', },))
    code.lines -= 1;
  /**
   The code, its trailing white space cut off.
   */
  const kept = lines.slice(
    0,
    code.lines,
  )
    .join('\n',)
    .trimEnd();
  return (kept === '') ? before : kept;
}

/**
 Takes the first opener a code span or inline math holds out of it, with what follows it in the span.

 @param source - text

 @param range - where the node stands, from its first delimiter to its last

 @param starts - index of the `<` of each opener of a comment that never closes

 @param closed - comments that close

 @returns Edit and the openers it writes, none when the node holds no opener between its delimiters

 @example
 ```ts
 const split = splitSpan({ source, range: { start: 2, end: 12, }, starts: [5,], closed: [], },);
 ```
 */
function splitSpan(
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
   The delimiter that opens the node, which closes it too.
   */
  const delimiter = delimiterRun({
    source,
    range,
  },);
  /**
   Where the content of the node ends.
   */
  const contentEnd = range.end - delimiter.length;
  /**
   The openers between the delimiters.
   */
  const inside = starts.filter(function holds(start,): boolean {
    return (start >= (range.start + delimiter.length)) && (start < contentEnd);
  },);
  /**
   The first opener the node holds.
   */
  const [first,] = inside;
  if (first === undefined) {
    return {
      edits: [],
      handled: [],
    };
  }
  /**
   What stands in the node before the first opener.
   */
  const before = source.slice(
    range.start + delimiter.length,
    first,
  );
  /**
   Whether anything but white space stands before the opener, which the span then keeps as code.
   */
  const keepsCode = before.trim() !== '';
  /**
   The code the span keeps.
   */
  const kept = codeBefore({ before, },);
  /**
   Where the content of the node starts.
   */
  const contentStart = range.start + delimiter.length;
  /**
   Where the code the span keeps ends.
   */
  const keptEnd = contentStart + kept.length;
  /**
   What separates the code from the closing delimiter when it ends with the character the delimiter is made of, which
   would make the run longer than the delimiter and close nothing.
   */
  const separator = kept.endsWith(delimiter.charAt(0,),) ? ' ' : '';
  return {
    edits: [
      {
        start: keepsCode ? keptEnd : range.start,
        end: range.end,
        replacement: `${keepsCode ? `${separator}${delimiter}` : ''}${
          writtenAsText({
            source,
            range: {
              start: keepsCode ? keptEnd : (range.start + delimiter.length),
              end: contentEnd,
            },
            openers: inside,
            closed,
          },)
        }`,
      },
    ],
    handled: inside,
  };
}

/**
 Finds the fences, display maths, code spans and inline maths of a parse that hold an opener of a comment that never
 closes, and writes what holds one outside the code.

 @param source - text the parse was read from, with its comments as they stand

 @param root - parse of the text by the plain reading with the site's math

 @param starts - index of the `<` of each opener of a comment that never closes

 @param edits - edits the other readers made, from which the opening mark of a lengthened fence is taken

 @param closed - comments that close

 @returns Edits that take what holds an opener out of its code or math, in text order, and the openers they write

 @example
 ```ts
 const split = openersInCodeOf({ source, root, starts: [9,], edits: [], closed: [], },);
 ```
 */
export function openersInCodeOf(
  {
    source,
    root,
    starts,
    edits,
    closed,
  }: {
    readonly source: string;
    readonly root: TreeNode;
    readonly starts: readonly number[];
    readonly edits: readonly TextEdit[];
    readonly closed: readonly TextRange[];
  },
): OpenerInCode {
  /**
   Nodes still to read with whether their parent is the root, a work stack so that a deep tree is no deep recursion.
   */
  const pending: {
    readonly node: TreeNode;
    readonly atTop: boolean
  }[] = (starts.length === 0)
    ? []
    : [
      {
        node: root,
        atTop: false,
      },
    ];
  /**
   Edits written.
   */
  const written: TextEdit[] = [];
  /**
   Openers the edits write.
   */
  const handled: number[] = [];
  for (let next = pending.pop(); next !== undefined; next = pending.pop()) {
    /**
     The node read.
     */
    const {
      node,
      atTop,
    } = next;
    if ('children' in node) {
      pending.push(...node.children
        .map(function child(under,): typeof next {
          return {
            node: under,
            atTop: node.type === 'root',
          };
        },),);
    }
    /**
     The node's type as a plain string, since the math nodes are not in the type vocabulary.
     */
    const type: string = node.type;
    if ((type !== 'code') && (type !== 'math')
      && (type !== 'inlineCode')
      && (type !== 'inlineMath'))
      continue;
    /**
     Where the node stands.
     */
    const range = rangeOf({ node, },);
    // Inline math that is display math to the strict grammar (`source-ships-display-math.ts`) is split as a display math is.
    if (((type === 'inlineCode') || (type === 'inlineMath')) && (!readsAsDisplayMath({
      source,
      range,
    },))) {
      /**
       What the span comes to.
       */
      const span = splitSpan({
        source,
        range,
        starts,
        closed,
      },);
      written.push(...span.edits,);
      handled.push(...span.handled,);
      continue;
    }
    /**
     Where its opening line ends.
     */
    const openingEnd = source.indexOf(
      '\n',
      range.start,
    );
    /**
     The mark that opens the block, none when it is indented code or the block starts with no mark.
     */
    const mark = openingMark({
      line: source.slice(
        range.start,
        (openingEnd === (-1)) ? source.length : openingEnd,
      ),
    },);
    if (mark === NO_MARK)
      continue;
    /**
     What stands before the mark on its line: the marks of the containers and the indentation, a list marker among them.
     */
    const before = source.slice(
      source.lastIndexOf(
        '\n',
        range.start - 1,
      ) + 1,
      range.start,
    );
    /**
     What every later line of the block carries: nothing for a block at the top, the marks of the containers otherwise, a
     list marker as spaces.
     */
    const carried = atTop
      ? ''
      : Array.from(
        before,
        function marked(character,): string {
          return CARRIED.includes(character,) ? character : ' ';
        },
      )
        .join('',);
    /**
     The edit made for the opening mark by another reader, which lengthens it.
     */
    const lengthened = edits.find(function lengthens({ start, },): boolean {
      return start === range.start;
    },);
    /**
     What the block comes to.
     */
    const split = splitFlow({
      source,
      range,
      mark: lengthened?.replacement ?? mark,
      carried,
      starts,
      closed,
    },);
    written.push(...split.edits,);
    handled.push(...split.handled,);
  }
  return {
    edits: written.toSorted(function byStart(
      left,
      right,
    ): number {
      return left.start - right.start;
    },),
    handled,
  };
}

/**
 Finds the blocks of raw HTML of a parse.

 @param root - parse of the text by the plain reading with the site's math

 @returns Where each block of raw HTML stands

 @example
 ```ts
 const ranges = htmlRangesOf({ root, },);
 ```
 */
function htmlRangesOf({ root, }: { readonly root: TreeNode; },): readonly TextRange[] {
  /**
   Ranges found.
   */
  const found: TextRange[] = [];
  /**
   Nodes still to read, a work stack so that a deep tree is no deep recursion.
   */
  const pending: TreeNode[] = [root,];
  for (let node = pending.pop(); node !== undefined; node = pending.pop()) {
    if ('children' in node)
      pending.push(...node.children,);
    if (node.type === 'html')
      found.push(rangeOf({ node, },),);
  }
  return found;
}

/**
 Merges the edits of the other readers with the openers' own and with what takes an opener out of code or math.

 @param source - text the parse was read from, with its comments as they stand

 @param root - parse of the text by the plain reading with the site's math

 @param starts - index of the `<` of each opener of a comment that never closes

 @param edits - edits the other readers made

 @param openers - edit that writes each opener as a character reference

 @param closed - comments that close

 @param footnotes - where a footnote definition stands, for a reader of the source and for the page

 @returns Every edit, in text order and not overlapping: the other readers' edits that fall outside what is rewritten, the
 openers not rewritten with it, and what is rewritten

 @example
 ```ts
 const all = editsWithOpeners({ source, root, starts: [9,], edits: [], openers, closed: [], footnotes: [], },);
 ```
 */
export function editsWithOpeners(
  {
    source,
    root,
    starts,
    edits,
    openers,
    closed,
    footnotes,
  }: {
    readonly source: string;
    readonly root: TreeNode;
    readonly starts: readonly number[];
    readonly edits: readonly TextEdit[];
    readonly openers: readonly TextEdit[];
    readonly closed: readonly TextRange[];
    readonly footnotes: readonly BlockRange[];
  },
): readonly TextEdit[] {
  /**
   What holds an opener inside a fence or display math under a footnote definition, written outside it, less the lines of
   the blocks of raw HTML a reader of the source reads, which are text to that reader.
   */
  const underFootnotes = openersInFootnoteFlowsOf({
    source,
    starts,
    closed,
    footnotes,
    htmlRanges: htmlRangesOf({ root, },),
  },);
  /**
   The openers the footnotes' lines took.
   */
  const { handled: taken, } = underFootnotes;
  /**
   Where the lines of a block of raw HTML that a footnote definition opens with stand: text on the page, in which the
   plain reading's code span or math is no code, so an opener there is written as the character reference like any other.
   */
  const rawHtml = rawHtmlRegionsOf({
    source,
    footnotes,
  },);
  /**
   What holds an opener inside code or math elsewhere, written outside it: the openers a footnote's lines took are left to
   them, since the plain reading may read the same lines as a code span or as indented code, and those in the lines of a
   block of raw HTML are left as text.
   */
  const atTop = openersInCodeOf({
    source,
    root,
    starts: starts.filter(function notTaken(start,): boolean {
      return (!taken.includes(start,)) && (!rawHtml.some(function holds(region,): boolean {
        return (region.start <= start) && (start < region.end);
      },));
    },),
    edits,
    closed,
  },);
  /**
   Both, together.
   */
  const written = {
    edits: [
      ...atTop.edits,
      ...underFootnotes.edits,
    ],
    handled: [
      ...atTop.handled,
      ...underFootnotes.handled,
    ],
  };
  return [
    ...edits.filter(function outside({ start, },): boolean {
      return !written.edits
        .some(function holds(edit,): boolean {
          return (start >= edit.start) && (start < edit.end);
        },);
    },),
    ...openers.filter(function notWritten({ end, },): boolean {
      return !written.handled
        .includes(end - 1,);
    },),
    ...written.edits,
  ].toSorted(function byStart(
    left,
    right,
  ): number {
    return left.start - right.start;
  },);
}

//endregion Source ships opener in code
