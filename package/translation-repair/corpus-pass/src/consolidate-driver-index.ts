import type { ArtifactContestSlice, } from '@monochromatic-dev/translation-repair-lane-contest/ts';
import {
  type ConsolidateRoster,
  consolidateRunShape,
  type ProjectedLanes,
} from '@monochromatic-dev/translation-repair-artifact/ts';
import type { DeclaredNamePair, } from '@monochromatic-dev/translation-repair-house-rule/ts';
import type { PreparedSheetFacts, } from '@monochromatic-dev/translation-repair-corpus-source/ts';

//region Consolidation driver index
// The two lookups the consolidation driver reads per slice, built once over
// the whole document: the original of every slice (the repair ledger alone
// carries it) and the contest record of every slice the contest answered.
// Split out of `consolidate-driver.ts` at the line cap (class one hundred
// seven, 2026-09-24).

/**
 Per-slice lookups the driver settles against.

 @example
 ```ts
 const { sourceTexts, contestBySlice, } = indexConsolidationInputs({ projected, contests, },);
 ```
 */
export type ConsolidationIndex = {
  /**
   Original of each slice, keyed by slice index.
   */
  readonly sourceTexts: ReadonlyMap<number, string>;

  /**
   Contest record for each slice it answered, keyed by slice index.
   */
  readonly contestBySlice: ReadonlyMap<number, ArtifactContestSlice>;
};

/**
 Builds the per-slice lookups once for the whole document.

 @param projected - both lane ledgers and comparison

 @param contests - what the contest settled per slice

 @returns Originals and contest records keyed by slice index

 @example
 ```ts
 const index = indexConsolidationInputs({ projected, contests, },);
 ```
 */
export function indexConsolidationInputs(
  {
    projected,
    contests,
  }: {
    readonly projected: ProjectedLanes;
    readonly contests: readonly ArtifactContestSlice[];
  },
): ConsolidationIndex {
  return {
    sourceTexts: new Map(projected.delivery
      .repair
      .map(function nameSource(row,): readonly [
        number,
        string,
      ] {
        return [
          row.sliceIndex,
          row.sourceText,
        ];
      },),),
    contestBySlice: new Map(contests.map(function nameSlice(slice,): readonly [
      number,
      ArtifactContestSlice,
    ] {
      return [
        slice.sliceIndex,
        slice,
      ];
    },),),
  };
}

/**
 One comparison row beside the contest record that settled it.

 @example
 ```ts
 const contested: ContestedRow = { row, contest, };
 ```
 */
export type ContestedRow = {
  /**
   Both lanes' texts at this slice.
   */
  readonly row: ProjectedLanes['comparison'][number];

  /**
   What the contest settled here.
   */
  readonly contest: ArtifactContestSlice;
};

/**
 Comparison rows beside the contests that settled them, in document order;
 a slice the contest never answered is left out.

 @param projected - both lane ledgers and comparison

 @param contestBySlice - contest record per slice it answered

 @returns Rows the consolidation settles

 @example
 ```ts
 const eligibleRows = contestedRows({ projected, contestBySlice, },);
 ```
 */
export function contestedRows(
  {
    projected,
    contestBySlice,
  }: {
    readonly projected: ProjectedLanes;
    readonly contestBySlice: ReadonlyMap<number, ArtifactContestSlice>;
  },
): readonly ContestedRow[] {
  return projected.comparison
    .flatMap(function withContest(row,): readonly ContestedRow[] {
      /**
       What the contest settled here, absent where it never ran.
       */
      const contest = contestBySlice.get(row.sliceIndex,);
      return (contest === undefined)
        ? []
        : [{
          row,
          contest,
        },];
    },);
}

/**
 Names a contested slice on the lines the overlapped map writes about a
 failure it leaves unthrown.

 @param item - contested row and its contest

 @returns The slice by its index

 @example
 ```ts
 contestedSliceName({ item: { row: { sliceIndex: 3, }, }, },); // 'slice 3'
 ```
 */
export function contestedSliceName(
  { item: { row, }, }: { readonly item: { readonly row: { readonly sliceIndex: number; }; }; },
): string {
  return `slice ${String(row.sliceIndex,)}`;
}

/**
 What a run on one roster asks, folded into every key it settles, so a slice
 seated on a re-read roster is keyed by the roster that settled it.

 @param roster - writers, judges and naturalness roles the slice runs on

 @param sheetFacts - what the page tells every sheet and key, none when the
 page tells none

 @param declaredNamePairs - name pairs the front matter declares

 @returns Run shape for that roster

 @example
 ```ts
 const runShape = rosterRunShapeOf({ roster, sheetFacts, declaredNamePairs, },);
 ```
 */
export function rosterRunShapeOf(
  {
    roster,
    sheetFacts,
    declaredNamePairs,
  }: {
    readonly roster: ConsolidateRoster;
    readonly sheetFacts?: PreparedSheetFacts;
    readonly declaredNamePairs: readonly DeclaredNamePair[];
  },
): string {
  return consolidateRunShape({
    modelIds: roster.modelIds,
    judgeModelIds: roster.judgeModelIds,
    ...sheetFacts,
    ...((roster.polishConfig === undefined) ? {} : { polishConfig: roster.polishConfig, }),
    declaredNamePairs,
  },);
}

//endregion Consolidation driver index
