/**
 What one slice of the editor calibration produced and the slice itself, built
 from the few facts a case turns on, for the cases of the modules that print
 or drive them.

 @module
 */

import type { RosterModelId, } from '@monochromatic-dev/translation-repair-shared-contract';
import type {
  BenchSlice,
  SliceRounds,
} from '../../dist/final/node/index.mjs';

/**
 Builds a drawn slice from where it sits in the corpus, with invented text.

 @param entryId - entry the slice is cut from

 @param index - position within that entry's slices

 @returns Slice whose texts name a cat's nap

 @example
 ```ts
 const slice = benchSliceOf({ entryId: 'mittens', index: 0, },);
 ```
 */
export function benchSliceOf(
  {
    entryId,
    index,
  }: {
    readonly entryId: string;
    readonly index: number;
  },
): BenchSlice {
  return {
    entryId,
    index,
    sourceText: '小猫在窗台上打盹。\n',
    incumbentText: 'The kitten dozes on the windowsill.\n',
    lineStructured: false,
  };
}

/**
 Builds what a slice produced when no round was judged, naming only who
 shipped and whether the naturalness lane reached a rewriter.

 @param shippers - models credited with writing the shipped repair

 @param refineAsked - whether the lane found a paragraph to offer a rewriter

 @returns Rounds with both seats' judged rounds empty

 @example
 ```ts
 const rounds = unjudgedRounds({ shippers: [], refineAsked: true, },);
 ```
 */
export function unjudgedRounds(
  {
    shippers,
    refineAsked,
  }: {
    readonly shippers: readonly RosterModelId[];
    readonly refineAsked: boolean;
  },
): SliceRounds {
  return {
    editor: [],
    refiner: [],
    refineAsked,
    editorShipped: shippers,
    refinerShipped: [],
    refinerHeard: [],
  };
}
