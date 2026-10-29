import {
  footnoteDefinitionRangesOf,
  pageFootnoteDefinitionRangesOf,
  type TreeNode,
} from '@monochromatic-dev/translation-repair-document-structure/ts';
import {
  rangeOf,
  type TextEdit,
  type TextRange,
} from './source-ships-ranges.ts';

//region Source ships html interrupt
// A BLOCK OF RAW HTML THAT STARTS FIRST ON A LINE RIGHT AFTER A PARAGRAPH. A reader of the source takes a line that starts
// an HTML block of the kinds that may interrupt a paragraph (an element of the block list such as a `div`, a `pre` or a
// `script`, a processing instruction, a declaration, a comment, among them one that never closes) for the start of that
// block, which cuts the paragraph short even with no blank line between them, so a backtick or a dollar sign in the
// paragraph that nothing closes opens no code span or math there. The page has no raw HTML: the lines of the block are
// written as the text of a paragraph (`source-ships-spans.ts`, and the opener of a comment that never closes as a character
// reference, `source-ships-encoding.ts`), so the paragraph goes on, and the backtick pairs with one in the lines of the
// block, which the page then shows as a code span the source does not have. A blank line before the line of the block ends
// the paragraph on the page where it ends for the reader of the source, in a block quote or a list item as at the top, the
// blank line then carrying the marks of the container. The paragraph the block cuts is the last block of whatever stands
// right before it, however deep: a block at column zero after a paragraph of a block quote or a list item stands outside
// them, since a line of a block of raw HTML cannot be a lazy line, and the paragraph it cuts is in the container before it. A paragraph that a footnote definition holds is left as it is: the
// page is arranged to go on with a footnote over a line of raw HTML (`document-structure/src/gfm-block-ranges.ts`).

/**
 Characters a line of a block quote or a list item opens with before its content.
 */
const CONTAINER_MARKS = ' \t>';

/**
 Node types whose children are blocks, among which a paragraph may stand right before a block of raw HTML.
 */
const CONTAINERS: ReadonlySet<string> = new Set([
  'root',
  'blockquote',
  'list',
  'listItem',
]);

/**
 Whether a block is a paragraph that ends on the line right before the one a block of raw HTML starts on.

 @param before - block before the raw HTML block

 @param block - block of raw HTML

 @returns Whether a paragraph stands right before the block with no blank line between them

 @example
 ```ts
 const adjacent = endsRightBefore({ before, block, },);
 ```
 */
function endsRightBefore(
  {
    before,
    block,
  }: {
    readonly before: TreeNode;
    readonly block: TreeNode;
  },
): boolean {
  if (before.type !== 'paragraph')
    return false;
  if ((before.position === undefined) || (block.position === undefined))
    return false;
  /**
   Where the paragraph ends.
   */
  const { end: paragraphEnd, } = before.position;
  /**
   Where the block of raw HTML starts.
   */
  const { start: blockStart, } = block.position;
  return (paragraphEnd.line + 1) === blockStart.line;
}

/**
 The last block of a block and, when that is a container, of its last child, down to a block that holds no blocks.

 @param block - block that stands right before another

 @returns Block the first one ends with: itself for a block that is no container, the last block of the last item of a
 list, the last block of a block quote

 @example
 ```ts
 const last = lastBlockOf({ block: list, },);
 ```
 */
function lastBlockOf({ block, }: { readonly block: TreeNode; },): TreeNode {
  /**
   The block read last, which the walk goes down from.
   */
  const reading = { node: block, };
  for (;;) {
    /**
     The block the walk is at.
     */
    const { node, } = reading;
    if ((!CONTAINERS.has(node.type,)) || (!('children' in node)))
      break;
    /**
     The last block the container holds.
     */
    const last = node
      .children
      .at(-1,);
    if (last === undefined)
      break;
    reading.node = last;
  }
  return reading.node;
}

/**
 The edit that doubles the line ending before the line a block of raw HTML starts on.

 @param readable - whole text, with its comments blanked

 @param range - where the block of raw HTML stands

 @returns Edit that writes a blank line, carrying the marks of the container, before the block; none when the block
 starts the text or something other than the marks of a container stands before it on its line

 @example
 ```ts
 const edits = blankLineBefore({ readable: 'a `x\n<div>', range: { start: 5, end: 10, }, },);
 ```
 */
function blankLineBefore(
  {
    readable,
    range,
  }: {
    readonly readable: string;
    readonly range: TextRange;
  },
): readonly TextEdit[] {
  /**
   Where the line of the block starts, after the line ending that ends the line before it.
   */
  const lineStart = readable.lastIndexOf(
    '\n',
    range.start - 1,
  ) + 1;
  if (lineStart === 0)
    return [];
  /**
   What stands before the block on its line: the marks of its containers and its indentation.
   */
  const marks = readable.slice(
    lineStart,
    range.start,
  );
  for (let at = 0; at < marks.length; at += 1) {
    if (!CONTAINER_MARKS.includes(marks.charAt(at,),))
      return [];
  }
  /**
   The line ending, with its carriage return when the text has one.
   */
  const ending = readable.startsWith(
    '\r\n',
    lineStart - 2,
  )
    ? '\r\n'
    : '\n';
  return [
    {
      start: lineStart - ending.length,
      end: lineStart,
      replacement: `${ending}${marks.trimEnd()}${ending}`,
    },
  ];
}

/**
 What cutting the paragraphs before blocks of raw HTML comes to.
 */
export type HtmlInterruption = {
  /**
   The edits that double the line endings.
   */
  readonly edits: readonly TextEdit[];

  /**
   Where each block of raw HTML cut off starts, which starts a run of text of its own on the page: a bracket opened in the
   paragraph before does not reach into it.
   */
  readonly starts: readonly number[];
};

/**
 Finds the blocks of raw HTML that cut a paragraph and writes the blank line that cuts it on the page too.

 @param readable - whole text with its comments blanked, the parse's own input

 @param root - parse of the text by the plain reading with the site's math, with the tails of its footnotes read on their
 own

 @returns Edits that double the line ending before each block of raw HTML that starts on the line right after a paragraph
 of the same container, in text order, and where those blocks start; none for a paragraph that a footnote definition holds

 @example
 ```ts
 const edits = htmlInterruptionEdits({ readable: 'a `x\n<div>\ny`', root, },);
 ```
 */
export function htmlInterruptionEdits(
  {
    readable,
    root,
  }: {
    readonly readable: string;
    readonly root: TreeNode;
  },
): HtmlInterruption {
  /**
   The blocks of raw HTML that stand right after a paragraph, each with that paragraph.
   */
  const cutting: {
    readonly block: TreeNode;
    readonly paragraph: TreeNode;
  }[] = [];
  /**
   Nodes still to read, a work stack so that a deep tree is no deep recursion.
   */
  const pending: TreeNode[] = [root,];
  for (let node = pending.pop(); node !== undefined; node = pending.pop()) {
    if ((!CONTAINERS.has(node.type,)) || (!('children' in node)))
      continue;
    /**
     The blocks the container holds.
     */
    const { children, } = node;
    pending.push(...children,);
    for (const [at, block,] of children.entries()) {
      /**
       The block right before this one.
       */
      const before = children[at - 1];
      if (block.type !== 'html')
        continue;
      if (before === undefined)
        continue;
      /**
       The block the one before ends with, a paragraph of a block quote or a list item among them.
       */
      const paragraph = lastBlockOf({ block: before, },);
      if (!endsRightBefore({
        before: paragraph,
        block,
      },))
        continue;
      cutting.push({
        block,
        paragraph,
      },);
    }
  }
  if (cutting.length === 0) {
    return {
      edits: [],
      starts: [],
    };
  }
  /**
   Where a footnote definition stands for a reader of the source and for the page.
   */
  const footnotes = [
    ...footnoteDefinitionRangesOf({ body: readable, },),
    ...pageFootnoteDefinitionRangesOf({ body: readable, },),
  ];
  /**
   The blocks of raw HTML cut off from a paragraph that no footnote definition holds.
   */
  const cut = cutting.filter(function heldByNoFootnote({ paragraph, },): boolean {
    /**
     Where the paragraph stands.
     */
    const {
      start,
      end,
    } = rangeOf({ node: paragraph, },);
    return !footnotes.some(function holdsParagraph({
      start: from,
      end: to,
    },): boolean {
      return (from < end) && (start < to);
    },);
  },);
  /**
   The edits for those blocks, each with the block it was made for.
   */
  const written = cut.map(function blankLine({ block, },): {
    readonly edits: readonly TextEdit[];
    readonly start: number;
  } {
    /**
     Where the block stands.
     */
    const range = rangeOf({ node: block, },);
    return {
      edits: blankLineBefore({
        readable,
        range,
      },),
      start: range.start,
    };
  },);
  return {
    edits: written
      .flatMap(function editsOf({ edits, },): readonly TextEdit[] {
        return edits;
      },)
      .toSorted(function byStart(
        left,
        right,
      ): number {
        return left.start - right.start;
      },),
    starts: written
      .filter(function cutAtAll({ edits, },): boolean {
        return edits.length > 0;
      },)
      .map(function startOf({ start, },): number {
        return start;
      },),
  };
}

//endregion Source ships html interrupt
