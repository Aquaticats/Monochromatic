import type { ChunkPair, } from '@monochromatic-dev/translation-repair-slice-reading/ts';
import { isInsertionChunk, } from '@monochromatic-dev/translation-repair-shared-contract/ts';
import { wordForCount, } from '@monochromatic-dev/translation-repair-typography/ts';
import { UnhandledMemberInvariantError, } from '@monochromatic-dev/translation-repair-document-structure/ts';

//region Placement layout
// Whether a preparation's target spans can be written back into the document
// they were cut from.
//
// SEPARATE FROM INDEXING, which `slice-indexing.ts` covers. That file proves a
// slice is CALLED what its position says; this one proves the slice POINTS
// somewhere sane. Both were the same statement while every span came from a
// disjoint run of nodes, and insertion separates them: an insertion anchor covers
// no nodes at all, so nothing about its offsets follows from how it was carved.
//
// ONE ORDERING RULE COVERS THE WHOLE LIST the review asked for. Every span
// starts at or after the previous span's END, and from that single statement
// follow: non-empty spans never overlap; an insertion never lands strictly
// inside a span; two non-empty spans never start at one offset; an insertion
// never sits after a span it shares a boundary with; and no placement moves
// backwards. What it still ALLOWS is every legal shape: several insertions at
// one boundary, an insertion at a span's start meaning before it, an insertion
// at a span's end, and adjacent spans that touch.
//
// WHY IT MATTERS MORE THAN IT LOOKS: `String.prototype.slice` clamps. A span
// running past the end of the document, or backwards, produces plausible text
// rather than a diagnostic, and assembly then ships a document nobody can tell
// is wrong from the output alone.

/**
 What is wrong with one slice's span, in offset and count terms only.

 @example
 ```ts
 const fault: PlacementFault = { kind: 'hollow-content', startOffset: 12, };
 ```
 */
export type PlacementFault = {
  /**
   Offsets that are not both whole numbers.
   */
  readonly kind: 'fractional-offsets';

  /**
   Start as carried.
   */
  readonly startOffset: number;

  /**
   End as carried.
   */
  readonly endOffset: number;
} | {
  /**
   Span reaching past the document, which slicing would clamp rather than
   refuse.
   */
  readonly kind: 'past-document';

  /**
   Start as carried.
   */
  readonly startOffset: number;

  /**
   End as carried.
   */
  readonly endOffset: number;

  /**
   Characters the document holds.
   */
  readonly documentLength: number;
} | {
  /**
   Span ending before it starts, which slicing would read as empty.
   */
  readonly kind: 'backwards';

  /**
   Start as carried.
   */
  readonly startOffset: number;

  /**
   End as carried.
   */
  readonly endOffset: number;
} | {
  /**
   Insertion that covers characters, wording or nodes; a place covers none.
   */
  readonly kind: 'fat-anchor';

  /**
   Characters the span covers.
   */
  readonly covered: number;

  /**
   Characters of wording it carries.
   */
  readonly wordingLength: number;

  /**
   Nodes it carries.
   */
  readonly nodeCount: number;
} | {
  /**
   Wording the document does not hold between these offsets: stale, cut
   from another document, or misplaced.
   */
  readonly kind: 'foreign-text';

  /**
   Start as carried.
   */
  readonly startOffset: number;

  /**
   End as carried.
   */
  readonly endOffset: number;
} | {
  /**
   Content covering nothing, which is an insertion that does not say so.
   */
  readonly kind: 'hollow-content';

  /**
   Where it sits.
   */
  readonly startOffset: number;
} | {
  /**
   Span starting before the slice before it ends.
   */
  readonly kind: 'overlaps-previous';

  /**
   Start as carried.
   */
  readonly startOffset: number;

  /**
   Where the slice before it runs to.
   */
  readonly boundary: number;
};

/**
 Sentence for one fault, from its numbers alone.

 @param fault - what is wrong

 @returns Sentence completing "slice at position N ..."

 @throws {@link UnhandledMemberInvariantError} when a member of `PlacementFault` has no branch here, which the compiler rules out

 @example
 ```ts
 const said = placementSentence({ fault, },);
 ```
 */
function placementSentence({ fault, }: { readonly fault: PlacementFault; },): string {
  if (fault.kind === 'fractional-offsets') {
    return `carries offsets ${String(fault.startOffset,)} and ${String(fault.endOffset,)}, which are not both `
      + 'whole numbers';
  }
  if (fault.kind === 'past-document') {
    return `spans ${String(fault.startOffset,)} to ${String(fault.endOffset,)} in a document of ${
      String(fault.documentLength,)
    }, and slicing a string CLAMPS rather than fails`;
  }
  if (fault.kind === 'backwards') {
    return `ends at ${String(fault.endOffset,)} before it starts at ${String(fault.startOffset,)}, which slicing `
      + 'reads as an empty span rather than as the mistake it is';
  }
  if (fault.kind === 'fat-anchor') {
    return `says it is an insertion while covering ${String(fault.covered,)} ${
      wordForCount({
        count: fault.covered,
        one: 'character',
        many: 'characters',
      },)
    }, ${
      String(fault.wordingLength,)
    } of wording and ${String(fault.nodeCount,)} ${
      wordForCount({
        count: fault.nodeCount,
        one: 'node',
        many: 'nodes',
      },)
    }; a place covers none of the three`;
  }
  if (fault.kind === 'foreign-text') {
    return `carries text the document does not hold between ${String(fault.startOffset,)} and ${
      String(fault.endOffset,)
    }: these offsets, this wording and this document do not describe one passage, whether because the slices `
      + 'are stale, cut from another document, or misplaced';
  }
  if (fault.kind === 'hollow-content') {
    return `is content covering nothing at ${String(fault.startOffset,)}; an empty span is an insertion, and `
      + 'saying so is what tells assembly to write INTO it';
  }
  if (fault.kind === 'overlaps-previous') {
    return `starts at ${String(fault.startOffset,)} while the slice before it runs to ${String(fault.boundary,)}, `
      + 'so writing one would move or overwrite the other';
  }
  throw new UnhandledMemberInvariantError({
    what: 'placement fault',
    member: fault,
  },);
}

/**
 Thrown when target spans cannot be written back into their document.

 @example
 ```ts
 throw new PlacementLayoutError({ position: 3, fault: { kind: 'overlaps-previous', startOffset: 4, boundary: 9, }, },);
 ```
 */
export class PlacementLayoutError extends Error {
  /**
   Declares this message safe to print whole at a boundary: it is written
   here from a position, offsets and counts, and quotes nothing.
   */
  readonly messageNamesOnly: true = true;

  /**
   What could not be placed, for a caller that reads the fault rather than
   the sentence.
   */
  readonly fault: PlacementFault;

  /**
   Builds failure naming what cannot be placed.

   @param position - where the slice sits in the list

   @param fault - what is wrong, in offset and count terms

   @example
   ```ts
   throw new PlacementLayoutError({ position: 3, fault: { kind: 'past-document', ... }, },);
   ```
   */
  public constructor(
    {
      position,
      fault,
    }: {
      readonly position: number;
      readonly fault: PlacementFault;
    },
  ) {
    super(`slice at position ${String(position,)} ${placementSentence({ fault, },)}`,);
    this.name = 'PlacementLayoutError';
    this.fault = fault;
  }
}

/**
 Refuses one slice whose own span is malformed, whatever its neighbours do.

 @param slice - prepared pair whose target side is checked

 @param position - where it sits, for the message

 @param targetText - document the span indexes into

 @throws {@link PlacementLayoutError} when offsets are not whole numbers in
 order and in range, when a span's text is not what the document holds there,
 or when an insertion covers anything at all

 @example
 ```ts
 assertSpanShape({ slice, position: 0, targetText, },);
 ```
 */
function assertSpanShape(
  {
    slice,
    position,
    targetText,
  }: {
    readonly slice: ChunkPair;
    readonly position: number;
    readonly targetText: string;
  },
): void {
  /**
   Target side, which is the only side assembly writes into.
   */
  const span = slice.target;

  /**
   Where it starts.
   */
  const { startOffset, } = span;

  /**
   Where it ends, exclusive.
   */
  const { endOffset, } = span;
  if ((!Number.isInteger(startOffset,)) || (!Number.isInteger(endOffset,))) {
    throw new PlacementLayoutError({
      position,
      fault: {
        kind: 'fractional-offsets',
        startOffset,
        endOffset,
      },
    },);
  }
  /**
   How much document there is to point into.
   */
  const documentLength = targetText.length;
  if ((startOffset < 0)
    || (endOffset > documentLength)) {
    throw new PlacementLayoutError({
      position,
      fault: {
        kind: 'past-document',
        startOffset,
        endOffset,
        documentLength,
      },
    },);
  }
  if (startOffset > endOffset) {
    throw new PlacementLayoutError({
      position,
      fault: {
        kind: 'backwards',
        startOffset,
        endOffset,
      },
    },);
  }
  if (isInsertionChunk(span,)) {
    /**
     How many nodes it claims, which for a place is none.
     */
    const nodeCount = span.nodes
      .length;

    /**
     How much wording it claims, which for a place is none.
     */
    const wordingLength = span.text
      .length;

    /**
     Whether it covers a range or wording, either of which contradicts it.
     */
    const covers = (startOffset !== endOffset)
      || (wordingLength > 0);
    if (covers
      || (nodeCount > 0)) {
      throw new PlacementLayoutError({
        position,
        fault: {
          kind: 'fat-anchor',
          covered: endOffset - startOffset,
          wordingLength,
          nodeCount,
        },
      },);
    }
    return;
  }
  if (span.text !== targetText.slice(
    startOffset,
    endOffset,
  )) {
    throw new PlacementLayoutError({
      position,
      fault: {
        kind: 'foreign-text',
        startOffset,
        endOffset,
      },
    },);
  }
  if (startOffset === endOffset) {
    throw new PlacementLayoutError({
      position,
      fault: {
        kind: 'hollow-content',
        startOffset,
      },
    },);
  }
}

/**
 Refuses a preparation whose target spans cannot all be written back.

 WALKS IN SLICE ORDER rather than in offset order, because slice order is what
 a caller relies on: it decides which of two insertions at one boundary comes
 first, and it is the order every lane result and every ledger is read in. A
 preparation whose spans are sorted differently from its slices is exactly the
 defect this exists to name.

 @param slices - prepared slice pairs in document order

 @param targetText - translation those spans index into

 @throws {@link PlacementLayoutError} when a span is malformed, or when one
 does not start at or after the previous span's end

 @example
 ```ts
 assertPlacementLayout({ slices, targetText, },);
 ```
 */
export function assertPlacementLayout(
  {
    slices,
    targetText,
  }: {
    readonly slices: readonly ChunkPair[];
    readonly targetText: string;
  },
): void {
  for (const [position, slice,] of slices.entries()) {
    assertSpanShape({
      slice,
      position,
      targetText,
    },);

    /**
     Slice before this one, absent for the first.
     */
    const previous = (position === 0) ? undefined : slices[position - 1];
    if (previous === undefined)
      continue;

    /**
     Where the previous span ends, which this one may start at.
     */
    const boundary = previous.target
      .endOffset;

    /**
     Where this span starts.
     */
    const { startOffset, } = slice.target;
    if (startOffset < boundary) {
      throw new PlacementLayoutError({
        position,
        fault: {
          kind: 'overlaps-previous',
          startOffset,
          boundary,
        },
      },);
    }
  }
}

//endregion Placement layout
