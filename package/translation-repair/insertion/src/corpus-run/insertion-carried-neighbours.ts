import type { ChunkPair, } from '@monochromatic-dev/translation-repair-slice-reading/ts';
import {
  type ContentChunk,
  isInsertionChunk,
} from '@monochromatic-dev/translation-repair-shared-contract/ts';

//region Carried passage neighbours
// WHICH PAIRED SLICES STAND NEXT TO A POSITION. The fold (class one hundred
// ten) looks for the carrier among the paired slices on either side of a
// carried passage; the shift (class one hundred seventy-nine) looks for the
// paired slice on the carrier's far side. Both look past insertions, so a
// carried passage beside another carried passage (mikaela14) still finds the
// paired slice beyond it; the abutting check refuses a fold or a shift across
// a source nobody has absorbed.
//
// A NEIGHBOUR CARRIES ITS SLICE, narrowed to a paired one where it is found.
// The fold and the shift read the carrier's archive span and source from it,
// so neither looks a position up again and guards a lookup that cannot miss.

/**
 A prepared slice the archive renders, so its target is a span of text.

 @example
 ```ts
 const carrier: PairedSlice = { source, target, };
 ```
 */
export type PairedSlice = ChunkPair & {
  /**
   Archive span rendering this slice.
   */
  readonly target: ContentChunk;
};

/**
 A paired slice next to a carried passage, with where it stands.

 @example
 ```ts
 const neighbour: PairedNeighbour = { position: 2, slice: carrier, };
 ```
 */
export type PairedNeighbour = {
  /**
   Position in prepared slice order.
   */
  readonly position: number;

  /**
   The slice there.
   */
  readonly slice: PairedSlice;
};

/**
 Whether the archive renders a slice.

 TAKES ITS PARAMETER POSITIONALLY: a type predicate narrows a named
 parameter, and a destructured object has none.

 @param slice - prepared slice

 @returns True where its target is a span of text

 @example
 ```ts
 const paired = isPairedSlice(slice,);
 ```
 */
function isPairedSlice(slice: ChunkPair,): slice is PairedSlice {
  return !isInsertionChunk(slice.target,);
}

/**
 Nearest paired slice on one side of a position, looking past insertions.

 @param slices - prepared slices

 @param position - where the looking starts

 @param step - direction: -1 for earlier, 1 for later

 @returns That slice with its position, none where only insertions lie that way

 @example
 ```ts
 const earlier = pairedNeighbourToward({ slices, position: 3, step: -1, },);
 ```
 */
export function pairedNeighbourToward(
  {
    slices,
    position,
    step,
  }: {
    readonly slices: readonly ChunkPair[];
    readonly position: number;
    readonly step: number;
  },
): readonly PairedNeighbour[] {
  /**
   Paired slices on that side, nearest first.
   */
  const thatWay = slices
    .flatMap(function pairedOnSide(
      slice,
      index,
    ): readonly PairedNeighbour[] {
      /**
       Whether this position lies on the side looked at.
       */
      const onSide = (step < 0) ? (index < position) : (index > position);
      return (onSide && isPairedSlice(slice,))
        ? [{
          position: index,
          slice,
        },]
        : [];
    },)
    .toSorted(function nearestFirst(
      a,
      b,
    ): number {
      return Math.abs(a.position - position,) - Math.abs(b.position - position,);
    },);
  return thatWay.slice(
    0,
    1,
  );
}

/**
 Nearest paired slices on either side of a carried passage, earlier first.
 A carried passage next to another carried passage looks past it: on
 mikaela14 (2026-09-24) slice 12 folded into slice 13 and slice 11, whose
 evidence sat in slice 13's span too, was refused for having a folded
 insertion between; the abutting check still refuses a fold across a source
 the carrier has not absorbed.

 @param slices - prepared slices

 @param position - where the carried slice stands

 @returns The paired neighbours found, earlier first

 @example
 ```ts
 const neighbours = pairedNeighbours({ slices, position: 2, },);
 ```
 */
export function pairedNeighbours(
  {
    slices,
    position,
  }: {
    readonly slices: readonly ChunkPair[];
    readonly position: number;
  },
): readonly PairedNeighbour[] {
  return [
    ...pairedNeighbourToward({
      slices,
      position,
      step: -1,
    },),
    ...pairedNeighbourToward({
      slices,
      position,
      step: 1,
    },),
  ];
}

//endregion Carried passage neighbours
