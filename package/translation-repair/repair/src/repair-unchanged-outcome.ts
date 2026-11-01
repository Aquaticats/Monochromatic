import type { ChunkCriticPhase, } from './chunk-critic-phase.ts';
import type { ChunkRepairOutcome, } from '@monochromatic-dev/translation-repair-assembly/ts';
import { UNATTRIBUTED_TEXT, } from '@monochromatic-dev/translation-repair-house-rule/ts';

//region Unchanged chunk outcome
// The record a chunk leaves behind when the translation ships exactly as it
// stood. Three exits in `repair-chunk.ts` reach this state for three different
// reasons, and they spread one shape rather than restating it, so a field added
// to the contract cannot land on two of them and be forgotten on the third.

/**
 Builds the outcome for a chunk whose text nothing changed.

 THE CRITICS' WORK IS KEPT, which is the whole reason this is not a bare
 empty record. Nothing was repaired, but the votes, the contradiction screen
 and the attributions were all paid for and are read by later calibration.

 @param sliceIndex - position of this chunk in the document

 @param targetText - translation as it stood, which is what ships

 @param critic - critic phase result whose telemetry survives the exit

 @returns Outcome every unchanged exit spreads, less the two fields each exit supplies itself

 @example
 ```ts
 const unchangedOutcome = unchangedChunkOutcome({ sliceIndex, targetText, critic, },);
 ```

 @internal
 */
export function unchangedChunkOutcome(
  {
    sliceIndex,
    targetText,
    critic,
  }: {
    readonly sliceIndex: number;
    readonly targetText: string;
    readonly critic: ChunkCriticPhase;
  },
): Omit<ChunkRepairOutcome, 'issues' | 'findings'> {
  return {
    sliceIndex,
    repairedText: targetText,
    changed: false,
    resolvedIssueIds: [],
    // NO CHECKER RAN, so there is nothing to have said.
    checkerReadings: {},
    recheckReadings: {},
    candidateResolvedIssueIds: [],
    repairRegions: [],
    accuracyPatchSelected: false,
    refined: false,
    rounds: [],
    // Nothing was repaired on any exit that spreads this, so no model wrote the
    // text and no checker can be certifying its own work.
    authorship: UNATTRIBUTED_TEXT,
    droppedDeclaredNames: [],
    nonTranslationVotes: critic.nonTranslationVotes,
    nonTranslationContradicted: critic.contradicted,
    nonTranslationStanding: critic.votesStand,
    heardCritics: critic.heardCritics,
    heardCriticIds: critic.heardCriticIds,
    claimAttributions: critic.claimAttributions,
    // NO STAGE AFTER THE CRITICS RAN on the exits that spread this before the panel; a later exit gives the panel's.
    silentStages: [],
  };
}

//endregion Unchanged chunk outcome
