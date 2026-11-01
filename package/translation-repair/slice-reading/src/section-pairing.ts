import type { BlockPair, } from '@monochromatic-dev/translation-repair-stage/ts';
import type { ReadonlyEntry, } from '@monochromatic-dev/translation-repair-shared-contract/ts';

//region Section pairing
// Which original block renders which translation block, filed under the aligned
// section the question was asked about.
//
// ITS OWN FILE because three unrelated places need the shape and none of them
// owns it: preparation consumes it, the roster shell produces it, and the
// settled artifact records it. Putting it in any one of those would make the
// other two import that one's whole subject.
//
// A LIST RATHER THAN THE MAP preparation is handed, because this shape is what
// gets written down. A `Map` serializes to `{}`, which would record every
// pairing ever agreed as no pairing at all.

/**
 One aligned section's agreed correspondences.

 @example
 ```ts
 const pairing: SectionBlockPairing = { sectionIndex: 0, pairs: [{ source: 0, target: 0, },], };
 ```
 */
export type SectionBlockPairing = {
  /**
   Aligned section this answers about, which is the key preparation reads it
   under and the index every `pairs` entry is local to.
   */
  readonly sectionIndex: number;

  /**
   Correspondences agreed for this section, in document order.

   EMPTY IS A REAL ANSWER here as everywhere else in this subject: the roster
   was asked about these blocks and committed to nothing. A section nobody was
   asked about is ABSENT from the list instead.
   */
  readonly pairs: readonly BlockPair[];
};

/**
 Orders a pairing map into the shape that gets written down.

 SORTED BY SECTION, so two runs that agreed the same pairings record the same
 bytes regardless of what order the sections were asked in. Insertion order
 happens to be section order today, which is exactly the kind of accident that
 stops being true and takes a stored record's comparability with it.

 @param blockPairings - pairing per aligned section, as preparation takes it

 @returns Same pairings, ordered by section

 @example
 ```ts
 const recorded = sectionPairingsOf({ blockPairings, },);
 ```
 */
export function sectionPairingsOf(
  { blockPairings, }: { readonly blockPairings: ReadonlyMap<number, readonly BlockPair[]>; },
): readonly SectionBlockPairing[] {
  return [...blockPairings.entries(),]
    .map(function toPairing([sectionIndex, pairs,]: ReadonlyEntry<number, readonly BlockPair[]>,): SectionBlockPairing {
      return {
        sectionIndex,
        pairs,
      };
    },)
    .toSorted(function bySection(
      left,
      right,
    ): number {
      return left.sectionIndex - right.sectionIndex;
    },);
}

//endregion Section pairing
