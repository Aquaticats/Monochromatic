import type { RosterModelId, } from '@monochromatic-dev/translation-repair-shared-contract/ts';

//region Bench seating
// THE BENCH A PER-ITEM HOOK HANDS A PHASE THAT ASKS ONE BENCH (ledger X12):
// the lane contest's judges, the insertion admission's roster and the
// preparation's roster. Phases asking several benches at once hand a roster
// of their own shape (`ConsolidateSliceSeating`, `RepairSliceSeating`,
// `TranslateSliceSeating`, `PictureReaderSeating`).

/**
 What a per-item hook hands a phase that asks one bench.

 @example
 ```ts
 const seating: BenchSeating = { modelIds: seats.lateJudges, };
 ```
 */
export type BenchSeating = {
  /**
   Bench read since the phase started, absent while the given one stands.
   */
  readonly modelIds?: readonly RosterModelId[];

  /**
   Seats of that bench a wet provider would serve as of the reading, which a
   phase with a floor on its voices compares to the floor before it asks
   (the lane contest does); absent where the phase did not count.
   */
  readonly reachable?: number;
};

/**
 Hook for a caller that re-seats nothing: every item keeps the bench the
 phase started on.

 @returns No bench, so the given one stands

 @example
 ```ts
 const attestation = await attestPassReferences({ ...input, beforeItem: keepBench, },);
 ```
 */
export function keepBench(): Promise<BenchSeating> {
  return Promise.resolve({},);
}

//endregion Bench seating
