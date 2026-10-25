import type {
  BlockRange,
  TreeNode,
} from '@monochromatic-dev/translation-repair-document-structure/ts';
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
// whole span is written as text. An opener in a fence or display math that a footnote definition holds at the top is
// found in `source-ships-opener-footnote.ts`, since the plain reading takes the lines of that fence for indented code. An
// opener first on a line right after a paragraph is the start of a block of raw HTML that ends the paragraph to a reader of
// the source, and `source-ships-opener-interrupt.ts` puts a blank line before it.

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
  return {
    edits: [
      {
        start: keepsCode ? first : range.start,
        end: range.end,
        replacement: `${keepsCode ? delimiter : ''}${
          writtenAsText({
            source,
            range: {
              start: keepsCode ? first : (range.start + delimiter.length),
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
    if ((type === 'inlineCode') || (type === 'inlineMath')) {
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
   What holds an opener inside code or math, written outside it.
   */
  const atTop = openersInCodeOf({
    source,
    root,
    starts,
    edits,
    closed,
  },);
  /**
   What holds an opener inside a fence or display math under a footnote definition, written outside it.
   */
  const underFootnotes = openersInFootnoteFlowsOf({
    source,
    root,
    starts,
    closed,
    footnotes,
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
