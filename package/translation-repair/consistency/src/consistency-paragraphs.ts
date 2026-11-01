import type { Nodes, } from 'mdast';

import { withDefinitions, } from '@monochromatic-dev/translation-repair-shared-contract/ts';
import {
  type DeepReadonlyData,
  parseSliceBody,
  requireMdxRefusal,
} from '@monochromatic-dev/translation-repair-document-structure/ts';

//region Consistency paragraphs
// WHERE THE BLOCKS OF RUNNING TEXT OF AN ENGLISH ARE, by offset: its paragraphs and the text of its headings. The refusal run
// beside a span (`markPastRunOf`, `consistency-closing-marks.ts`) steps over a single line break only inside such a block, where
// the break renders as a space or as a break between two marks. A heading spans lines only when it is a setext heading (a
// line of text and an underline of `=` or `-`); the text of a heading ends before its underline, so the break before the
// underline is outside it, and the break after a heading of one line is too. A line break between a paragraph and a list item,
// a heading, a quotation or a table is the end of the paragraph, even when the marker of the block that follows is a unit the
// prose does not read as words. The blocks come from the same parse as the prose (`parse-slice-body.ts`), except for one
// thing: a line of only one invisible or non-ASCII space character (an ideographic space, a zero-width space, a no-break space)
// is no blank line to CommonMark, which the site renders, so the blocks are read with `invisibleLines: 'text'` and the
// paragraph goes on through such a line, where the page assembly turns it into spaces and ends the paragraph (n156, W9). A
// paragraph node starts
// and ends where the parser put it, and the text of a heading runs from the start of its first inline node to the end of its
// last, so a break at an offset inside that range is one the block holds. Each block also knows how many quotations enclose it
// (`quoteDepthAt`), which is how many `>` markers the run may step over at the start of each of its lines. AN ENGLISH THE
// GRAMMAR REFUSES has no tree to ask, and is read as text (`consistency-prose-text.ts`): every line break is then taken to be
// inside a block, and every line start to hold as many markers as a line can, so the run holds at least what any parse would
// hold and keeps a mark from being carried beside a mark. At the pass the receiving places of such an English keep their
// words whatever stands beside them (read on whole pages in `corpus-run/pass-consistency-shipped-marks.unit.test.ts`), so the
// pass carries no mark there.

/**
 A node of the tree the slice grammar reads, which it hands over read-only.
 */
type TreeNode = DeepReadonlyData<Nodes>;

/**
 A node of the tree with the number of quotations around it.
 */
type EnclosedNode = {
  /**
   The node.
   */
  readonly node: TreeNode;

  /**
   How many quotations enclose the node.
   */
  readonly depth: number;
};

/**
 One block of running text of the English: a paragraph, or the text of a heading.
 */
type TextBlockRange = {
  /**
   Offset where the block starts.
   */
  readonly start: number;

  /**
   Exclusive offset where it ends, which is the offset of the line break that follows it.
   */
  readonly end: number;

  /**
   How many quotations enclose the block.
   */
  readonly quoteDepth: number;
};

/**
 What one parse of an English says of its blocks.
 */
type TextBlocksRead = {
  /**
   Whether the grammar refused the English, so it has no blocks to tell.
   */
  readonly refused: boolean;

  /**
   The blocks, in ascending order of their start; none overlap.
   */
  readonly ranges: readonly TextBlockRange[];
};

/**
 One English whose blocks were read.
 */
type RememberedTextBlocks = {
  /**
   The English the blocks were read from.
   */
  readonly english: string;

  /**
   The definitions it was read with.
   */
  readonly definitions: string;

  /**
   What the parse said of them.
   */
  readonly read: TextBlocksRead;
};

/**
 How many English texts `textBlocksOf` remembers: the places of one slice ask about the one English, and it is parsed once.
 */
const REMEMBERED_TEXT_BLOCKS = 4;

/**
 The blocks of the most recent English texts, the newest last. They hold only what is read from the English, so remembering them
 changes no answer.
 */
const rememberedTextBlocks: RememberedTextBlocks[] = [];

/**
 The block that runs from the start of one node of the parse to the end of another, none where the parser gave either no offset.

 @param first - node the block starts with

 @param last - node the block ends with, the same node for a paragraph

 @param depth - how many quotations enclose the block

 @returns The block as a list of one, none when an offset is absent

 @example
 ```ts
 const [block,] = blockBetween({ first: node, last: node, depth: 1, },);
 ```
 */
function blockBetween(
  {
    first,
    last,
    depth,
  }: {
    readonly first: TreeNode;
    readonly last: TreeNode;
    readonly depth: number;
  },
): readonly TextBlockRange[] {
  /**
   Where the first node starts, as an offset of the parse.
   */
  const start = first.position
    ?.start
    .offset;
  /**
   Where the last node ends, as an offset of the parse.
   */
  const end = last.position
    ?.end
    .offset;
  return ((start === undefined) || (end === undefined))
    ? []
    : [
      {
        start,
        end,
        quoteDepth: depth,
      },
    ];
}

/**
 The block of running text a node is, none for any other node: a paragraph where the parser put it, a heading from the start
 of its first inline node to the end of its last, since the position of a setext heading holds its underline.

 @param node - the node asked about

 @param depth - how many quotations enclose it

 @returns The block as a list of one, none when the node is no such block, has no position or holds no text

 @example
 ```ts
 const [block,] = textBlockOf({ node, depth: 1, });
 ```
 */
function textBlockOf(
  {
    node,
    depth,
  }: {
    readonly node: TreeNode;
    readonly depth: number;
  },
): readonly TextBlockRange[] {
  if (node.type === 'paragraph') {
    return blockBetween({
      first: node,
      last: node,
      depth,
    },);
  }
  if (node.type === 'heading') {
    /**
     The first inline node of the heading.
     */
    const first = node.children
      .at(0,);
    /**
     The last inline node of the heading.
     */
    const last = node.children
      .at(-1,);
    if ((first === undefined) || (last === undefined))
      return [];
    return blockBetween({
      first,
      last,
      depth,
    },);
  }
  return [];
}

/**
 The blocks of running text of a tree, walked with a work stack rather than by recursion, since a tree is as deep as the nesting a
 page writes.

 @param root - the tree's top node

 @returns The paragraphs and the texts of the headings, in ascending order of their start

 @example
 ```ts
 const ranges = textBlockRangesOf({ root, });
 ```
 */
function textBlockRangesOf({ root, }: { readonly root: TreeNode; },): readonly TextBlockRange[] {
  /**
   Blocks met so far.
   */
  const met: TextBlockRange[] = [];
  /**
   Nodes still to visit, each with the number of quotations around it.
   */
  const pending: EnclosedNode[] = [
    {
      node: root,
      depth: 0,
    },
  ];
  // Each node in turn, until the stack is empty.
  for (let item = pending.pop(); item !== undefined; item = pending.pop()) {
    /**
     The node and the quotations around it.
     */
    const {
      node,
      depth,
    } = item;
    met.push(...textBlockOf({
      node,
      depth,
    },),);
    if ('children' in node) {
      /**
       How many quotations enclose the children: one more inside a quotation.
       */
      const childDepth = (node.type === 'blockquote') ? (depth + 1) : depth;
      for (const child of node.children as readonly TreeNode[]) {
        pending.push({
          node: child,
          depth: childDepth,
        },);
      }
    }
  }
  return met.toSorted(function byStart(
    left,
    right,
  ): number {
    return left.start - right.start;
  },);
}

/**
 What the parse of an English says of its blocks of running text, parsed once for each of the English texts most recently asked
 about.

 @param english - the English of a slice

 @param definitions - the page's definitions (`pageDefinitionsOf`), parsed after the English so a reference in it resolves; empty
 for an English read alone

 @returns Whether the grammar refused it, and the blocks otherwise

 @example
 ```ts
 const { refused, ranges, } = textBlocksOf({ english: 'Fill the bowl.\n\nThen nap.', definitions: '', });
 ```
 */
function textBlocksOf(
  {
    english,
    definitions,
  }: {
    readonly english: string;
    readonly definitions: string;
  },
): TextBlocksRead {
  /**
   The blocks read before for an equal English and equal definitions, none when there are none.
   */
  const known = rememberedTextBlocks.find(function isSame(entry,): boolean {
    return (entry.english === english) && (entry.definitions === definitions);
  },);
  if (known !== undefined)
    return known.read;
  /**
   What the parse says, which is that the grammar refused the English unless a tree comes back.
   */
  const read = (function parsed(): TextBlocksRead {
    try {
      /**
       The parse of the English, read as a renderer reads it: a line of only an invisible or a non-ASCII space character is no
       blank line, so the paragraph goes on through it (n156, W9).
       */
      const { root, } = parseSliceBody({
        text: withDefinitions({
          text: english,
          definitions,
        },),
        invisibleLines: 'text',
      },);
      return {
        refused: false,
        ranges: textBlockRangesOf({ root, },),
      };
    }
    catch (error) {
      // Only the grammar's own refusal is an English to be read as text; anything else keeps propagating.
      requireMdxRefusal({ error, },);
      return {
        refused: true,
        ranges: [],
      };
    }
  })();
  rememberedTextBlocks.push({
    english,
    definitions,
    read,
  },);
  if (rememberedTextBlocks.length > REMEMBERED_TEXT_BLOCKS)
    rememberedTextBlocks.shift();
  return read;
}

/**
 The block of running text that holds an offset, found by halving the blocks until one is left to ask.

 @param ranges - the blocks, in ascending order of their start

 @param index - the offset asked about

 @returns The block as a list of one, none when no block holds the offset

 @example
 ```ts
 const [block,] = blockHolding({ ranges, index: 8, });
 ```
 */
function blockHolding(
  {
    ranges,
    index,
  }: {
    readonly ranges: readonly TextBlockRange[];
    readonly index: number;
  },
): readonly TextBlockRange[] {
  /**
   The window of blocks still in question.
   */
  const window = {
    low: 0,
    high: ranges.length,
  };
  while (window.low < window.high) {
    /**
     The middle block of the window.
     */
    const middle = Math.floor((window.low + window.high) / 2);
    /**
     The block at the middle.
     */
    const block = ranges[middle];
    if (block === undefined)
      return [];
    if (index < block.start) {
      window.high = middle;
      continue;
    }
    if (index >= block.end) {
      window.low = middle + 1;
      continue;
    }
    return [block,];
  }
  return [];
}

/**
 Whether an offset of an English lies inside one of its blocks of running text, a paragraph or the text of a setext heading. For
 an English the grammar refuses, which has no blocks to tell, every offset does.

 @param english - the English of a slice

 @param index - the offset asked about, which is a line break in the one question the pass asks

 @param definitions - the page's definitions (`pageDefinitionsOf`), parsed after the English so a reference in it resolves; empty
 for an English read alone

 @returns Whether the offset is at or after the start of a block and before its end

 @example
 ```ts
 const inside = insideParagraphAt({ english: 'Fill the\nbowl.\n\nThen nap.', index: 8, },); // true
 ```
 */
export function insideParagraphAt(
  {
    english,
    index,
    definitions = '',
  }: {
    readonly english: string;
    readonly index: number;
    readonly definitions?: string;
  },
): boolean {
  /**
   What the parse says of the blocks.
   */
  const {
    refused,
    ranges,
  } = textBlocksOf({
    english,
    definitions,
  },);
  if (refused)
    return true;
  /**
   The block that holds the offset, as a list of zero or one.
   */
  const holding = blockHolding({
    ranges,
    index,
  },);
  return holding.length > 0;
}

/**
 How many quotations enclose the block of running text that holds an offset, which is how many `>` markers each line of the
 block may start with: none for a block outside every quotation, one for each quotation around it, however a list item or
 another quotation stands between them. An offset no block holds has none; an English the grammar refuses has no blocks to tell
 and is taken for as deep as a quotation can be.

 @param english - the English of a slice

 @param index - the offset asked about, which is a line break in the one question the pass asks

 @param definitions - the page's definitions (`pageDefinitionsOf`), parsed after the English so a reference in it resolves; empty
 for an English read alone

 @returns The number of quotations, `Number.POSITIVE_INFINITY` for an English the grammar refuses

 @example
 ```ts
 const depth = quoteDepthAt({ english: '> > Fill the\n> > bowl.', index: 12, },); // 2
 ```
 */
export function quoteDepthAt(
  {
    english,
    index,
    definitions = '',
  }: {
    readonly english: string;
    readonly index: number;
    readonly definitions?: string;
  },
): number {
  /**
   What the parse says of the blocks.
   */
  const {
    refused,
    ranges,
  } = textBlocksOf({
    english,
    definitions,
  },);
  if (refused)
    return Number.POSITIVE_INFINITY;
  /**
   The block that holds the offset, as a list of zero or one.
   */
  const [holding,] = blockHolding({
    ranges,
    index,
  },);
  return holding?.quoteDepth ?? 0;
}

//endregion Consistency paragraphs
