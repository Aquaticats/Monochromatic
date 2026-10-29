import type { TreeNode, } from '@monochromatic-dev/translation-repair-document-structure/ts';
import { contentStretches, } from './source-ships-lines.ts';
import { fenceLengthEdits, } from './source-ships-fenced.ts';
import { afterLiteralBlockEdits, } from './source-ships-separation.ts';
import {
  FENCE_MARKS,
  startsAny,
  type TextEdit,
  type TextRange,
} from './source-ships-ranges.ts';

//region Source ships literal block
// A BLOCK OF RAW HTML OR OF CODE, read for what the page must do with its lines. MDX has no raw HTML blocks and no
// indented code: it reads such lines as a paragraph, and the reader sees them as they stand, so their stretches are
// literal and the line after the block gets a blank line where the block ends at a line of its own. A fence is code to
// MDX too, and is copied, lengthened where its code holds a mark MDX would close it at.

/**
 What a block of raw HTML or code comes to for the page.
 */
export type LiteralBlockReading = {
  /**
   Stretches a reader sees as they stand, in which every character Markdown reads as syntax is escaped.
   */
  readonly literal: readonly TextRange[];

  /**
   Stretches rewritten.
   */
  readonly edits: readonly TextEdit[];
};

/**
 Reads a block of raw HTML or code.

 @param source - text

 @param readable - whole text with its comments blanked, the parse's own input

 @param node - `html` or `code` node of the parse

 @param range - where the node stands

 @param isRootChild - whether the node stands at the top of the text

 @returns Stretches read as they stand and rewrites, none of the first for a fence

 @example
 ```ts
 const { literal, edits, } = literalBlockReading({ source, readable, node, range, isRootChild: true, },);
 ```
 */
export function literalBlockReading(
  {
    source,
    readable,
    node,
    range,
    isRootChild,
  }: {
    readonly source: string;
    readonly readable: string;
    readonly node: TreeNode;
    readonly range: TextRange;
    readonly isRootChild: boolean;
  },
): LiteralBlockReading {
  if ((node.type === 'code') && startsAny({
    text: source,
    at: range.start,
    candidates: FENCE_MARKS,
  },)) {
    return {
      literal: [],
      edits: fenceLengthEdits({
        source,
        range,
      },),
    };
  }
  return {
    literal: contentStretches({
      source,
      node,
      range,
    },),
    edits: afterLiteralBlockEdits({
      readable,
      range,
      inContainer: !isRootChild,
    },),
  };
}

//endregion Source ships literal block
