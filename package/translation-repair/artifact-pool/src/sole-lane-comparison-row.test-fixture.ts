import type { ArtifactComparisonRow, } from '../dist/final/node/index.mjs';

//region Sole-lane comparison row
// A COMPARISON ROW FOR AN ARCHIVE-ABSENT INSERTION, where the repair lane
// recorded `not-applicable` and only the translate lane has wording, for cases
// that read a sole-lane contest against the row it answers.
//
// TEST SUPPORT, NOT PACKAGE SOURCE. The row carries the outcomes the ledger
// records for an insertion; `catRow` cannot stand for one because it marks
// both lanes decided.

/**
 Builds an insertion comparison row the translate lane filled.

 @param sliceIndex - slice this names

 @param translateText - wording the translate lane shipped at the insertion

 @returns Row whose repair lane has nothing to mend at this slice

 @example
 ```ts
 const row = insertionRow({ sliceIndex: 0, translateText: 'The cat dozes.', },);
 ```
 */
export function insertionRow(
  {
    sliceIndex,
    translateText,
  }: {
    readonly sliceIndex: number;
    readonly translateText: string;
  },
): ArtifactComparisonRow {
  return {
    sliceIndex,
    incumbentKind: 'absent',
    incumbentText: '',
    repairText: '',
    translateText,
    laneRelation: 'translate-only',
    repairOutcome: { kind: 'not-applicable', },
    translateOutcome: {
      kind: 'decided',
      acceptedText: translateText,
    },
    decisionComparison: {
      kind: 'not-comparable',
      undecidedLanes: ['repair',],
    },
    repairDelivery: { kind: 'gap-remains', },
    translateDelivery: { kind: 'replacement-shipped', },
  };
}

//endregion Sole-lane comparison row
