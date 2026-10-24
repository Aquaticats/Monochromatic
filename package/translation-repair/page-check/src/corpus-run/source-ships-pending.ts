import type {
  TreeNode,
} from '@monochromatic-dev/translation-repair-document-structure/ts';
import {
  continuesDefinition,
  endLineOf,
  opensFootnote,
} from './source-ships-footnote.ts';
import { endsInLiteral, } from './source-ships-literal-end.ts';
import type { TextRange, } from './source-ships-ranges.ts';

//region Source ships pending
// THE STATE THE SPANS READER KEEPS while it walks the nodes of a parse: a node still to read with where it stands, and,
// for each container, whether the block read last in it is part of a footnote definition. The state of a container is
// replaced when a block is read in it, never changed in place.

/**
 A node still to read, and where it stands.
 */
export type Pending = {
  /**
   The node.
   */
  readonly node: TreeNode;

  /**
   Whether its parent is the root of the text being read.
   */
  readonly isRootChild: boolean;

  /**
   Whether it stands inside the text of a link, where a link cannot be written.
   */
  readonly insideLink: boolean;

  /**
   Type of its parent, which says whether it stands among blocks or among inline content.
   */
  readonly parentType: string;

  /**
   Its parent, which keeps whether the block read last in it is part of a footnote definition.
   */
  readonly parent: TreeNode;
};

/**
 Whether the block read last in a container is part of a footnote definition, which a line right after it continues.
 */
export type FootnoteState = {
  /**
   Whether the block is the first line of a footnote definition, or a block indented under one.
   */
  readonly open: boolean;

  /**
   Line the block ends on.
   */
  readonly endLine: number;

  /**
   Whether the block ends in a raw HTML block or indented code, after which a definition line is a lazy line too.
   */
  readonly endsInLiteral: boolean;
};

/**
 The state of a container nothing has been read in.
 */
const NO_BLOCK_READ: FootnoteState = {
  open: false,
  endLine: 0,
  endsInLiteral: false,
};

/**
 What the block read last in a container was.

 @param footnotes - state of each container a block was read in

 @param parent - container

 @returns Container's state, that of a container nothing has been read in when no block was

 @example
 ```ts
 const state = footnoteStateOf({ footnotes: new Map(), parent: root, },);
 ```
 */
export function footnoteStateOf(
  {
    footnotes,
    parent,
  }: {
    readonly footnotes: ReadonlyMap<TreeNode, FootnoteState>;
    readonly parent: TreeNode;
  },
): FootnoteState {
  return footnotes.get(parent,) ?? NO_BLOCK_READ;
}

/**
 What a container's state is once a block has been read in it.

 @param before - container's state before the block

 @param source - text

 @param node - block

 @param range - where the block stands

 @param continuesFootnote - whether the block is indented code under a footnote definition, which the definition's own
 blocks continue

 @returns State with the block as the one read last

 @example
 ```ts
 const after = stateAfter({ before, source, node, range, continuesFootnote: false, },);
 ```
 */
export function stateAfter(
  {
    before,
    source,
    node,
    range,
    continuesFootnote,
  }: {
    readonly before: FootnoteState;
    readonly source: string;
    readonly node: TreeNode;
    readonly range: TextRange;
    readonly continuesFootnote: boolean;
  },
): FootnoteState {
  return {
    open: continuesFootnote || opensFootnote({
      source,
      node,
      range,
    },),
    endLine: endLineOf({
      node,
      fallback: before.endLine,
    },),
    endsInLiteral: endsInLiteral({
      source,
      node,
    },),
  };
}

/**
 Takes the paragraph that goes on right after a footnote definition off the nodes still to read, so that it is read with
 the definition.

 @param pending - nodes still to read, the next one on top; the paragraph is removed from it when there is one

 @param definition - footnote definition about to be read

 @returns The paragraph taken, which the definition's lines go on in, or the definition itself when none follows it

 @example
 ```ts
 const last = takeContinuation({ pending, definition, },);
 ```
 */
export function takeContinuation(
  {
    pending,
    definition,
  }: {
    readonly pending: Pending[];
    readonly definition: TreeNode;
  },
): TreeNode {
  /**
   The node that comes next in the text.
   */
  const following = pending.at(-1,);
  if (following === undefined)
    return definition;
  if (!continuesDefinition({
    definition,
    block: following.node,
  },))
    return definition;
  pending.pop();
  return following.node;
}

//endregion Source ships pending
