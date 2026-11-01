import type { Logger, } from '@monochromatic-dev/logger-core/ts';
import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';

import type { ModelClient, } from '@monochromatic-dev/translation-repair-provider/ts';
import {
  type AuditMemberClaim,
  type CorroboratedDefect,
  corroborate,
  corroborateByOverlap,
  type OverlapAgreement,
  type NearMiss,
  nearMisses,
} from './rendering-audit-corroborate.ts';
import { buildRenderingAuditMessages, } from './rendering-audit-prompt.ts';
import {
  RENDERING_AUDIT_RESPONSE_FORMAT,
  type RenderingAuditSubject,
  isRenderingAuditReportWire,
} from './rendering-audit-wire.ts';
import {
  type ScreenedFinding,
  screenRenderingAudit,
} from './rendering-audit-screen.ts';
import {
  gatherStageVoices,
  type StageSettings,
} from '@monochromatic-dev/translation-repair-stage/ts';
import type { RosterModelId, } from '@monochromatic-dev/translation-repair-shared-contract/ts';

//region Rendering audit stage
// Asks a roster whether one rendering says what its original says, screens
// every answer against the two texts, and reports what more than one voice
// found.
//
// SHADOW MODE, like the probe it sits beside. Nothing here decides what ships.
// The instrument's own error rate is unmeasured until its two live arms run,
// and gating on an unmeasured instrument is how a false claim discards a
// correct rendering.
//
// CORROBORATION IS BY DISTINCT VOICE, on the same rule the introduced-defect
// probe uses: a defect two auditors found independently is a different fact
// from one auditor's opinion, and the roster is the only independence available
// here.
//
// WHAT COUNTS AS THE SAME DEFECT lives in `rendering-audit-corroborate.ts`,
// which compares FOCUS INTERVALS rather than quoted text, and reports overlap
// as a near miss rather than merging it.
//
// PER-VOICE ROWS ARE KEPT. Three probers were measured disagreeing by an order
// of magnitude about how often an edit is worth a claim, and the decision about
// how to read a tally over such a roster is still open. A report that kept only
// the aggregate would have to be re-run to answer it; one that keeps the rows
// can be re-read.

/**
 One auditor's screened answer, kept whole.

 @example
 ```ts
 const row: AuditVoiceRow = { modelId, verdict: 'no-defect-found', findings: [], dropped: [], };
 ```
 */
export type AuditVoiceRow = {
  /**
   Auditor that answered.
   */
  readonly modelId: RosterModelId;

  /**
   What it concluded overall.
   */
  readonly verdict: string;

  /**
   What it claimed that anchored.
   */
  readonly findings: readonly ScreenedFinding[];

  /**
   Why each of its dropped claims fell, kept because a voice whose every claim
   fell is not a voice that found nothing.
   */
  readonly dropped: readonly string[];
};

/**
 Everything one audit produced about one rendering.

 @example
 ```ts
 const report: RenderingAuditReport = { corroborated: [], near: [], rows: [], findings: [], };
 ```
 */
export type RenderingAuditReport = {
  /**
   Defects at least two auditors located identically, most-agreed first.
   */
  readonly corroborated: readonly CorroboratedDefect[];

  /**
   Groups of voices that agreed about a defect without quoting identical
   spans, reported BESIDE the strict count rather than inside it: the strict
   count asks whether voices picked the same characters, and this asks whether
   they were talking about the same thing.
   */
  readonly agreed: readonly OverlapAgreement[];

  /**
   Pairs of claims from different voices that nearly agreed, reported rather
   than merged, since a merge on overlap would manufacture agreement nobody
   reached.
   */
  readonly near: readonly NearMiss[];

  /**
   Every auditor's screened answer, kept for a later decision about how to
   read a tally over a roster that disagrees with itself. The rows stand in
   the order the roster is seated from the bench the audit's prompt rotates
   (`rotatedBench`, `stage-fanout-window.ts`), so the order moves whenever the
   sheet's wording does; a reader that names a voice by its position derives the
   order from the prompt and the roster, as the print cases of the audit
   sensitivity runner do.
   */
  readonly rows: readonly AuditVoiceRow[];

  /**
   Degradation findings from the gather, empty when quorum was met.
   */
  readonly findings: readonly string[];
};

/**
 Audits one rendering against its original.

 @param client - injected model client

 @param subject - original, candidate and any licensed identity evidence

 @param modelIds - auditor roster

 @param signal - entry deadline and caller abort

 @param perCallTimeoutMs - deadline per exchange

 @param l - pipeline logger

 @param stageSettings - straggler windows and contest ledger the entry file
 read for this launch (`StageSettings`)

 @returns Corroborated defects, every voice's screened answer, and any
 degradation findings

 @throws Whatever the gather raises when the caller aborts

 @example
 ```ts
 const report = await runRenderingAudit({ client, subject, modelIds, signal, perCallTimeoutMs, l, stageSettings, },);
 ```
 */
export async function runRenderingAudit(
  {
    client,
    subject,
    modelIds,
    signal,
    perCallTimeoutMs,
    l,
    stageSettings,
  }: ForeignBorrowed<{
    readonly client: ModelClient;
    readonly subject: RenderingAuditSubject;
    readonly modelIds: readonly RosterModelId[];
    readonly signal: AbortSignal;
    readonly perCallTimeoutMs: number;
    readonly l: Logger;
    readonly stageSettings: StageSettings;
  }>,
): Promise<RenderingAuditReport> {
  /**
   What every auditor answered, to quorum.
   */
  const gathered = await gatherStageVoices({
    client,
    modelIds,
    messages: buildRenderingAuditMessages({ subject, },),
    signal,
    exchangeTimeoutMs: perCallTimeoutMs,
    responseFormat: RENDERING_AUDIT_RESPONSE_FORMAT,
    validate: isRenderingAuditReportWire,
    stage: runRenderingAudit.name,
    l,
    graceMs: stageSettings
      .grace
      .roundMs,
  },);

  /**
   Each answer screened against the two texts.
   */
  const rows: readonly AuditVoiceRow[] = gathered.voices
    .map(function screenVoice(voice,): AuditVoiceRow {
      /**
       What survived of this voice's answer.
       */
      const screened = screenRenderingAudit({
        report: voice.value,
        sourceText: subject.sourceText,
        candidateText: subject.candidateText,
      },);

      return {
        modelId: voice.modelId,
        verdict: screened.verdict,
        findings: screened.findings,
        dropped: screened.dropped,
      };
    },);

  /**
   Every anchored claim, tagged with the voice that made it, which is the form
   both the matcher and the near-miss pass read.
   */
  const claims: readonly AuditMemberClaim[] = rows.flatMap(function toClaims(row,): readonly AuditMemberClaim[] {
    return row.findings
      .map(function toClaim(finding,): AuditMemberClaim {
        return {
          modelId: row.modelId,
          finding,
        };
      },);
  },);

  return {
    corroborated: corroborate({ claims, },),
    agreed: corroborateByOverlap({ claims, },),
    near: nearMisses({ claims, },),
    rows,
    findings: gathered.findings,
  };
}

//endregion Rendering audit stage
