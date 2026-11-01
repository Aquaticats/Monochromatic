import type { DocumentDisplacement, } from '@monochromatic-dev/translation-repair-artifact/ts';
import type { ReadonlyEntry, } from '@monochromatic-dev/translation-repair-shared-contract/ts';

//region Window trial draw
// Which slices the window trial buys, and under what label.
//
// TWO JOBS, and the second is the one that makes the first mean anything.
// Flagged slices come from the displacement screen. Matched UNFLAGGED slices
// come from the same documents, and without them a wide arm that keeps the
// archive more often everywhere would read as the window working on
// relocations. The judge-quality bench measured the roster declining on any
// archive imperfection, so a general context-induced conservatism is a live possibility rather than a
// theoretical one.
//
// RELOCATION CANDIDATES ARE ADJACENCIES, so one slice can appear in several. The
// unit of trial is a SLICE, and buying one twice would spend quota twice and
// count one model's answer twice in the tally.

/**
 Label a control slice carries, so it is never pooled with a flagged class.
 */
export const CONTROL_CLASS = 'control-unflagged';

/**
 Labels the two ends of a relocation candidate carry.

 SEPARATE CLASSES BECAUSE THE WINDOW CAN ONLY EXPLAIN ONE OF THEM. At the HIGH
 end the archive carries English the original does not account for, and the
 neighbouring Chinese is exactly where that English could have come from, so a
 judge shown it can recognise a move rather than a fabrication. At the LOW end
 the archive is missing English, and neighbouring CHINESE cannot say where the
 missing translation went; only the neighbouring ENGLISH could, and this trial
 never shows it. Pooled, the low ends would dilute the high ends with a
 population the treatment cannot reach, and a real effect would read as a
 weaker one.
 */
export const RELOCATION_CLASSES = {
  high: 'relocation-high',
  low: 'relocation-low',
} as const;

/**
 One slice the trial will buy, with the label its rows carry.

 @example
 ```ts
 const pick: TrialSlice = { entryId: 'Mittens', sliceIndex: 7, sliceClass: 'relocation', };
 ```
 */
export type TrialSlice = {
  /**
   Entry the slice belongs to.
   */
  readonly entryId: string;

  /**
   Slice position within that entry's preparation.
   */
  readonly sliceIndex: number;

  /**
   Class the screen flagged, or {@link CONTROL_CLASS}.
   */
  readonly sliceClass: string;
};

/**
 Flagged slices of one entry, deduplicated, each under one class.

 A SLICE FLAGGED TWO WAYS TAKES THE FIRST LABEL IN THIS ORDER, and the order is
 deliberate rather than incidental. The high end of a relocation comes first
 because it is the relocation case itself and the only one the window can reach:
 the archive there carries English the original does not account for, and the
 neighbouring Chinese is where it could have come from. Its low end comes next,
 since a slice that is both ends of two candidates is more informative read as
 the surplus one. The alternative, dropping multiply-flagged slices, would
 discard exactly the ambiguous cases the trial exists to resolve.

 @param entryId - entry these slices belong to

 @param displacement - what the screen found for it

 @returns One entry per flagged slice, no slice twice

 @example
 ```ts
 const flagged = flaggedSlices({ entryId, displacement, },);
 ```
 */
export function flaggedSlices(
  {
    entryId,
    displacement,
  }: {
    readonly entryId: string;
    readonly displacement: DocumentDisplacement;
  },
): readonly TrialSlice[] {
  /**
   Class already assigned to each slice, which is what keeps a slice from
   being bought twice.
   */
  const assigned = new Map<number, string>();

  for (const [sliceClass, indices,] of [
    [
      RELOCATION_CLASSES.high,
      displacement.relocationCandidates
        .map(function toHigh(candidate,): number {
          return candidate.high;
        },),
    ],
    [
      RELOCATION_CLASSES.low,
      displacement.relocationCandidates
        .map(function toLow(candidate,): number {
          return candidate.low;
        },),
    ],
    [
      'untranslated',
      displacement.untranslated,
    ],
    [
      'target-only',
      displacement.targetOnly,
    ],
    [
      'other-imbalance',
      displacement.otherImbalances,
    ],
  ] as const) {
    for (const sliceIndex of indices) {
      if (assigned.has(sliceIndex,))
        continue;
      assigned.set(
        sliceIndex,
        sliceClass,
      );
    }
  }

  return [...assigned,]
    .map(function toSlice([sliceIndex, sliceClass,]: ReadonlyEntry<number, string>,): TrialSlice {
      return {
        entryId,
        sliceIndex,
        sliceClass,
      };
    },)
    .toSorted(function byIndex(
      left,
      right,
    ): number {
      return left.sliceIndex - right.sliceIndex;
    },);
}

/**
 Unflagged slices of the same entry, drawn as controls.

 DRAWN FROM THE SAME ENTRIES as the flagged ones, deliberately. A control from
 elsewhere in the corpus would differ in author, register and era as well as in
 being unflagged, and any of those could move a judge. Same document, same
 preparation, same everything except the thing under test.

 EVENLY SPACED rather than taken from the front, because slices early in a
 document are systematically different: they carry the opening, and several
 entries begin with a heading-plus-stub the screen would not flag but a judge
 reads differently from body prose.

 @param entryId - entry to draw from

 @param displacement - what the screen found for it

 @param wanted - how many controls to draw

 @returns Up to `wanted` unflagged slices, evenly spaced

 @example
 ```ts
 const controls = controlSlices({ entryId, displacement, wanted: 2, },);
 ```
 */
export function controlSlices(
  {
    entryId,
    displacement,
    wanted,
  }: {
    readonly entryId: string;
    readonly displacement: DocumentDisplacement;
    readonly wanted: number;
  },
): readonly TrialSlice[] {
  /**
   Slices this entry had flagged, whatever the class.
   */
  const flagged = new Set(flaggedSlices({
    entryId,
    displacement,
  },)
    .map(function toIndex(slice,): number {
      return slice.sliceIndex;
    },),);

  /**
   Every slice the screen left alone.
   */
  const unflagged = displacement.slices
    .map(function toIndex(
      _classified,
      sliceIndex,
    ): number {
      return sliceIndex;
    },)
    .filter(function notFlagged(sliceIndex,): boolean {
      return !flagged.has(sliceIndex,);
    },);
  if ((wanted <= 0) || (unflagged.length === 0))
    return [];

  /**
   Stride that spreads the draw across the document rather than clustering it.
   */
  const stride = Math.max(
    1,
    Math.floor(unflagged.length / wanted,),
  );

  /**
   Where the first control sits inside its stride.

   CENTRED RATHER THAN AT THE START. Taking position zero of every stride puts
   the first control on the FIRST unflagged slice of the document, and at
   `wanted` of one, where the stride is the whole document, that is the only
   control the entry contributes. Openings are the population this draw was
   written to avoid: several entries begin with a heading and a stub, which a
   judge reads differently from body prose, so a control class made of them
   would report the opening effect and be labelled context-conservatism.
   */
  const offset = Math.floor(stride / 2,);

  return unflagged
    .filter(function onStride(
      _sliceIndex,
      position,
    ): boolean {
      return ((position % stride) === offset);
    },)
    .slice(
    0,
    wanted,
    )
    .map(function toSlice(sliceIndex,): TrialSlice {
      return {
        entryId,
        sliceIndex,
        sliceClass: CONTROL_CLASS,
      };
    },);
}

//endregion Window trial draw
