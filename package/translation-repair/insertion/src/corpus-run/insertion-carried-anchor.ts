import type { ChunkPair, } from '@monochromatic-dev/translation-repair-slice-reading/ts';
import {
  type AnchorTarget,
  type ContentChunk,
  isInsertionChunk,
} from '@monochromatic-dev/translation-repair-shared-contract/ts';
import { codePointCount, } from '@monochromatic-dev/translation-repair-markup/ts';
import { locateQuote, } from '@monochromatic-dev/translation-repair-typography/ts';

//region Carried evidence anchoring
// WHERE A CARRIED PASSAGE'S EVIDENCE SITS. The coverage voices quote the
// page text that renders a source-only passage; the fold needs to know which
// paired slices' archive spans hold that text, block by block. A quote that
// crosses a paragraph boundary anchors once per block it touches, and each
// block may belong to a different paired slice: on mikaela13 (2026-09-24)
// the one ballot quoted the HRT sentences out of slice 10's span and the
// "originally just a typical thing" line out of slice 12's, because the
// archive rendered the carried passage across both neighbours (class one
// hundred eleven). The anchoring reports every block's holder with its
// share, so the fold can choose the neighbour that holds most of it.

/**
 One anchored block's share of a quoted region and the paired slice holding it.

 @example
 ```ts
 const holder: AnchorHolder = { position: 2, codePoints: 40, nodeId: 'block/3', };
 ```
 */
export type AnchorHolder = {
  /**
   Prepared position of the paired slice whose target span holds the block.
   */
  readonly position: number;

  /**
   Code points of the region inside that block.
   */
  readonly codePoints: number;

  /**
   Archive block the share sits in, so the shift (class one hundred
   seventy-nine) can tell whether the evidence touches every block of a span.
   */
  readonly nodeId: string;
};

/**
 Where a quoted region's blocks sit among the paired slices, or that it
 anchors nowhere usable.

 @example
 ```ts
 const anchoring: RegionAnchoring = { anchored: false, reason: 'quote-not-found (target)', };
 ```
 */
export type RegionAnchoring =
  | {
    readonly anchored: true;

    /**
     One holder per block the region touches, in page order.
     */
    readonly holders: readonly AnchorHolder[];
  }
    | {
      readonly anchored: false;

    /**
     Why the region cannot be placed: the locator's reason, or a block no
     paired slice holds.
     */
      readonly reason: string;
    };

/**
 Positions of paired slices whose target span holds the whole block.

 @param slices - prepared slices

 @param startOffset - block start in the target

 @param endOffset - block end in the target

 @returns Every holding position, in prepared order (at most one, since paired
 spans do not overlap)

 @example
 ```ts
 const holders = holdingPositions({ slices, startOffset: 10, endOffset: 40, },);
 ```
 */
function holdingPositions(
  {
    slices,
    startOffset,
    endOffset,
  }: {
    readonly slices: readonly ChunkPair[];
    readonly startOffset: number;
    readonly endOffset: number;
  },
): readonly number[] {
  return slices.flatMap(function holding(
    slice,
    position,
  ): readonly number[] {
    /**
     Placement on the target side.
     */
    const { target, } = slice;
    if (isInsertionChunk(target,))
      return [];
    return ((target.startOffset <= startOffset) && (endOffset <= target.endOffset))
      ? [position,]
      : [];
  },);
}

/**
 Places one quoted region block by block among the paired slices.

 @param slices - prepared slices

 @param target - archive parsed for anchoring

 @param region - page text a voice quoted

 @returns Holders with their shares, or why the region cannot be placed

 @example
 ```ts
 const anchoring = anchorRegion({ slices, target, region: 'The cat slept.', },);
 ```
 */
export function anchorRegion(
  {
    slices,
    target,
    region,
  }: {
    readonly slices: readonly ChunkPair[];
    readonly target: AnchorTarget;
    readonly region: string;
  },
): RegionAnchoring {
  // THE REGION IS STORED TEXT, a verdict's recorded evidence, not a quote copied from a sheet that cut the notes
  // between translators; it is searched in the page as it stands, so it takes no original
  // (`typography/src/locate-quote.ts`), and a region holding a note finds it where it stands.
  /**
   Where the region sits on the page.
   */
  const location = locateQuote({
    document: target,
    side: 'target',
    quote: region,
  },);
  if (!location.located) {
    return {
      anchored: false,
      reason: location.reason,
    };
  }
  /**
   Each block's holders, empty where no paired span holds the block.
   */
  const perBlock = location.anchors
    .map(function holdersOf(anchor,): readonly AnchorHolder[] {
      /**
       The region's share of this block.
       */
      const blockText = target.text
        .slice(
        anchor.startOffset,
        anchor.endOffset,
        );
      /**
       Code points of that share.
       */
      const codePoints = codePointCount({ text: blockText, },);
      return holdingPositions({
        slices,
        startOffset: anchor.startOffset,
        endOffset: anchor.endOffset,
      },)
        .map(function toHolder(position,): AnchorHolder {
          return {
            position,
            codePoints,
            nodeId: anchor.nodeId,
          };
        },);
    },);
  /**
   Whether some block sits in no paired span.
   */
  const unheld = perBlock.some(function isUnheld(holders,): boolean {
    return holders.length === 0;
  },);
  if (unheld) {
    return {
      anchored: false,
      reason: 'a block of the region sits in no paired slice',
    };
  }
  return {
    anchored: true,
    holders: perBlock.flat(),
  };
}

/**
 Whether the original writes nothing but blank space between two spans,
 whichever comes first.

 IN EITHER ORDER, so no caller picks which span it names first by the
 slices' positions. The gap runs from the earlier span's end to the later
 one's start. Spans that overlap have no gap and count as abutting:
 prepared sources never overlap, and widening either over the other gives
 their union, which is what a fold makes of two abutting spans.

 @param sourceText - whole original

 @param one - either source chunk

 @param other - the other one

 @returns True where the spans abut across blank space alone

 @example
 ```ts
 const touching = abutting({ sourceText, one: a.source, other: b.source, },);
 ```
 */
export function abutting(
  {
    sourceText,
    one,
    other,
  }: {
    readonly sourceText: string;
    readonly one: ContentChunk;
    readonly other: ContentChunk;
  },
): boolean {
  /**
   What the original writes between the two spans, empty where they
   overlap.
   */
  const between = sourceText.slice(
    Math.min(
      one.endOffset,
      other.endOffset,
    ),
    Math.max(
      one.startOffset,
      other.startOffset,
    ),
  );
  return between.trim() === '';
}

//endregion Carried evidence anchoring
