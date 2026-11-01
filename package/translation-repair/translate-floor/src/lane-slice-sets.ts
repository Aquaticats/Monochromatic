import type { ChunkPair, } from '@monochromatic-dev/translation-repair-slice-reading/ts';
import { isInsertionChunk, } from '@monochromatic-dev/translation-repair-shared-contract/ts';
import {
  LaneSliceCoverageError,
  type NamedSliceSetLabel,
} from './lane-slice-coverage-error.ts';

//region Lane slice sets
// The slices a lane NAMES as something other than decided, and the five checks
// each of those lists has to pass.
//
// Three such lists exist, and they arrived one at a time: unfilled, then
// unheard, then not-applicable. Each was written as its own loop against the
// preparation, and by the third the loops were the same five checks with the
// wording changed, which is how the pairwise disjointness between them came to
// be checked in one direction only.
//
// So the checks live here once and the lists differ by DATA: what each is
// called, what it says when a slice is named twice, and which side of the
// archive it belongs to. That last one is the check worth having: every list
// here is only legal at one kind of slice, and getting it wrong is how an
// exemption list becomes a way around the coverage rule.

/**
 @internal

 One list of slices a lane names, beside what makes the list legal.

 @example
 ```ts
 const set: NamedSliceSet = {
   label: 'unfilled',
   indices: [3,],
   incumbent: 'absent',
 };
 ```
 */
export type NamedSliceSet = {
  /**
   What the lane calls these slices, which every message repeats.
   */
  readonly label: NamedSliceSetLabel;

  /**
   Slices named, by global index.
   */
  readonly indices: readonly number[];

  /**
   Which side of the archive this list is legal at: `absent` for lists about
   passages the archive never translated, `present` for lists about wording it
   holds.
   */
  readonly incumbent: 'absent' | 'present';

};

/**
 One list beside the indices it names, read once and carried together.
 */
type NamedIndices = {
  /**
   List the lane named.
   */
  readonly set: NamedSliceSet;

  /**
   Its indices, repeats already refused.
   */
  readonly indices: ReadonlySet<number>;
};

/**
 Refuses a list that repeats a slice, and turns it into a set.

 @param set - list being checked, for its label and indices

 @returns Distinct indices it names

 @throws {@link LaneSliceCoverageError} when an index appears twice, since the
 set would still be the right shape and one slice would be named once

 @example
 ```ts
 const unfilled = distinctIndices({ set, },);
 ```
 */
function distinctIndices(
  { set, }: { readonly set: NamedSliceSet; },
): ReadonlySet<number> {
  /**
   Indices this list names, with any repeat collapsed.
   */
  const distinct = new Set(set.indices,);

  /**
   How many the list claims, before the repeats were collapsed.
   */
  const claimed = set.indices
    .length;
  if (distinct.size !== claimed) {
    throw new LaneSliceCoverageError({
      fault: {
        kind: 'set-repeats',
        set: set.label,
        claimed,
        distinct: distinct.size,
      },
    },);
  }
  return distinct;
}

/**
 Refuses a named slice the preparation never produced, one already decided, or
 one whose archive state contradicts what the list means.

 @param set - list being checked

 @param indices - its indices, already proven distinct

 @param slices - prepared pairs, which answer both membership and archive
 state

 @param decidedIndices - slices the lane also reported a wording for

 @throws {@link LaneSliceCoverageError} on any of the three

 @example
 ```ts
 assertNamesLegalSlices({ set, indices, slices, decidedIndices, },);
 ```
 */
function assertNamesLegalSlices(
  {
    set,
    indices,
    slices,
    decidedIndices,
  }: {
    readonly set: NamedSliceSet;
    readonly indices: ReadonlySet<number>;
    readonly slices: readonly ChunkPair[];
    readonly decidedIndices: ReadonlySet<number>;
  },
): void {
  for (const sliceIndex of indices) {
    /**
     Pair this index names, absent when the two were built from different
     preparations.
     */
    const named = slices.find(function isNamed(slice,): boolean {
      return slice.target
        .sliceIndex
        === sliceIndex;
    },);
    if (named === undefined) {
      throw new LaneSliceCoverageError({
        fault: {
          kind: 'set-names-unproduced',
          set: set.label,
          sliceIndex,
        },
      },);
    }
    if (decidedIndices.has(sliceIndex,)) {
      throw new LaneSliceCoverageError({
        fault: {
          kind: 'set-and-decided',
          set: set.label,
          sliceIndex,
        },
      },);
    }
  }
}

/**
 Refuses a named slice sitting on the wrong side of the archive.

 Checked LAST of the per-list rules, so a slice named by two lists reports the
 contradiction between them rather than whichever archive rule the first list
 happens to break.

 @param set - list being checked

 @param indices - its indices

 @param slices - prepared pairs, which are the only thing that knows

 @throws {@link LaneSliceCoverageError} when a list about missing passages
 names one the archive translates, or the other way around

 @example
 ```ts
 assertArchiveAllows({ set, indices, slices, },);
 ```
 */
function assertArchiveAllows(
  {
    set,
    indices,
    slices,
  }: {
    readonly set: NamedSliceSet;
    readonly indices: ReadonlySet<number>;
    readonly slices: readonly ChunkPair[];
  },
): void {
  for (const sliceIndex of indices) {
    /**
     Pair this index names, which {@link assertNamesLegalSlices} proved is
     there.
     */
    const named = slices.find(function isNamed(slice,): boolean {
      return slice.target
        .sliceIndex
        === sliceIndex;
    },);

    /**
     Whether the archive holds nothing at this slice.
     */
    const absent = (named !== undefined) && isInsertionChunk(named.target,);
    if ((named !== undefined) && (absent !== (set.incumbent === 'absent'))) {
      throw new LaneSliceCoverageError({
        fault: {
          kind: 'set-against-archive',
          set: set.label,
          sliceIndex,
        },
      },);
    }
  }
}

/**
 @internal

 Validates every list a lane names, and refuses any slice on two of them.

 @param sets - lists to validate, in the order their messages should be tried

 @param slices - prepared pairs

 @param decidedIndices - slices the lane reported a wording for

 @returns One index set per list, in the order given

 @throws {@link LaneSliceCoverageError} when a list repeats a slice, names one
 the preparation never produced, names one already decided, names one another
 list also names, or names one whose archive state the list forbids

 @example
 ```ts
 const [unfilled, unheard,] = validateNamedSets({ sets, slices, decidedIndices, },);
 ```
 */
export function validateNamedSets(
  {
    sets,
    slices,
    decidedIndices,
  }: {
    readonly sets: readonly NamedSliceSet[];
    readonly slices: readonly ChunkPair[];
    readonly decidedIndices: ReadonlySet<number>;
  },
): readonly ReadonlySet<number>[] {
  /**
   Each list beside its indices, repeats within one list refused as each is
   read. Paired once, so no later step looks a list's indices up by position.
   */
  const named = sets.map(function withIndices(set,): NamedIndices {
    return {
      set,
      indices: distinctIndices({ set, },),
    };
  },);
  for (const {
    set,
    indices,
  } of named) {
    assertNamesLegalSlices({
      set,
      indices,
      slices,
      decidedIndices,
    },);
  }

  // BEFORE THE ARCHIVE RULES, because a slice named by two lists disagrees with
  // itself first: reporting which archive rule it breaks would answer a
  // question neither list has earned the right to ask.
  for (const [position, {
    set,
    indices,
  },] of named.entries()) {
    for (const {
      set: other,
      indices: otherIndices,
    } of named.slice(position + 1,)) {
      /**
       Slices both name, which is a contradiction whichever two lists they are.
       */
      const both = [...indices,].filter(function inOther(sliceIndex,): boolean {
        return otherIndices.has(sliceIndex,);
      },);
      for (const sliceIndex of both) {
        throw new LaneSliceCoverageError({
          fault: {
            kind: 'two-sets',
            set: set.label,
            other: other.label,
            sliceIndex,
          },
        },);
      }
    }
  }
  for (const {
    set,
    indices,
  } of named) {
    assertArchiveAllows({
      set,
      indices,
      slices,
    },);
  }
  return named.map(function indicesOf({ indices, },): ReadonlySet<number> {
    return indices;
  },);
}

//endregion Lane slice sets
