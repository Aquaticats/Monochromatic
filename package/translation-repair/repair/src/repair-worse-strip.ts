import type {
  AdjudicatedIssue,
  EditableEnvelope,
  ReadonlyEntry,
} from '@monochromatic-dev/translation-repair-shared-contract/ts';
import { applyPatchOperations, } from '@monochromatic-dev/translation-repair-page-pass/ts';
import {
  type AppliedEnvelopeReading,
  readAppliedEnvelopes,
} from './chunk-measure.ts';
import {
  type ChunkProof,
  proveRepairedChunk,
} from './repair-chunk-proof.ts';
import type { EditorStageResult, } from './repair-editor-stage.ts';
import type { IssueResolutionTally, } from '@monochromatic-dev/translation-repair-adjudication/ts';

//region Worse-voted strip
// LEDGER L3, the owner's ruling of 2026-09-28 ("Revert worse-voted,
// recheck"): a selected patch carried edits whose issues the checkers did not
// confirm (345 of 2,148 patches over every run, 18 of 51 on TianqiChen666),
// and TianqiChen66616 slice 3 shipped one a checker had voted worse. An edit
// whose issue the checkers did not confirm and which drew at least one worse
// ballot is stripped (58 issues in 31 patches over every run), and the
// reduced patch faces one more checker round, and the introduced-defect
// probe, before it ships. An unconfirmed edit with no worse ballot stays.
//
// TWO BALLOTS FOR UNDO TOO (the owner's answer of 2026-10-25, "Two ballots
// for undo too", under the rule that one judge alone never decides): the ruling
// of 2026-09-28 said "at least one worse ballot" and now reads as the condition that
// sets `regressed` on the issue's tally (`tallyResolutionChecks`, the one
// definition): two or more ballots name worse and the worse weight exceeds
// the fixed and not-fixed weight together. One worse ballot strips nothing,
// whatever sits beside it. This reads the tally's flag rather than a count, so
// the strip, the rollback of a rewrite and the candidate ranking cannot drift.
//
// ONE ROUND, NEVER A LADDER, as the recovery round is: the recheck's tallies
// settle the chunk whatever they say, and nothing is stripped a second time.
//
// THE ENVELOPE IS THE UNIT. An edit replaces a whole envelope, and an
// envelope serving a worse-voted issue beside a confirmed one goes whole: the
// checker voted the text that edit wrote worse, whichever issue it answered.

/**
 Arguments of one proof, as the chunk hands them to the prover.
 */
type ProofRequest = Parameters<typeof proveRepairedChunk>[0];

/**
 What stripping worse-voted edits made of a patch.
 */
export type WorseStrip =
  | {
    /**
     No unconfirmed edit's issue regressed; the patch stands whole.
     */
    readonly stripped: false;
  }
    | {
    /**
     At least one edit was stripped.
     */
      readonly stripped: true;

    /**
     Editor result with the reduced patch in place of the whole one.
     */
      readonly editor: EditorStageResult;

    /**
     Issues whose edits were stripped.
     */
      readonly issueIds: readonly string[];
    };

/**
 Strips from a patch every envelope's edit that serves an issue the checkers
 did not confirm and flagged `regressed`: two or more checkers voted worse and
 the worse weight exceeds the fixed and not-fixed weight together.

 @param editor - editor result whose patch the checkers judged

 @param envelopes - envelopes the patch was written against

 @param creditableIssues - accepted issues an applied envelope served

 @param tallies - checkers' verdicts by issue id

 @param targetText - archive wording the patch applies to

 @returns Patch as it stands, or reduced with the stripped issues named

 @example
 ```ts
 const strip = stripWorseVotedEdits({ editor, envelopes, creditableIssues, tallies, targetText, },);
 ```
 */
export function stripWorseVotedEdits(
  {
    editor,
    envelopes,
    creditableIssues,
    tallies,
    targetText,
  }: {
    readonly editor: EditorStageResult;
    readonly envelopes: readonly EditableEnvelope[];
    readonly creditableIssues: readonly AdjudicatedIssue[];
    readonly tallies: Readonly<Record<string, IssueResolutionTally>>;
    readonly targetText: string;
  },
): WorseStrip {
  /**
   Issues an applied envelope served, the only ones whose edits can go.
   */
  const creditableIds = new Set(creditableIssues.map(function toId(issue,): string {
    return issue.issueId;
  },),);
  /**
   Issues the checkers did not confirm and two or more voted worse, the
   weight of those ballots exceeding the rest (the tally's `regressed`).

   READ OFF THE TALLIES, since a worse ballot is only ever counted in one: an
   issue without a tally has no worse vote to strip. The checker stage keeps a
   tally for every issue it was given (`tallyResolutionChecks`), one no checker
   voted on included, so looking each creditable issue up would guard a miss no
   run can produce.
   */
  const worseIssueIds = new Set(Object.entries(tallies,)
    .filter(function isWorseVoted([issueId, tally,]: ReadonlyEntry<string, IssueResolutionTally>,): boolean {
      if (!creditableIds.has(issueId,))
        return false;
      // `regressed` excludes `resolved` (worse outweighing fixed plus the rest cannot
      // sit with fixed outweighing worse plus the rest), so it alone is the condition.
      return tally.regressed;
    },)
    .map(function toId([issueId,]: ReadonlyEntry<string, IssueResolutionTally>,): string {
      return issueId;
    },),);
  if (worseIssueIds.size === 0)
    return { stripped: false, };
  /**
   Envelopes serving one of those issues, whose edits go.
   */
  const strippedEnvelopeIds = new Set(envelopes
    .filter(function servesWorseVoted(envelope,): boolean {
      return envelope.issueIds
        .some(function isWorseVoted(issueId,): boolean {
          return worseIssueIds.has(issueId,);
        },);
    },)
    .map(function toId(envelope,): string {
      return envelope.envelopeId;
    },),);
  /**
   Applied edits that stay.
   */
  const kept = editor.patch
    .applied
    .filter(function stays(operation,): boolean {
      return !strippedEnvelopeIds.has(operation.envelopeId,);
    },);
  /**
   The reduced patch, UNDER THE GATE THE WHOLE PATCH PASSED. Every per-edit
   rule passes again, since envelopes never overlap and each edit is judged on
   its own envelope; the markup rule may not, since an edit whose lost
   markup a stripped sibling wrote has lost it once the sibling goes (ledger
   L4).
   */
  const reduced = applyPatchOperations({
    targetText,
    envelopes,
    operations: kept,
    preservation: editor.preservation,
  },);
  return {
    stripped: true,
    editor: {
      ...editor,
      patch: {
        ...reduced,
        // The gate's refusals of the whole patch stay on the record, with any
        // the reduced patch added; a stripped edit is not a refusal.
        rejected: [
          ...editor.patch
            .rejected,
          ...reduced.rejected,
        ],
      },
    },
    issueIds: [...worseIssueIds,],
  };
}

/**
 What the chunk ships on after the proof and any strip.
 */
export type ShedProof = {
  /**
   Editor result as it ships: the whole patch, or the reduced one.
   */
  readonly editor: EditorStageResult;

  /**
   What the shipped patch's envelopes bought.
   */
  readonly appliedEnvelopes: AppliedEnvelopeReading;

  /**
   Proof of the shipped patch: the first round's, or the recheck's.
   */
  readonly proof: ChunkProof;

  /**
   Findings the strip adds: the stripped issues and the first round's
   checker findings, none when nothing was stripped.
   */
  readonly findings: readonly string[];
};

/**
 Proves the patch, strips its worse-voted unconfirmed edits, and proves the
 reduced patch once more when any was stripped and any edit remains.

 @param request - proof as the chunk would ask it for the whole patch

 @returns Editor result, envelope reading and proof the chunk settles on

 @example
 ```ts
 const shed = await proveSheddingWorseVoted({ client, models, reseat, sourceText, targetText, envelopes, editor, acceptedIssues, authorship, neighbours, identityContext, signal, perCallTimeoutMs, l, },);
 ```
 */
export async function proveSheddingWorseVoted(request: ProofRequest,): Promise<ShedProof> {
  /**
   What the strip reads off the request.
   */
  const {
    acceptedIssues,
    envelopes,
    editor,
    targetText,
    l,
  } = request;
  /**
   Proof of the whole patch.
   */
  const first = await proveRepairedChunk(request,);
  /**
   What the whole patch's envelopes bought.
   */
  const whole = readAppliedEnvelopes({
    acceptedIssues,
    envelopes,
    editor,
  },);
  /**
   The patch with its worse-voted unconfirmed edits stripped, if any.
   */
  const strip = stripWorseVotedEdits({
    editor,
    envelopes,
    creditableIssues: whole.creditableIssues,
    tallies: first.checker
      .tallies,
    targetText,
  },);
  if (!strip.stripped) {
    return {
      editor,
      appliedEnvelopes: whole,
      proof: first,
      findings: [],
    };
  }
  /**
   What the strip records, naming the issues rather than the slice, since
   the record is stored and re-stamped by position.
   */
  const strippedList = strip.issueIds
    .join(', ',);
  /**
   The line itself.
   */
  const finding = `repair-stripped-worse-voted (issues ${strippedList}: `
    + 'unconfirmed by the checkers, with two or more worse ballots outweighing the rest)';
  l.warn(finding,);
  /**
   What the reduced patch's envelopes bought.
   */
  const reducedReading = readAppliedEnvelopes({
    acceptedIssues,
    envelopes,
    editor: strip.editor,
  },);
  if (strip.editor
    .patch
    .applied
    .length
    === 0) {
    l.info('no edit of the patch survived the strip and its gate, so the chunk keeps its standing with no recheck',);
    return {
      editor: strip.editor,
      appliedEnvelopes: reducedReading,
      proof: first,
      findings: [
        finding,
        ...first.checker
          .findings,
      ],
    };
  }
  /**
   Proof of the reduced patch: the one more checker round the ruling asks.
   */
  const recheck = await proveRepairedChunk({
    ...request,
    editor: strip.editor,
    authorship: reducedReading.authorship,
  },);
  return {
    editor: strip.editor,
    appliedEnvelopes: reducedReading,
    proof: recheck,
    findings: [
      finding,
      ...first.checker
        .findings,
    ],
  };
}

//endregion Worse-voted strip
