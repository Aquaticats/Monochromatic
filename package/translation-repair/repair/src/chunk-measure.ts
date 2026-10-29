import type { PatchOperation, } from '@monochromatic-dev/translation-repair-page-pass/ts';
import {
  appliedIssuesByEnvelope,
  collectIssueAuthors,
} from './issue-authors.ts';
import type { EditorStageResult, } from './repair-editor-stage.ts';
import type { FootnoteReferenceHit, } from '@monochromatic-dev/translation-repair-document-structure/ts';
import type { IssueAuthorship, } from '@monochromatic-dev/translation-repair-house-rule/ts';
import { downgradeCount, } from '@monochromatic-dev/translation-repair-stage/ts';
import {
  type AdjudicatedIssue,
  type EditableEnvelope,
  footnoteBreakCount,
  type RepairDocument,
  restoredFootnoteReferences,
} from '@monochromatic-dev/translation-repair-shared-contract/ts';
import type {
  CandidateMeasurements,
  IssueResolutionTally,
} from '@monochromatic-dev/translation-repair-adjudication/ts';

//region Chunk candidate measurement
// Everything deterministic that selection ranks a patched chunk by, gathered in
// one place so the chunk runner stays a sequence of stages rather than a stage
// with arithmetic in it.
//
// Two of these measure less than their original names promised, and both now
// carry names that say what they actually count. `regressedKnownIssues` can only
// see EXISTING accepted issues the checkers marked worse, so a wholly new defect
// the patch introduces has nowhere to be counted; `touchedRegionChars` sums each
// touched envelope's longer side rather than differing characters, which a
// merged envelope serving several issues makes broad. The BEHAVIOR is unchanged
// on purpose: renaming is honest, while changing what selection ranks by is a
// pipeline decision that needs its own evidence.

/**
 Accepted issues an applied operation actually served.

 Selection credit is limited to these. Checkers are asked about EVERY accepted
 issue, including ones no envelope could be cut for and ones whose envelope
 received no surviving operation, and a checker reading the patched text can
 call such an issue fixed. Counting that let a patch touching issue A beat
 unchanged on credit for issue B that nothing touched, which is not evidence
 the patch improved anything. Verdicts on unserved issues remain in the
 tallies as telemetry; they simply stop deciding the selection.

 @param acceptedIssues - every accepted issue of the chunk

 @param envelopes - envelopes operations were written against

 @param applied - operations that survived the apply gate

 @returns Accepted issues an applied operation served, in issue order

 @example
 ```ts
 const creditable = selectCreditableIssues({ acceptedIssues, envelopes, applied, },);
 ```
 */
export function selectCreditableIssues(
  {
    acceptedIssues,
    envelopes,
    applied,
  }: {
    readonly acceptedIssues: readonly AdjudicatedIssue[];
    readonly envelopes: readonly EditableEnvelope[];
    readonly applied: readonly PatchOperation[];
  },
): readonly AdjudicatedIssue[] {
  /**
   Issues named by the envelope of some applied operation.
   */
  const served = new Set(
    Object.values(appliedIssuesByEnvelope({
      envelopes,
      applied,
    },),)
      .flat(),
  );
  return acceptedIssues.filter(function wasServed(issue,) {
    return served.has(issue.issueId,);
  },);
}

/**
 Both answers one walk over the applied operations yields.

 @example
 ```ts
 const reading: AppliedEnvelopeReading = { creditableIssues, authorship, };
 ```
 */
export type AppliedEnvelopeReading = {
  /**
   Accepted issues an applied operation served, in issue order.
   */
  readonly creditableIssues: readonly AdjudicatedIssue[];

  /**
   Models that wrote the text those operations put into the candidate.
   */
  readonly authorship: IssueAuthorship;
};

/**
 What the applied envelopes bought.

 ONE READING RATHER THAN TWO CALLS. Both answers come off the same walk over
 the operations that survived the gate, and taking them separately lets the
 issues a candidate is credited for drift from the authors it is discounted
 for.

 @param acceptedIssues - accepted issues the checkers examined

 @param envelopes - editable envelopes offered to the editors

 @param editor - editor stage result, carrying its rounds and its gate

 @returns Creditable issues beside authorship of the patched text

 @example
 ```ts
 const reading = readAppliedEnvelopes({ acceptedIssues, envelopes, editor, },);
 ```
 */
export function readAppliedEnvelopes(
  {
    acceptedIssues,
    envelopes,
    editor,
  }: {
    readonly acceptedIssues: readonly AdjudicatedIssue[];
    readonly envelopes: readonly EditableEnvelope[];
    readonly editor: EditorStageResult;
  },
): AppliedEnvelopeReading {
  return {
    creditableIssues: selectCreditableIssues({
      acceptedIssues,
      envelopes,
      applied: editor.patch
        .applied,
    },),
    authorship: collectIssueAuthors({
      editor,
      envelopes,
    },),
  };
}

/**
 Measures the patched candidate against the unchanged translation.

 @param acceptedIssues - accepted issues the checkers examined

 @param tallies - checker verdicts keyed by issue id

 @param resolvedTotal - count of issues the checkers confirmed fixed

 @param envelopes - envelopes operations were written against

 @param applied - operations that survived the apply gate

 @param patchedDocument - parsed patched candidate

 @param targetDocument - parsed unchanged translation

 @param sourceReferences - footnote references of the slice's own source;
 an unresolved reference of the patched chunk that one of these names and
 the translation lacked is a restoration and leaves the footnote count.
 Absent means none, so nothing is excused

 @returns Measurements selection ranks by

 @throws Error when an applied operation names no envelope here, which no
 patch applied over these envelopes holds

 @example
 ```ts
 const measurements = measurePatchedCandidate({ ... },);
 ```
 */
export function measurePatchedCandidate(
  {
    acceptedIssues,
    tallies,
    resolvedTotal,
    envelopes,
    applied,
    patchedDocument,
    targetDocument,
    sourceReferences = [],
  }: {
    readonly acceptedIssues: readonly AdjudicatedIssue[];
    readonly tallies: Readonly<Record<string, IssueResolutionTally>>;
    readonly resolvedTotal: number;
    readonly envelopes: readonly EditableEnvelope[];
    readonly applied: readonly PatchOperation[];
    readonly patchedDocument: RepairDocument;
    readonly targetDocument: RepairDocument;
    readonly sourceReferences?: readonly FootnoteReferenceHit[];
  },
): CandidateMeasurements {
  /**
   Whether the patch left document grammar no worse than it found it.
   */
  const grammarOk = downgradeCount({ document: patchedDocument, },)
    <= downgradeCount({ document: targetDocument, },);

  /**
   Unresolved references in the patched chunk that restore one the slice's
   source carries and the translation had dropped.
   */
  const restoredCount = restoredFootnoteReferences({
    patchedDocument,
    targetDocument,
    sourceReferences,
  },)
    .length;

  /**
   Footnote findings the patched chunk carries once its restorations leave the count.
   */
  const patchedBreakCount = footnoteBreakCount({ document: patchedDocument, },) - restoredCount;

  /**
   Whether the patch left footnote structure no worse than it found it.

   A broken footnote leaves the grammar perfectly valid, so the downgrade
   signal cannot see it, and four settled repairs shipped footnote damage
   because nothing else asked. Comparison rather than an absolute count: an
   input translation is free to arrive with dangling references, and one does.

   Chunk-scoped like every other measurement here, so it sees damage a patch
   does WITHIN one chunk. A definition deleted in one chunk whose reference
   lives in another still passes, since neither chunk's own count rises.

   The same scoping would punish a RESTORATION: a reference the translation
   dropped and the patch put back has its definition in another chunk, so
   inside this one it reads as a new unresolved reference (run XingZ6022
   slice 44, `[^3]`, definition in slice 115). The owner ruled on 2026-10-29
   that a reference the slice's own source also carries is a restoration, so
   those leave the patched count and every other finding stays in it. A
   reference the source lacks, an orphan definition and a duplicated
   definition still raise it, and the page-level guard (`page-check`
   footnote resolution, the `assembly` integrity guard) still catches a
   reference with no definition anywhere on the page.
   */
  const footnotesOk = patchedBreakCount <= footnoteBreakCount({ document: targetDocument, },);

  return {
    integrityOk: grammarOk && footnotesOk,
    resolvedHighSeverity: acceptedIssues.filter(function isResolvedHigh(issue,) {
      if (tallies[issue.issueId]
        ?.resolved
        !== true)
        return false;
      return (issue.severity === 'major') || (issue.severity === 'critical');
    },)
      .length,
    resolvedTotal,
    regressedKnownIssues: acceptedIssues.filter(function isRegressed(issue,) {
      return tallies[issue.issueId]
        ?.regressed
        === true;
    },)
      .length,
    touchedRegionChars: applied.reduce(
      function addChange(
        sum,
        operation,
      ): number {
        /**
         Envelope of this operation, for its base length. Always found: an
         applied operation passed `applyPatchOperations` over these same
         envelopes, which refuses one naming no envelope (`unknown-envelope`)
         before applying anything.
         */
        const envelope = envelopes.find(function matches(candidate,) {
          return candidate.envelopeId === operation.envelopeId;
        },);
        if (envelope === undefined) {
          throw new Error(
            `unreachable: applied operation names envelope ${operation.envelopeId}, which is not among the `
              + 'envelopes the patch was applied over',
          );
        }
        return sum + Math.max(
          envelope.baseText
            .length,
          operation.newText
            .length,
        );
      },
      0,
    ),
  };
}

/**
 Accepted issues the checkers confirmed fixed ON THE CANDIDATE, whether or not
 that candidate won.

 Kept apart from what a slice REPORTS as resolved, which is gated on the
 candidate shipping. A patched candidate that loses selection still produced
 checker verdicts, and discarding them would leave every rejected repair
 looking like one nobody examined.

 @param acceptedIssues - issues the panel accepted for this slice

 @param tallies - checker verdicts keyed by issue id

 @returns Ids the checkers confirmed, in the order the issues appear

 @example
 ```ts
 const confirmed = candidateConfirmedIssueIds({ acceptedIssues, tallies, },);
 ```
 */
export function candidateConfirmedIssueIds(
  {
    acceptedIssues,
    tallies,
  }: {
    readonly acceptedIssues: readonly AdjudicatedIssue[];
    readonly tallies: Readonly<Record<string, IssueResolutionTally>>;
  },
): readonly string[] {
  return acceptedIssues
    .filter(function confirmedOnCandidate(issue,): boolean {
      return tallies[issue.issueId]
        ?.resolved
        === true;
    },)
    .map(function toId(issue,): string {
      return issue.issueId;
    },);
}

//endregion Chunk candidate measurement
