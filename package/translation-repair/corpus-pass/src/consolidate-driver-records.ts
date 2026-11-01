import {
  type ArtifactConsolidateSlice,
  describeConsolidateSlice,
  withShippedTenseCandidates,
} from '@monochromatic-dev/translation-repair-artifact/ts';
import type { ConsolidationSettlement, } from '@monochromatic-dev/translation-repair-consolidate/ts';
import type {
  ArtifactContestVerdict,
  LaneChoice,
} from '@monochromatic-dev/translation-repair-lane-contest/ts';
import type { PreparedSheetFacts, } from '@monochromatic-dev/translation-repair-corpus-source/ts';
import type { TwinStored, } from '@monochromatic-dev/translation-repair-stage/ts';

//region Consolidate driver records
// The consolidation driver's small helpers, moved out of
// `consolidate-driver.ts` at its line budget: what a fresh purchase leaves a
// twin, which lane a contest verdict backed, and the artifact record of one
// settled slice.

/**
 Fresh consolidation beside whether it became warm-run evidence.

 @example
 ```ts
 const bought: BoughtConsolidation = { settlement, persisted: true, };
 ```
 */
export type BoughtConsolidation = {
  readonly settlement: ConsolidationSettlement;
  readonly persisted: boolean;
};

/**
 Reads cache-eligible record from fresh consolidation.

 @param bought - fresh result beside persistence status

 @returns Record a twin may reuse, or deliberate nothing

 @example
 ```ts
 const stored = storedConsolidationOf({ settlement, persisted: true, },);
 ```
 */
export function storedConsolidationOf(
  bought: BoughtConsolidation,
): TwinStored<ConsolidationSettlement> {
  return bought.persisted
    ? {
      kind: 'stored',
      record: bought.settlement,
    }
    : { kind: 'nothing', };
}

/**
 Reads which lane the contest backed out of the verdict it recorded.

 BOTH WAYS OF NOT SETTLING READ AS `neither`, deliberately. The record keeps
 `settled-neither` apart from `quorum-not-met` because they are different
 facts about the run, but this function asks which LANE stood. Neither did.
 `standingTextFor` then uses archive as comparison baseline so consolidation
 can recover, while final-selection guard prevents that unendorsed baseline
 from becoming publication fallback.

 @param verdict - what the contest recorded for this slice

 @returns Lane the contest backed, or the refusal

 @example
 ```ts
 const choice = laneChoiceOf({ verdict, },);
 ```
 */
export function laneChoiceOf(
  { verdict, }: { readonly verdict: ArtifactContestVerdict; },
): LaneChoice {
  // A SOLE LANE IS A WIN FOR THAT LANE (ledger L6): the consolidation and its
  // polish run on it exactly as on a contest the lane won.
  if ((verdict.kind === 'lane-won') || (verdict.kind === 'sole-lane'))
    return verdict.lane;
  return 'neither';
}

/**
 Records one settled slice for the artifact: the settlement as the artifact
 describes it, and the tense candidates of the wording it ships where the
 entry records a death. A fresh settlement and one resumed from the cache are
 recorded alike, because the finding is read off the settled wording and not
 stored with it.

 @param sliceIndex - slice this answers

 @param settlement - what the stage settled

 @param incumbentStandsIn - whether the standing the settlement ran against
 was the incumbent standing in for an ineligible lane standing

 @param sheetFacts - what the page tells every sheet: whether its entry records a death and the names the translation declares for its subject; absent when the document carries none

 @returns Record for one consolidated slice

 @example
 ```ts
 const slice = recordedConsolidateSlice({ sliceIndex: 0, settlement, incumbentStandsIn: false, },);
 ```
 */
export function recordedConsolidateSlice(
  {
    sliceIndex,
    settlement,
    incumbentStandsIn,
    sheetFacts,
  }: {
    readonly sliceIndex: number;
    readonly settlement: ConsolidationSettlement;
    readonly incumbentStandsIn: boolean;
    readonly sheetFacts?: PreparedSheetFacts;
  },
): ArtifactConsolidateSlice {
  return withShippedTenseCandidates({
    slice: describeConsolidateSlice({
      sliceIndex,
      settlement,
      incumbentStandsIn,
    },),
    lifeEnded: sheetFacts?.lifeEnded === true,
    ...((sheetFacts?.subjectNames === undefined) ? {} : { subjectNames: sheetFacts.subjectNames, }),
  },);
}

//endregion Consolidate driver records
