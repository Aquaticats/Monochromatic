import type { ArtifactDeliveryRow, } from '../dist/final/node/index.mjs';

//region Two-lane ledger fixtures
// A REPAIR AND TRANSLATE LEDGER OVER THE SAME CAT-THEMED SLICES, for cases
// that read what each lane recorded against a shared, fixed source and
// archive wording.
//
// TEST SUPPORT, NOT PACKAGE SOURCE. The repair-lane-records test and the
// two-lane artifact read test kept their own copy of this ledger; both now
// import it from here.

/**
 Original of the slice both lanes work on.
 */
const SOURCE_NAP = '猫猫在窗台上睡觉。';

/**
 Original of the passage the archive never translated.
 */
const SOURCE_BIRD = '窗台上有一只鸟。';

/**
 Archive's own English for the first slice.
 */
export const ARCHIVE_NAP = 'The cat sleeps on the sill.';

/**
 Wording the translate lane decided for it.
 */
const FRESH_NAP = 'The cat naps on the windowsill.';

/**
 Repair lane's ledger: it kept the archive's wording, and had nothing to do at
 a passage the archive never translated.

 @returns Two rows, in document order

 @example
 ```ts
 const rows = repairLedger();
 ```
 */
export function repairLedger(): readonly ArtifactDeliveryRow[] {
  return [
    {
      sliceIndex: 0,
      sourceText: SOURCE_NAP,
      incumbentKind: 'present',
      incumbentText: ARCHIVE_NAP,
      outcome: {
        kind: 'decided',
        acceptedText: ARCHIVE_NAP,
      },
      shippedText: ARCHIVE_NAP,
      delivery: { kind: 'incumbent-retained', },
    },
    {
      sliceIndex: 1,
      sourceText: SOURCE_BIRD,
      incumbentKind: 'absent',
      incumbentText: '',
      outcome: { kind: 'not-applicable', },
      shippedText: '',
      delivery: { kind: 'gap-remains', },
    },
  ];
}

/**
 Translate lane's ledger: it replaced the first slice and could not fill the
 second.

 @returns Two rows, in document order

 @example
 ```ts
 const rows = translateLedger();
 ```
 */
export function translateLedger(): readonly ArtifactDeliveryRow[] {
  return [
    {
      sliceIndex: 0,
      sourceText: SOURCE_NAP,
      incumbentKind: 'present',
      incumbentText: ARCHIVE_NAP,
      outcome: {
        kind: 'decided',
        acceptedText: FRESH_NAP,
      },
      shippedText: FRESH_NAP,
      delivery: { kind: 'replacement-shipped', },
    },
    {
      sliceIndex: 1,
      sourceText: SOURCE_BIRD,
      incumbentKind: 'absent',
      incumbentText: '',
      outcome: { kind: 'unfilled', },
      shippedText: '',
      delivery: { kind: 'gap-remains', },
    },
  ];
}

/**
 Translate lane's raw result, which no case here varies.

 @returns Raw result JSON

 @example
 ```ts
 const raw = translateResult();
 ```
 */
export function translateResult(): Record<string, unknown> {
  return {
    translatedText: `## Section one\n\n${FRESH_NAP}`,
    sliceCount: 2,
    changedSliceCount: 1,
    refusedSliceCount: 0,
    withdrawnSliceCount: 0,
    changedSliceIndices: [0,],
    withdrawnSliceIndices: [],
    resumedSliceCount: 0,
    status: 'unfilled',
    unfilled: [{ sliceIndex: 1, },],
    slices: [],
    sliceSelections: [],
    findings: [],
    sliceTexts: [
      {
        sliceIndex: 0,
        incumbentKind: 'present',
        incumbentText: ARCHIVE_NAP,
        outcome: {
          kind: 'decided',
          acceptedText: FRESH_NAP,
        },
      },
      {
        sliceIndex: 1,
        incumbentKind: 'absent',
        incumbentText: '',
        outcome: { kind: 'unfilled', },
      },
    ],
  };
}

//endregion Two-lane ledger fixtures
