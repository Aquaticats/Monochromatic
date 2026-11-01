import type { ChunkPair, } from '@monochromatic-dev/translation-repair-slice-reading/ts';
import type {
  AnchorTarget,
  ContentChunk,
} from '@monochromatic-dev/translation-repair-shared-contract/ts';
import {
  abutting,
  type AnchorHolder,
} from './insertion-carried-anchor.ts';
import {
  type PairedNeighbour,
  pairedNeighbourToward,
} from './insertion-carried-neighbours.ts';

//region Carried passage shift
// THE PAIRING LEFT ONE OFF. On TianqiChen66611 (2026-09-26) the archive
// rendered two quoted lines in one quote and the paragraph after them on its
// own; the pairing gave the first line the whole quote, the second line the
// paragraph, and left the paragraph's own source carried. The class one
// hundred ten fold widened the carrier (the second line) over the carried
// source, so the carrier rendered the second line again beside the paragraph
// and the page carried that line twice (class one hundred seventy-nine).
//
// Where the carried passage's evidence sits in the carrier alone and touches
// every block of the carrier's archive span, that span renders the passage
// and nothing of the carrier's own source. So the carrier takes the carried
// source in place of its own, and its own source joins the paired slice on
// its far side, provided the two sources abut across blank space alone. Where
// any of that fails the plain fold stands: the carrier's own source is never
// dropped, only moved to the neighbour it abuts.

/**
 Finding prefix for a carrier whose own source moved to its far neighbour.
 */
export const CARRIED_SHIFTED_FINDING = 'insertion-carried-shifted';

/**
 The paired slice on the carrier's far side that takes the carrier's own
 source, where the carrier's archive span renders the carried passage alone.

 @param slices - prepared slices as they stand

 @param sourceText - whole original

 @param target - archive parsed for anchoring

 @param holders - every evidence block's holder

 @param carrier - the paired slice the passage folds into

 @param carriedPosition - the carried passage

 @returns The receiving slice, none where the plain fold stands

 @example
 ```ts
 const [receiver,] = shiftReceiver({ slices, sourceText, target, holders, carrier, carriedPosition: 3, },);
 ```
 */
export function shiftReceiver(
  {
    slices,
    sourceText,
    target,
    holders,
    carrier,
    carriedPosition,
  }: {
    readonly slices: readonly ChunkPair[];
    readonly sourceText: string;
    readonly target: AnchorTarget;
    readonly holders: readonly AnchorHolder[];
    readonly carrier: PairedNeighbour;
    readonly carriedPosition: number;
  },
): readonly PairedNeighbour[] {
  /**
   Whether every evidence block sits in the carrier's span.
   */
  const carrierAlone = holders.every(function inCarrier(holder,): boolean {
    return holder.position === carrier.position;
  },);
  if (!carrierAlone)
    return [];
  /**
   The carrier's archive span.
   */
  const span = carrier.slice
    .target;
  /**
   Archive blocks inside that span.
   */
  const spanBlocks = target.nodes
    .filter(function inSpan(node,): boolean {
      return (span.startOffset <= node.startOffset) && (node.endOffset <= span.endOffset);
    },);
  /**
   Blocks the evidence touches.
   */
  const touched = new Set(holders.map(function blockOf(holder,): string {
    return holder.nodeId;
  },),);
  /**
   Whether the span renders the carried passage and nothing else.
   */
  const passageAlone = (spanBlocks.length > 0)
    && spanBlocks.every(function isTouched(node,): boolean {
      return touched.has(node.id,);
    },);
  if (!passageAlone)
    return [];
  /**
   The paired slice on the far side of the carrier from the passage.
   */
  const [receiver,] = pairedNeighbourToward({
    slices,
    position: carrier.position,
    step: (carrier.position < carriedPosition) ? -1 : 1,
  },);
  if (receiver === undefined)
    return [];
  /**
   Whether the carrier's source and the receiver's abut across blank space.
   */
  const touching = abutting({
    sourceText,
    one: receiver.slice
      .source,
    other: carrier.slice
      .source,
  },);
  return touching ? [receiver,] : [];
}

/**
 One slice's source widened over another's, in document order.

 @param sourceText - whole original

 @param widened - slice whose source grows, keeping its own index

 @param absorbed - slice whose source joins it

 @returns The widened slice

 @example
 ```ts
 const carrier = widenSource({ sourceText, widened: carrierSlice, absorbed: carriedSlice, },);
 ```
 */
export function widenSource(
  {
    sourceText,
    widened,
    absorbed,
  }: {
    readonly sourceText: string;
    readonly widened: ChunkPair;
    readonly absorbed: ChunkPair;
  },
): ChunkPair {
  /**
   Both sources, earlier first.
   */
  const ordered = [
    widened.source,
    absorbed.source,
  ]
    .toSorted(function byStart(
      a,
      b,
    ): number {
      return a.startOffset - b.startOffset;
    },);
  /**
   Where the widened span starts.
   */
  const startOffset = Math.min(...ordered.map(function start(chunk,): number {
    return chunk.startOffset;
  },),);
  /**
   Where it ends.
   */
  const endOffset = Math.max(...ordered.map(function end(chunk,): number {
    return chunk.endOffset;
  },),);
  /**
   Stable index the widened source reports under.
   */
  const sourceIndex = widened.source
    .sliceIndex;
  /**
   The widened source.
   */
  const source: ContentChunk = {
    kind: 'content',
    sliceIndex: sourceIndex,
    nodes: ordered.flatMap(function nodesOf(chunk,): ContentChunk['nodes'] {
      return chunk.nodes;
    },),
    startOffset,
    endOffset,
    text: sourceText.slice(
      startOffset,
      endOffset,
    ),
  };
  return {
    ...widened,
    source,
  };
}

/**
 Slices after a shift: the receiver widened over the carrier's own source,
 the carrier holding the carried source under its own index.

 @param sourceText - whole original

 @param slices - prepared slices as they stand

 @param carrier - the paired slice the passage folds into

 @param receiver - the far neighbour taking the carrier's own source

 @param carried - the carried slice

 @returns Slices with both replaced

 @example
 ```ts
 const next = shiftSlices({ sourceText, slices, carrier, receiver, carried, },);
 ```
 */
export function shiftSlices(
  {
    sourceText,
    slices,
    carrier,
    receiver,
    carried,
  }: {
    readonly sourceText: string;
    readonly slices: readonly ChunkPair[];
    readonly carrier: PairedNeighbour;
    readonly receiver: PairedNeighbour;
    readonly carried: ChunkPair;
  },
): readonly ChunkPair[] {
  return slices.map(function replaceBoth(
    slice,
    position,
  ): ChunkPair {
    if (position === receiver.position) {
      return widenSource({
        sourceText,
        widened: slice,
        absorbed: carrier.slice,
      },);
    }
    if (position !== carrier.position)
      return slice;
    /**
     Stable index the carrier's source reports under.
     */
    const carrierSourceIndex = slice.source
      .sliceIndex;
    return {
      ...slice,
      source: {
        ...carried.source,
        sliceIndex: carrierSourceIndex,
      },
    };
  },);
}

/**
 Names a shift the way every other admission finding is named.

 @param carrierSliceIndex - the slice the passage folded into

 @param receiverSliceIndex - the far neighbour that took the carrier's source

 @param carriedSliceIndex - the carried passage

 @returns Finding in scorecard-stable wording

 @example
 ```ts
 const finding = shiftedFinding({ carrierSliceIndex: 2, receiverSliceIndex: 1, carriedSliceIndex: 3, },);
 ```
 */
export function shiftedFinding(
  {
    carrierSliceIndex,
    receiverSliceIndex,
    carriedSliceIndex,
  }: {
    readonly carrierSliceIndex: number;
    readonly receiverSliceIndex: number;
    readonly carriedSliceIndex: number;
  },
): string {
  /**
   The carrier's index as printed.
   */
  const carrier = String(carrierSliceIndex,);
  return `${CARRIED_SHIFTED_FINDING} (slice ${carrier}'s own source joins slice ${
    String(receiverSliceIndex,)
  }: slice ${carrier}'s archive span renders slice ${String(carriedSliceIndex,)}'s passage alone)`;
}

//endregion Carried passage shift
