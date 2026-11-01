import type { ChunkPair, } from './chunk-document.ts';
import { isInsertionChunk, } from '@monochromatic-dev/translation-repair-shared-contract/ts';
import { wordForCount, } from '@monochromatic-dev/translation-repair-typography/ts';
import type { DocumentNode, } from '@monochromatic-dev/translation-repair-document-structure/ts';

//region Span contiguity
// The one thing a span's own text cannot prove: that it covers what it claims to
// cover.
//
// A content chunk carries `nodes`, two offsets and the text between them, and
// the offsets are taken from the FIRST and LAST node of its run. Where a run is
// built by filtering, a document node lying between two of its members but
// absent from it is still inside the resulting range: the text is sliced from
// the offsets, so it contains that node's bytes, and the text-agreement check in
// `placement-layout.ts` passes byte for byte while `nodes` omits a whole block.
//
// WHAT THAT COSTS is silent and total for the omitted block: assembly writes
// over the offset range, so a replacement decided without ever seeing that
// block replaces it anyway. Nothing downstream can notice, because every later
// reader trusts the range.
//
// SO IT IS CHECKED AGAINST THE DOCUMENT rather than against the chunk. The rule
// is that a content chunk's nodes are exactly the document nodes its range
// contains: not a subset chosen by whoever built the run, and not a superset
// reaching outside the offsets.

/**
 Raised when a span claims a range it does not cover node for node.

 @example
 ```ts
 throw new SpanContiguityError({ message: 'slice 3 covers a block it does not carry', },);
 ```
 */
export class SpanContiguityError extends Error {
  /**
   Builds the failure naming the slice and what its range holds.

   @param message - what the range contains that the nodes do not

   @example
   ```ts
   throw new SpanContiguityError({ message: 'slice 3 covers 3 blocks and carries 2', },);
   ```
   */
  public constructor({ message, }: { readonly message: string; },) {
    super(message,);
    this.name = 'SpanContiguityError';
  }
}

/**
 Reads the document nodes one half-open range touches at all.

 TOUCHES RATHER THAN CONTAINS, which is the difference between this and what
 it replaced. Counting only whole nodes made a range cutting through one
 invisible: the straddled block is not inside, so it is not counted, and a
 span carrying nothing across a range covering half a paragraph agreed with
 itself. Assembly still writes over that half.

 @param nodes - every node of the document, in order

 @param startOffset - absolute start of the range

 @param endOffset - absolute exclusive end

 @returns Nodes sharing any character with the range

 @example
 ```ts
 const touched = nodesTouching({ nodes, startOffset, endOffset, },);
 ```
 */
function nodesTouching(
  {
    nodes,
    startOffset,
    endOffset,
  }: {
    readonly nodes: readonly DocumentNode[];
    readonly startOffset: number;
    readonly endOffset: number;
  },
): readonly DocumentNode[] {
  return nodes.filter(function isTouching(node,): boolean {
    return (node.startOffset < endOffset)
      && (node.endOffset > startOffset);
  },);
}

/**
 Refuses an anchor sitting strictly inside a block.

 An insertion names a place between two blocks, and every legal such place is
 a block boundary. An offset in the middle of one is a place assembly will
 happily write to, splitting a paragraph around text nobody asked to have
 split, and the layout check cannot see it: an empty span starts where it ends
 and so never overlaps a neighbour.

 @param anchor - insertion chunk to place

 @param nodes - every block-level node of the translation, in order

 @throws {@link SpanContiguityError} when the offset is inside a block

 @example
 ```ts
 assertAnchorBetweenBlocks({ anchor, nodes, },);
 ```
 */
function assertAnchorBetweenBlocks(
  {
    anchor,
    nodes,
  }: {
    readonly anchor: ChunkPair['target'];
    readonly nodes: readonly DocumentNode[];
  },
): void {
  for (const node of nodes) {
    if ((node.startOffset < anchor.startOffset)
      && (anchor.startOffset < node.endOffset)) {
      throw new SpanContiguityError({
        message: `slice ${String(anchor.sliceIndex,)} anchors at ${
          String(anchor.startOffset,)
        }, inside block ${node.id}, so assembly would split that block around the inserted text`,
      },);
    }
  }
}

/**
 Checks that every content span carries exactly the blocks it covers.

 CALLED AT PREPARATION, where the document's whole node sequence is in hand.
 Assembly cannot make this check: it is handed slices and a text, and the text
 agrees with the offsets whether or not a block went missing from the run.

 @param slices - prepared slice pairs

 @param targetNodes - every block-level node of the translation, in order

 @throws {@link SpanContiguityError} when a span's range holds a block its
 nodes do not, or its nodes reach outside its range

 @example
 ```ts
 assertSpanContiguity({ slices, targetNodes: targetDocument.nodes, },);
 ```
 */
export function assertSpanContiguity(
  {
    slices,
    targetNodes,
  }: {
    readonly slices: readonly ChunkPair[];
    readonly targetNodes: readonly DocumentNode[];
  },
): void {
  for (const slice of slices) {
    /**
     Target side of this pair, which is the side assembly writes over.
     */
    const span = slice.target;
    if (isInsertionChunk(span,)) {
      assertAnchorBetweenBlocks({
        anchor: span,
        nodes: targetNodes,
      },);
      continue;
    }

    /**
     Blocks the document shares any character of this span's range with.
     */
    const covered = nodesTouching({
      nodes: targetNodes,
      startOffset: span.startOffset,
      endOffset: span.endOffset,
    },);
    for (const node of covered) {
      if ((node.startOffset < span.startOffset)
        || (node.endOffset > span.endOffset)) {
        throw new SpanContiguityError({
          message: `slice ${String(span.sliceIndex,)} cuts through block ${node.id}, covering part of `
            + 'it: assembly replaces exactly the range, so the rest of that block would be left beside '
            + 'a replacement written without it',
        },);
      }
    }

    /**
     Blocks this slice says it carries.
     */
    const carriedCount = span.nodes
      .length;
    if (covered.length !== carriedCount) {
      throw new SpanContiguityError({
        message: `slice ${String(span.sliceIndex,)} spans ${
          String(covered.length,)
        } ${
          wordForCount({
            count: covered.length,
            one: 'block',
            many: 'blocks',
          },)
        } of the translation and carries ${String(carriedCount,)}: a block inside the range `
          + 'that the slice never saw is replaced anyway, because assembly writes over the range '
          + 'rather than over the nodes',
      },);
    }

    /**
     Whether every block the range holds is one this slice carries.

     Checked by identity rather than by count alone, since a slice carrying a
     block from OUTSIDE its range plus one fewer from inside would count
     correctly and describe two different passages.
     */
    const carried = new Set(span.nodes
      .map(function toId(node,): string {
        return node.id;
      },),);
    for (const node of covered) {
      if (!carried.has(node.id,)) {
        throw new SpanContiguityError({
          message: `slice ${String(span.sliceIndex,)} covers block ${
            node.id
          } and does not carry it, so its nodes and its range describe different passages`,
        },);
      }
    }
  }
}

//endregion Span contiguity
