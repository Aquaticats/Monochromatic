import type { ArtifactComparisonRow, } from '../dist/final/node/index.mjs';

//region Two-lane comparison row
// A COMPARISON ROW CARRYING THE REPAIR AND TRANSLATE LANE WORDINGS A TEST
// NEEDS, every other field held constant, over the same fixed archive
// wording.
//
// TEST SUPPORT, NOT PACKAGE SOURCE. Several two-lane contest tests kept
// their own copy of this builder; all now import it from here.

/**
 Archive's wording for the slice every contest row compares.
 */
export const ARCHIVE_NAP = 'The cat sleeps in the bookshop attic.';

/**
 Builds a comparison row carrying the two lane wordings a test needs.

 @param sliceIndex - slice this names

 @param repairText - wording the repair document carries

 @param translateText - wording the translate document carries

 @returns Row with the rest of its fields held constant

 @example
 ```ts
 const row = catRow({ sliceIndex: 0, repairText: REPAIR_NAP, translateText: TRANSLATE_NAP, },);
 ```
 */
export function catRow(
  {
    sliceIndex,
    repairText,
    translateText,
  }: {
    readonly sliceIndex: number;
    readonly repairText: string;
    readonly translateText: string;
  },
): ArtifactComparisonRow {
  return {
    sliceIndex,
    incumbentKind: 'present',
    incumbentText: ARCHIVE_NAP,
    repairText,
    translateText,
    laneRelation: (repairText === translateText) ? 'both-agree' : 'both-differ',
    repairOutcome: {
      kind: 'decided',
      acceptedText: repairText,
    },
    translateOutcome: {
      kind: 'decided',
      acceptedText: translateText,
    },
    decisionComparison: {
      kind: 'comparable',
      verdict: (repairText === translateText) ? 'same' : 'different',
    },
    repairDelivery: { kind: 'replacement-shipped', },
    translateDelivery: { kind: 'replacement-shipped', },
  };
}

//endregion Two-lane comparison row
