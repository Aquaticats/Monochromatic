import type { DeclaredNamePair, } from '@monochromatic-dev/translation-repair-house-rule/ts';
import type {
  ConsolidateSubject,
  ConsolidationSubject,
} from '@monochromatic-dev/translation-repair-consolidate/ts';
import type { DisputedWording, } from '@monochromatic-dev/translation-repair-translate-floor/ts';
import type { LaneContestBallot, } from '@monochromatic-dev/translation-repair-lane-contest/ts';
import type { PreparedSheetFacts, } from '@monochromatic-dev/translation-repair-corpus-source/ts';
import type { SliceSyntax, } from '@monochromatic-dev/translation-repair-slice-reading/ts';

//region Consolidation driver subject
// The slice as both halves of the consolidation take it, split out of
// `consolidate-driver.ts` at the line cap when the claims the repair panel
// accepted joined it (CuspariaKLSY21 slice 0, 2026-10-21).

/**
 Builds the slice both the producers and the judging half are given.

 EVERY OPTIONAL FIELD IS ABSENT RATHER THAN EMPTY, so a producer shown no
 readings is shown no heading promising any, and a sheet and its key cannot
 disagree about which spelling of nothing the caller used.

 THE WINDOW REACHES THE JUDGING HALF ONLY, for now. The producer sheet has no
 block for it, so putting it here promises nothing to a producer and gives
 `settleConsolidation` what its judges need. Whether the producers should have
 it too is a real question, recorded as an explicit exclusion rather than
 answered in passing.

 THE ACCEPTED REPAIR CLAIMS REACH THE FINAL POLISH GATE ONLY, as the window
 reaches the judges only: the producer sheet has no block for them.

 @param sourceText - original of the slice

 @param incumbentText - archive wording as this settlement takes it

 @param repairText - repair lane's wording

 @param translateText - translate lane's wording

 @param ballots - what the contest judges said

 @param syntax - syntax role of the slice, absent for body prose

 @param lineStructured - whether the line-structure rule governs the slice

 @param declared - name pairs the front matter declares

 @param sheetFacts - what the page tells every sheet, absent when it tells none

 @param disputeNote - sheet note of a disputed slice, absent otherwise

 @param acceptedRepairClaimsNote - claims the repair panel accepted, absent
 where it accepted none

 @param disputedWordings - wordings the slice refuses

 @param pictureContext - what the pictures near the slice were read to say,
 empty where none were

 @param neighbours - passages either side of the slice, empty for a lone slice

 @returns Subject for the producers and the judging half

 @example
 ```ts
 const subject = consolidationSubjectOf({ sourceText, incumbentText, repairText, translateText, ballots, lineStructured, declared, disputedWordings, pictureContext, neighbours, },);
 ```
 */
export function consolidationSubjectOf(
  {
    sourceText,
    incumbentText,
    repairText,
    translateText,
    ballots,
    syntax,
    lineStructured,
    declared,
    sheetFacts,
    disputeNote,
    acceptedRepairClaimsNote,
    disputedWordings,
    pictureContext,
    neighbours,
  }: {
    readonly sourceText: string;
    readonly incumbentText: string;
    readonly repairText: string;
    readonly translateText: string;
    readonly ballots: readonly LaneContestBallot[];
    readonly syntax?: SliceSyntax;
    readonly lineStructured: boolean;
    readonly declared: readonly DeclaredNamePair[];
    readonly sheetFacts?: PreparedSheetFacts;
    readonly disputeNote?: string;
    readonly acceptedRepairClaimsNote?: string;
    readonly disputedWordings: readonly DisputedWording[];
    readonly pictureContext: string;
    readonly neighbours: {
      readonly sourceText: string;
      readonly incumbentText: string;
    };
  },
): ConsolidateSubject & ConsolidationSubject {
  return {
    sourceText,
    incumbentText,
    repairText,
    translateText,
    ballots,
    ...((syntax === undefined) ? {} : { syntax, }),
    lineStructured,
    declared,
    ...sheetFacts,
    ...((disputeNote === undefined) ? {} : { archiveDisputeNote: disputeNote, }),
    ...((acceptedRepairClaimsNote === undefined) ? {} : { acceptedRepairClaimsNote, }),
    ...((disputedWordings.length === 0) ? {} : { disputedWordings, }),
    ...((pictureContext === '') ? {} : { pictureContext, }),
    ...((neighbours.sourceText === '') ? {} : { neighbouringSourceText: neighbours.sourceText, }),
    ...((neighbours.incumbentText === '')
      ? {}
      : { neighbouringIncumbentText: neighbours.incumbentText, }),
  };
}

//endregion Consolidation driver subject
