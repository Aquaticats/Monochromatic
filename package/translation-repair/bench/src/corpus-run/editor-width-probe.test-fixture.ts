/**
 The facts one slice of the editor width probe contributes, built from the few
 a case turns on, for the cases of the modules that draw, spend and report the
 draw without asking a model.

 @module
 */

import type {
  WidthComparison,
  WidthProbeInput,
  WidthRow,
} from '../../dist/final/node/index.mjs';

/**
 Builds the work a slice carries for the editors, with no issue and no envelope.

 @param entryId - entry the slice is cut from

 @param sliceIndex - position within that entry's slices

 @returns Input whose texts name a cat's nap

 @example
 ```ts
 const input = widthInputOf({ entryId: 'mittens', sliceIndex: 0, },);
 ```
 */
export function widthInputOf(
  {
    entryId,
    sliceIndex,
  }: {
    readonly entryId: string;
    readonly sliceIndex: number;
  },
): WidthProbeInput {
  return {
    entryId,
    sliceIndex,
    sourceText: '小猫在窗台上打盹。\n',
    targetText: 'The kitten dozes on the windowsill.\n',
    issues: [],
    envelopes: [],
    findings: [],
  };
}

/**
 Builds the row a slice contributed when both arms shipped and the head-to-head was not run.

 @param entryId - entry the slice came from

 @param sliceIndex - position within that entry's slices

 @param comparison - how the two widths compared

 @param narrowRepeatAgreed - whether the narrow arm run twice shipped the same text

 @returns Row with no producer named

 @example
 ```ts
 const row = widthRowOf({ entryId: 'mittens', sliceIndex: 0, comparison: 'same-text', narrowRepeatAgreed: true, },);
 ```
 */
export function widthRowOf(
  {
    entryId,
    sliceIndex,
    comparison,
    narrowRepeatAgreed,
  }: {
    readonly entryId: string;
    readonly sliceIndex: number;
    readonly comparison: WidthComparison;
    readonly narrowRepeatAgreed: boolean;
  },
): WidthRow {
  return {
    entryId,
    sliceIndex,
    acceptedIssues: 1,
    comparison,
    heardNarrow: 2,
    heardWide: 3,
    narrowShipped: true,
    wideShipped: true,
    narrowRepeatAgreed,
    verdict: 'not-run',
    usableBallots: 0,
    narrowProducers: [],
    wideProducers: [],
  };
}
