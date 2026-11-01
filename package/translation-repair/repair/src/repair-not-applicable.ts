import type { ChunkRepairOutcome, } from '@monochromatic-dev/translation-repair-assembly/ts';
import { UNATTRIBUTED_TEXT, } from '@monochromatic-dev/translation-repair-house-rule/ts';

//region Repair not applicable
// What the repair lane has to say about a passage the archive never translated:
// nothing, said explicitly.
//
// The lane MENDS EXISTING WORDING. Its critics compare a translation against
// its original and file defects in what is there; its editor rewrites regions
// those defects name; its checkers confirm the defects are gone. Handed an
// anchor, which names a boundary where a rendering belongs and none exists,
// every one of those stages is asked about text that does not exist. What comes
// back is not a repair: it is a critic inventing complaints about a blank, at
// full roster cost, on every inserted slice.
//
// SAID RATHER THAN SKIPPED. The outcome list is read BY POSITION against the
// slice list, so an anchor cannot simply be passed over; and a lane that
// silently produced nothing for a slice would be indistinguishable from one
// that examined it and found it clean. This states the third thing: the
// question does not apply here, and the translate lane owns this passage.
//
// NOT CACHED, because it costs nothing to state and depends on nothing a run
// can change. A cache entry would only add a way for it to go stale.

/**
 Finding naming a slice the repair lane has nothing to say about.

 @param sliceIndex - slice this describes

 @returns Finding in scorecard-stable wording

 @example
 ```ts
 const finding = notApplicableFinding({ sliceIndex: 4, },);
 ```
 */
export function notApplicableFinding(
  { sliceIndex, }: { readonly sliceIndex: number; },
): string {
  return `repair-not-applicable chunk ${String(sliceIndex,)}; no translation to repair`;
}

/**
 Builds the outcome for a slice with no translation to repair.

 EVERY COUNT ZERO AND EVERY LIST EMPTY, which is the honest reading rather
 than a placeholder: no critic was heard because none was asked, no issue was
 filed because there was nothing to file one about, and nothing changed
 because there is nothing here to change. The finding is what says why, and it
 is the only thing this outcome asserts.

 @param sliceIndex - slice this outcome describes

 @returns Outcome carrying the archive's own absence of wording

 @example
 ```ts
 const outcome = notApplicableRepair({ sliceIndex: 4, },);
 ```
 */
export function notApplicableRepair(
  { sliceIndex, }: { readonly sliceIndex: number; },
): ChunkRepairOutcome {
  return {
    sliceIndex,
    // The archive's wording for an anchor, which is none of it. Assembly reads
    // `changed` rather than this, so nothing is written at that boundary.
    repairedText: '',
    changed: false,
    // No repair was attempted at all, so nothing here has an author.
    authorship: UNATTRIBUTED_TEXT,
    issues: [],
    claimAttributions: [],
    heardCriticIds: [],
    resolvedIssueIds: [],
    // NO CHECKER RAN, so there is nothing to have said.
    checkerReadings: {},
    recheckReadings: {},
    candidateResolvedIssueIds: [],
    repairRegions: [],
    accuracyPatchSelected: false,
    refined: false,
    rounds: [],
    droppedDeclaredNames: [],
    nonTranslationVotes: 0,
    // NOT a non-translation verdict. An untranslated passage is exactly what
    // that block is meant to catch, and counting anchors toward it would block
    // documents for the gaps this work exists to fill: the dominance ratio is
    // over target characters, and an anchor carries none, so it can neither
    // stand nor contradict.
    nonTranslationContradicted: false,
    nonTranslationStanding: false,
    heardCritics: 0,
    findings: [notApplicableFinding({ sliceIndex, },),],
    silentStages: [],
  };
}

//endregion Repair not applicable
