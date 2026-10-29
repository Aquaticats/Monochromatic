import {
  type Logger,
  tagged,
} from '@monochromatic-dev/logger-core/ts';

import type {
  InsertionAdmission,
  PreparedDocumentPair,
} from '@monochromatic-dev/translation-repair-corpus-source/ts';
import { foldCarriedInsertions, } from '@monochromatic-dev/translation-repair-insertion/ts';

//region Pass carried fold
// The fold of every carried passage into the neighbour whose archive span
// renders it, with the lines the run log reads of it. Split out of
// `pass-entry.ts` so those lines go through a logger the caller hands in,
// which a test reads (T8 batch 9: the stand-aside warning ran in no test).

/**
 Folds every carried passage into the neighbour whose archive span renders
 it, and logs what folded, what stayed carried and why, and the regions each
 carried passage's coverage rests on.

 BOTH LANES THEN WRITE THE CARRIER FROM THE SOURCE IT RENDERS (class one
 hundred ten, mikaela12), instead of dropping the passage the archive merged
 in. A passage that stays carried says why at warn (class one hundred eleven,
 mikaela13), so the log reader can tell a scattered rendering from one the
 fold could not place. Every carried passage, folded or not, prints the
 regions its coverage rests on, so a carried-evidence-lost stop at publish or
 a fold's choice of carrier (class one hundred seventy-nine) can be read
 against the admission without the artifact.

 @param paired - preparation the admission read

 @param admission - insertion admission as the coverage round read it

 @param l - entry logger

 @returns Preparation and admission after the fold

 @example
 ```ts
 const { prepared, admission, } = foldPassCarried({ paired, admission: admissionAsRead, l, },);
 ```
 */
export function foldPassCarried(
  {
    paired,
    admission,
    l,
  }: {
    readonly paired: PreparedDocumentPair;
    readonly admission: InsertionAdmission;
    readonly l: Logger;
  },
): {
  readonly prepared: PreparedDocumentPair;
  readonly admission: InsertionAdmission;
} {
  /**
   Entry logger, tagged with this function's name.
   */
  const fl = tagged({
    l,
    tag: foldPassCarried.name,
  },);
  /**
   The fold's outcome.
   */
  const {
    prepared,
    admission: folded,
    findings,
    asides,
  } = foldCarriedInsertions({
    prepared: paired,
    admission,
  },);
  for (const finding of findings)
    fl.info(finding,);
  for (const aside of asides)
    fl.warn(aside,);
  for (const carried of admission.carried ?? []) {
    /**
     Regions the coverage voices quoted, one JSON string each.
     */
    const regions = carried.evidence
      .map(function preview(region,): string {
        return JSON.stringify(region,);
      },)
      .join(' | ',);
    /**
     How many regions the voices quoted.
     */
    const regionCount = carried.evidence
      .length;
    fl.info(
      `slice ${String(carried.sliceIndex,)} carried on ${String(regionCount,)} region(s): ${regions}`,
    );
  }
  return {
    prepared,
    admission: folded,
  };
}

//endregion Pass carried fold
