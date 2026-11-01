import type { PatchOperation, } from '@monochromatic-dev/translation-repair-page-pass/ts';
import { measurePatchedCandidate, } from './chunk-measure.ts';
import { wordForCount, } from '@monochromatic-dev/translation-repair-typography/ts';
import { findDroppedDeclaredNames, } from '@monochromatic-dev/translation-repair-house-rule/ts';
import {
  type AdjudicatedIssue,
  type EditableEnvelope,
  parseDocument,
  type RepairDocument,
} from '@monochromatic-dev/translation-repair-shared-contract/ts';
import {
  type CandidateMeasurements,
  type IssueResolutionTally,
  type RepairCandidate,
  selectRepairCandidate,
  UNCHANGED_CANDIDATE_ID,
  UNCHANGED_MEASUREMENTS,
  winnerChangedText,
} from '@monochromatic-dev/translation-repair-adjudication/ts';

//region Repair chunk verdict
// What one slice ends up saying about itself: which candidate won, and whether
// the text it returns differs from the archive's.
//
// THOSE ARE TWO QUESTIONS, and a slice answers both. Selection decides whether
// a repair proved itself; the text decides what the document carries. They come
// apart when a patch whose envelope operations cancel wins selection and writes
// no byte, and every count downstream reads one or the other, so a slice that
// conflates them is wrong in one of the two places.
//
// A THIRD ANSWER SITS BESIDE THEM: whether a declared name was refused here. The
// editor's judges lose one six times out of six when asked, measured against
// their own sheet and roster, so the loss is refused rather than argued about,
// and `declared-name-survival.ts` records why that is a guard and not a prompt.
//
// REFUSED HERE RATHER THAN OVERRIDDEN AFTERWARDS, because a caller that reset
// the text after this returned would produce `patchSelected` true with `changed`
// false, which this file already uses for something else entirely: a patch that
// won on the measurements and whose envelope operations cancelled. The two would
// then be one shape meaning two things. Selection still reports what it found,
// since the patch did beat the archive on the measurements and hiding that would
// make the refusal unauditable; what changes is only what ships.

/**
 What a slice settled on.

 @example
 ```ts
 const verdict: ChunkVerdict = { repairedText, patchSelected: true, changed: true, };
 ```
 */
export type ChunkVerdict = {
  /**
   Wording this slice returns, which is the winner's own text.
   */
  readonly repairedText: string;

  /**
   Whether the patched candidate beat the archive on the measurements.
   */
  readonly patchSelected: boolean;

  /**
   Whether the returned wording differs from the archive's, which is what
   assembly and every shipped count read.
   */
  readonly changed: boolean;

  /**
   Declared names the winning patch would have dropped, empty when it dropped
   none.

   NON-EMPTY MEANS THE PATCH WAS REFUSED, and is the only way to tell that
   apart from a slice where the archive simply won on the measurements.
   */
  readonly droppedDeclaredNames: readonly string[];
};

/**
 What a slice settled on, in the words of the settlement line.

 THREE KINDS OF UNCHANGED (ledger L12): the archive beat the patch on the
 measurements, the patch won and its operations wrote no byte, or the patch
 won and was refused for dropping a declared name. The line said "unchanged"
 for all three, so the log could not tell a lost repair from one that wrote
 nothing.

 @param changed - whether the returned wording differs from the archive's

 @param patchSelected - whether the patched candidate beat the archive

 @param refused - whether the winning patch was refused for a declared name

 @returns State phrase the line opens with

 @example
 ```ts
 settlementState({ changed: false, patchSelected: false, refused: false, },); // 'unchanged, the archive won'
 ```
 */
function settlementState(
  {
    changed,
    patchSelected,
    refused,
  }: {
    readonly changed: boolean;
    readonly patchSelected: boolean;
    readonly refused: boolean;
  },
): string {
  if (changed)
    return 'repaired';
  if (refused)
    return 'unchanged, the patch won and was refused for dropping a declared name';
  if (patchSelected)
    return 'unchanged, the patch won and wrote nothing';
  return 'unchanged, the archive won';
}

/**
 Runs the slate for one slice and reads both verdicts off the winner.

 The unchanged translation always competes, so it is built here rather than by
 the caller: an archive that has to be passed in as a candidate is an archive a
 caller can get wrong, and `selectRepairCandidate` refuses a slate whose
 unchanged entry carries anything else.

 @param sliceIndex - slice being settled, which names the patched candidate

 @param incumbentText - archive wording of this slice

 @param patchedText - wording the editor produced

 @param measurements - what the checks and the resolution stage measured about
 that patched wording

 @param declaredNames - name forms the archive's front matter declares, which
 the patch may not drop

 @param declaredSubjectForms - the declared forms among them that name the subject, which may stand in for one another

 @returns Returned wording plus every verdict

 @throws {@link Error} when the slate cannot be formed, which cannot happen
 from here since this function builds it

 @example
 ```ts
 const verdict = settleChunkVerdict({ sliceIndex, incumbentText, patchedText, measurements, declaredNames, },);
 ```
 */
export function settleChunkVerdict(
  {
    sliceIndex,
    incumbentText,
    patchedText,
    measurements,
    declaredNames,
    declaredSubjectForms,
  }: {
    readonly sliceIndex: number;
    readonly incumbentText: string;
    readonly patchedText: string;
    readonly measurements: CandidateMeasurements;
    readonly declaredNames: readonly string[];
    readonly declaredSubjectForms?: readonly string[];
  },
): ChunkVerdict {
  /**
   Archive and repair, ranked against each other.
   */
  const candidates: readonly RepairCandidate[] = [
    {
      candidateId: UNCHANGED_CANDIDATE_ID,
      text: incumbentText,
      measurements: UNCHANGED_MEASUREMENTS,
    },
    {
      candidateId: `candidate/chunk-${String(sliceIndex,)}`,
      text: patchedText,
      measurements,
    },
  ];

  /**
   Selection between them.
   */
  const selection = selectRepairCandidate({
    candidates,
    incumbentText,
  },);
  /**
   Whether the patch beat the archive on the measurements.
   */
  const patchSelected = selection.winner
    .candidateId
    !== UNCHANGED_CANDIDATE_ID;

  /**
   Declared names the winner would have taken out of the document.

   ASKED ONLY OF A WINNING PATCH. A slice the archive won needs no permission
   to stay as it is, and reporting a refusal there would name a protection
   that protected nothing.
   */
  const droppedDeclaredNames = patchSelected
    ? findDroppedDeclaredNames({
      forms: declaredNames,
      ...((declaredSubjectForms === undefined) ? {} : { subjectForms: declaredSubjectForms, }),
      baseText: incumbentText,
      candidateText: selection.winner
        .text,
    },)
    : [];
  if (droppedDeclaredNames.length > 0) {
    return {
      repairedText: incumbentText,
      patchSelected,
      changed: false,
      droppedDeclaredNames,
    };
  }
  return {
    repairedText: selection.winner
      .text,
    patchSelected,
    changed: winnerChangedText({
      winner: selection.winner,
      incumbentText,
    },),
    droppedDeclaredNames,
  };
}

/**
 What one slice settled on, plus the issue ids that survived the checkers.

 @example
 ```ts
 const settled: SettledChunk = { ...verdict, resolvedIssueIds: [], };
 ```
 */
export type SettledChunk = ChunkVerdict & {
  /**
   Accepted issues the checker majority confirmed fixed.
   */
  readonly resolvedIssueIds: readonly string[];
};

/**
 Turns the editor's patch and the checkers' tallies into a settlement.

 MEASURING AND SELECTING ARE ONE STEP HERE, though they are two functions,
 because the resolved count is both an input to the measurement and an output
 of the slice. Computed at the caller they drift: in one recorded case a
 count credited issues no applied envelope ever served, and the only reason
 that was findable is that one place owned both.

 @param sliceIndex - slice being settled

 @param incumbentText - archive wording of this slice

 @param patchedText - wording the editor produced

 @param appliedOperations - envelope operations the editor actually applied

 @param creditableIssues - accepted issues an applied envelope served, which
 are the only ones a repair may be credited for

 @param tallies - checker verdicts by issue id

 @param envelopes - eligible paragraphs the editor was allowed to touch

 @param targetDocument - archive parsed, the grammar baseline

 @param declaredNames - name forms the patch may not drop

 @param declaredSubjectForms - the declared forms among them that name the subject, which may stand in for one another

 @returns Every verdict plus the resolved ids

 @throws {@link Error} when the slate cannot be formed, which cannot happen
 from here since this function builds it

 @example
 ```ts
 const settled = settleChunkFromChecks({ sliceIndex, incumbentText, patchedText, ... },);
 ```
 */
export function settleChunkFromChecks(
  {
    sliceIndex,
    incumbentText,
    patchedText,
    appliedOperations,
    creditableIssues,
    tallies,
    envelopes,
    targetDocument,
    declaredNames,
    declaredSubjectForms,
  }: {
    readonly sliceIndex: number;
    readonly incumbentText: string;
    readonly patchedText: string;
    readonly appliedOperations: readonly PatchOperation[];
    readonly creditableIssues: readonly AdjudicatedIssue[];
    readonly tallies: Readonly<Record<string, IssueResolutionTally>>;
    readonly envelopes: readonly EditableEnvelope[];
    readonly targetDocument: RepairDocument;
    readonly declaredNames: readonly string[];
    readonly declaredSubjectForms?: readonly string[];
  },
): SettledChunk {
  /**
   Issue ids the checker majority confirmed fixed.
   */
  const resolvedIssueIds = creditableIssues
    .filter(function isResolved(issue,) {
      return tallies[issue.issueId]
        ?.resolved
        === true;
    },)
    .map(function toId(issue,) {
      return issue.issueId;
    },);

  return {
    ...settleChunkVerdict({
      sliceIndex,
      declaredNames,
      ...((declaredSubjectForms === undefined) ? {} : { declaredSubjectForms, }),
      incumbentText,
      patchedText,
      measurements: measurePatchedCandidate({
        acceptedIssues: creditableIssues,
        tallies,
        resolvedTotal: resolvedIssueIds.length,
        envelopes,
        applied: appliedOperations,
        patchedDocument: parseDocument({ text: patchedText, },),
        targetDocument,
      },),
    },),
    resolvedIssueIds,
  };
}

/**
 Renders one slice's settlement as the line an operator watches a run by.

 KEPT BESIDE THE VERDICT rather than at the caller, because every number in it
 is a count of something the verdict decided, and a summary that drifts from
 the decision it summarises is worse than no summary: a run reads as healthy
 while shipping something else.

 @param sliceIndex - slice being reported

 @param changed - whether the returned wording differs from the archive's

 @param patchSelected - whether the patched candidate beat the archive, which
 tells a lost repair from one that wrote nothing

 @param refused - whether the winning patch was refused for dropping a
 declared name

 @param resolvedCount - accepted issues the checkers found resolved

 @param creditableCount - accepted issues an applied envelope actually served

 @param acceptedCount - issues the panel accepted at all

 @param unenvelopedCount - accepted issues no envelope could serve

 @returns One line naming what this slice settled on

 @internal

 @example
 ```ts
 const line = describeChunkSettlement({ sliceIndex, changed, patchSelected, refused, resolvedCount, ... },);
 ```
 */
export function describeChunkSettlement(
  {
    sliceIndex,
    changed,
    patchSelected,
    refused,
    resolvedCount,
    creditableCount,
    acceptedCount,
    unenvelopedCount,
  }: {
    readonly sliceIndex: number;
    readonly changed: boolean;
    readonly patchSelected: boolean;
    readonly refused: boolean;
    readonly resolvedCount: number;
    readonly creditableCount: number;
    readonly acceptedCount: number;
    readonly unenvelopedCount: number;
  },
): string {
  return `chunk ${String(sliceIndex,)}: ${settlementState({
    changed,
    patchSelected,
    refused,
  },)}, ${
    String(resolvedCount,)
  }/${String(creditableCount,)} served accepted ${
    wordForCount({
      count: creditableCount,
      one: 'issue',
      many: 'issues',
    },)
  } resolved (${
    String(acceptedCount,)
  } accepted, ${String(unenvelopedCount,)} unenveloped)`;
}

//endregion Repair chunk verdict
