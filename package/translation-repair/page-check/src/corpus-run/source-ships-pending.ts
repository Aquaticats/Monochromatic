import type {
  BlockRange,
  TreeNode,
} from '@monochromatic-dev/translation-repair-document-structure/ts';
import {
  continuesDefinition,
  endLineOf,
  opensFootnote,
} from './source-ships-footnote.ts';
import { endsInLiteral, } from './source-ships-literal-end.ts';
import {
  rangeOf,
  type TextRange,
} from './source-ships-ranges.ts';

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
 Takes the paragraph that goes on right after a footnote definition off the nodes
 still to read, so that it is read with the definition.

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

/**
 How far a footnote definition at the top reaches, and the state of the top once it is read.
 */
export type FootnoteReach = {
  /**
   Where the lines the footnote's body is read from end.
   */
  readonly end: number;

  /**
   State of the top once the footnote is read.
   */
  readonly state: FootnoteState;
};

/**
 The most columns of indentation a footnote definition may start after.
 */
const MAX_DEFINITION_INDENT = 3;

/**
 Whether a footnote definition starts at a block: at its first character, or after the few spaces that a
 definition may start after.

 @param source - text

 @param range - where the block stands

 @param start - where the footnote definition starts

 @returns Whether the footnote starts at the block

 @example
 ```ts
 const same = startsAtBlock({ source: '[^1]: x', range: { start: 0, end: 7, }, start: 0, },);
 ```
 */
function startsAtBlock(
  {
    source,
    range,
    start,
  }: {
    readonly source: string;
    readonly range: TextRange;
    readonly start: number;
  },
): boolean {
  if ((start < range.start) || (start > (range.start + MAX_DEFINITION_INDENT)))
    return false;
  /**
   What stands between the block's first character and the footnote's.
   */
  const between = source.slice(
    range.start,
    start,
  );
  return between.trim() === '';
}

/**
 Takes the blocks of the root that start before a place off the nodes still to read.

 @param pending - nodes still to read, the next one on top; the blocks taken are removed from it

 @param end - place the blocks must start before

 @returns The blocks taken, in text order

 @example
 ```ts
 const taken = takeRootBlocksBefore({ pending, end: 12, },);
 ```
 */
function takeRootBlocksBefore(
  {
    pending,
    end,
  }: {
    readonly pending: Pending[];
    readonly end: number;
  },
): readonly TreeNode[] {
  /**
   Blocks taken.
   */
  const taken: TreeNode[] = [];
  for (let next = pending.at(-1,); next !== undefined; next = pending.at(-1,)) {
    if (!next.isRootChild)
      break;
    /**
     Where the block stands.
     */
    const { start, } = rangeOf({ node: next.node, },);
    if (start >= end)
      break;
    pending.pop();
    taken.push(next.node,);
  }
  return taken;
}

/**
 Takes the blocks the strict grammar reads as part of a footnote definition at the top off the nodes still to read, so
 that they are read with the definition as the footnote's own text.

 The strict grammar ends the footnote where a parse with GFM does; the plain reading goes on in paragraphs, raw HTML
 and definitions that the footnote's paragraph takes for lazy lines, and a block that starts inside the footnote is
 part of it.

 @param pending - nodes still to read, the next one on top; the blocks the footnote reaches are removed from it

 @param footnotes - where the strict grammar reads a footnote definition in the whole text

 @param source - text

 @param node - block that opens the footnote

 @param range - where the footnote starts in the block, and where the block ends

 @param state - state of the top once the block is read

 @returns Where the footnote's lines end and the state of the top, the state given when no footnote starts at the block

 @example
 ```ts
 const reach = takeFootnote({ pending, footnotes, source, node, range, state, },);
 ```
 */
export function takeFootnote(
  {
    pending,
    footnotes,
    source,
    node,
    range,
    state,
  }: {
    readonly pending: Pending[];
    readonly footnotes: readonly BlockRange[];
    readonly source: string;
    readonly node: TreeNode;
    readonly range: TextRange;
    readonly state: FootnoteState;
  },
): FootnoteReach {
  /**
   The footnote the strict grammar starts at the block.
   */
  const footnote = footnotes.find(function opensHere({ start, },): boolean {
    return startsAtBlock({
      source,
      range,
      start,
    },);
  },);
  if (footnote === undefined) {
    /**
     The block the definition's lines go on in: the paragraph right after a definition, else the block itself.
     */
    const continued = (node.type === 'definition')
      ? takeContinuation({
        pending,
        definition: node,
      },)
      : node;
    /**
     Where the block the lines go on in ends.
     */
    const { end: continuedEnd, } = rangeOf({ node: continued, },);
    return {
      end: continuedEnd,
      state,
    };
  }
  /**
   The blocks the footnote reaches into.
   */
  const taken = takeRootBlocksBefore({
    pending,
    end: footnote.end,
  },);
  /**
   Where each of them ends.
   */
  const ends = taken.map(function endOf(block,): number {
    /**
     Where the block ends.
     */
    const { end, } = rangeOf({ node: block, },);
    return end;
  },);
  return {
    end: Math.max(
      range.end,
      footnote.end,
      ...ends,
    ),
    state: {
      open: false,
      endLine: endLineOf({
        node: taken.at(-1,) ?? node,
        fallback: state.endLine,
      },),
      endsInLiteral: false,
    },
  };
}

/**
 Lists the blocks at the top of a text as the nodes still to read, the last of them first so they come out in order.

 @param root - parse of the text

 @returns Nodes still to read, none when the root holds no children

 @example
 ```ts
 const pending = topLevelPending({ root: parseMarkdownMathBody({ body: source, },), },);
 ```
 */
export function topLevelPending({ root, }: { readonly root: TreeNode; }): Pending[] {
  if (!('children' in root))
    return [];
  return root.children
    .map(function topLevel(node,): Pending {
      return {
        node,
        isRootChild: true,
        insideLink: false,
        parentType: 'root',
        parent: root,
      };
    },)
    .toReversed();
}

//endregion Source ships pending
