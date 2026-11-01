/**
 Readings of the displacement probe built by hand, for cases that need an
 entry's row and not the slicing behind it.

 TEST SUPPORT, NOT PACKAGE SOURCE. The totals, report and run cases of the
 displacement probe all build rows; they import this one builder.

 Fixtures are cat-themed invention.

 @module
 */

import type { EntryDisplacement, } from '../../dist/final/node/index.mjs';

/**
 A row that read one slice and found nothing, which the caller changes in the
 fields a case is about.

 @param changes - fields to set apart from the quiet reading

 @returns The row

 @example
 ```ts
 const row = rowOf({ changes: { entryId: 'Tabby', untranslated: [3,], }, },);
 ```
 */
export function rowOf(
  { changes, }: {
    readonly changes: {
      readonly entryId?: string;
      readonly sliceCount?: number;
      readonly baseline?: number;
      readonly baselineFrom?: EntryDisplacement['baselineFrom'];
      readonly untranslated?: readonly number[];
      readonly targetOnly?: readonly number[];
      readonly relocationCandidates?: EntryDisplacement['relocationCandidates'];
      readonly transcriptionSuspects?: readonly number[];
      readonly markupDonors?: readonly number[];
      readonly otherImbalances?: readonly number[];
    };
  },
): EntryDisplacement {
  return {
    entryId: 'Mittens',
    sliceCount: 1,
    baseline: 2.86,
    baselineFrom: 'document',
    untranslated: [],
    targetOnly: [],
    relocationCandidates: [],
    transcriptionSuspects: [],
    markupDonors: [],
    otherImbalances: [],
    ...changes,
  };
}
