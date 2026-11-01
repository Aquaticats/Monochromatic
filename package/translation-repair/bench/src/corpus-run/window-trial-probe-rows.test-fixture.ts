/**
 Test-only arms of the window trial, as the ledger holds them.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import type { WindowTrialRow, } from '../../dist/final/node/index.mjs';

/**
 Arm a row records, one of the trial's three.

 @example
 ```ts
 const arm: ArmName = 'wide';
 ```
 */
type ArmName = 'narrow-a' | 'narrow-b' | 'wide';

/**
 Row one arm of one slice leaves in the ledger.

 @param arm - which of the three arms bought it

 @param shipped - whether the arm replaced the archive's wording

 @param sliceIndex - slice the arm judged, 1 where absent

 @param entryId - entry the slice belongs to, `Mittens` where absent

 @param protocol - digest the arm was bought under, `protocol-one` where absent

 @param sliceClass - class the slice was drawn as, `untranslated` where absent

 @returns The ledger row

 @example
 ```ts
 const row = armRow({ arm: 'wide', shipped: true, },);
 ```
 */
export function armRow(
  {
    arm,
    shipped,
    sliceIndex = 1,
    entryId = 'Mittens',
    protocol = 'protocol-one',
    sliceClass = 'untranslated',
  }: {
    readonly arm: ArmName;
    readonly shipped: boolean;
    readonly sliceIndex?: number;
    readonly entryId?: string;
    readonly protocol?: string;
    readonly sliceClass?: string;
  },
): WindowTrialRow {
  return {
    protocol,
    entryId,
    sliceIndex,
    arm,
    sliceClass,
    shipped,
    decision: shipped ? 'judged' : 'incumbent-kept',
    winnerText: 'The cat sleeps on the sill.',
    judgesHeard: 2,
    judgesSeated: 2,
    position: 0,
  };
}
