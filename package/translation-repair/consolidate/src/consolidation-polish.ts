import type { Logger, } from '@monochromatic-dev/logger-core/ts';
import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';

import { confirmAbsoluteNaturalness, } from '@monochromatic-dev/translation-repair-archive/ts';
import type { ModelClient, } from '@monochromatic-dev/translation-repair-provider/ts';
import type { SliceSyntax, } from '@monochromatic-dev/translation-repair-slice-reading/ts';
import { describeReviewFindings, } from './consolidation-naturalness-state.ts';
import type {
  ConsolidationPolish,
  ConsolidationPolishConfig,
} from './consolidation-polish-model.ts';
import {
  reviewParagraphsOf,
  runConsolidationPolishRound,
} from './consolidation-polish-round.ts';
import { unpolishedBaseline, } from './consolidation-polish-skip.ts';
import type { DisputedWording, } from '@monochromatic-dev/translation-repair-translate-floor/ts';
import type { RefineStageMode, } from '@monochromatic-dev/translation-repair-repair/ts';
import type { StageSettings, } from '@monochromatic-dev/translation-repair-stage/ts';

export type {
  ConsolidationNaturalnessAudit,
  ConsolidationNaturalnessCorrectionAudit,
  ConsolidationPolish,
  ConsolidationPolishConfig,
} from './consolidation-polish-model.ts';

//region Consolidation naturalness polish
// ONE FIXED POLISH ROUND BY DESIGN: refiners propose, judges select between
// proposal and standing text, the deterministic gate applies, and the
// absolute review that follows is recorded evidence, never withholding
// authority. There is no correction loop and no re-ask under any verdict
// (doc/planning/translation-repair-no-loop-design.md). An objection
// correction (owner, 2026-09-27) is that same one round, asked a different
// question.

/**
 Finding naming an objection correction and whether it changed the text.

 @param mode - mode the round ran in

 @param corrected - whether the round replaced the base

 @returns One finding on an objection correction, none otherwise

 @example
 ```ts
 objectionPolishFindings({ mode, corrected: true, },);
 ```
 */
function objectionPolishFindings(
  {
    mode,
    corrected,
  }: {
    readonly mode: RefineStageMode;
    readonly corrected: boolean;
  },
): readonly string[] {
  if (mode.kind !== 'objection-correction')
    return [];
  /**
   Each set of judges with how many objections it raised.
   */
  const counted = mode.groups
    .map(function countOf(group,): string {
      /**
       Judges and what they objected with.
       */
      const {
        origin,
        objections,
      } = group;
      return `${origin}: ${String(objections.length,)} objection(s)`;
    },)
    .join(', ',);
  return [`polish-objection-correction (${counted}, ${corrected ? 'corrected' : 'base kept'})`,];
}

/**
 Polishes final body text and lets fidelity-first roster approve replacement.

 @param client - shared provider client

 @param sourceText - original passage anchoring fidelity

 @param archiveText - archive wording shown as supporting evidence

 @param baseText - wording already approved by consolidation gate

 @param syntax - explicit syntax role; front matter is never polished

 @param lineStructured - whether source line boundaries must survive

 @param identityContext - names and handles prompts preserve

 @param referenceContext - what the pages the original cites say, with
 their rule, when the original cites any (class forty-one)

 @param archiveDisputeNote - accepted claims against the archive rendering
 on a disputed slice, for the gate (ledger S12), the writers and the judges of
 their rewrites (ledger B805)

 @param acceptedRepairClaimsNote - every claim the repair panel accepted for
 the slice, disputing or not, for the writers, the judges of their rewrites and
 the gate (ledger B805)

 @param disputedWordings - wordings the slice refuses, which the polish may
 not land on any more than a proposal may (ledger B29)

 @param sliceIndex - prepared slice position

 @param config - model roles and document-wide guard facts

 @param eligible - whether approved base may cross publication boundary

 @param mode - comparative polish, or an objection correction carrying what
 the gate or slate judges held against the base (owner, 2026-09-27); the
 base stays the fallback either way

 @param signal - caller cancellation

 @param perCallTimeoutMs - per-exchange ceiling

 @param l - stage logger

 @param stageSettings - straggler windows and contest ledger the entry file
 read for this launch (`StageSettings`)

 @returns Auditable polish decision and final text

 @example
 ```ts
 const polish = await polishConsolidation({ client, sourceText, archiveText, baseText, lineStructured: false, sliceIndex: 0, config, signal, perCallTimeoutMs, l, stageSettings, });
 ```
 */
export async function polishConsolidation(
  {
    client,
    sourceText,
    archiveText,
    baseText,
    syntax,
    lineStructured,
    identityContext,
    referenceContext,
    archiveDisputeNote,
    acceptedRepairClaimsNote,
    disputedWordings,
    sliceIndex,
    config,
    eligible = true,
    mode = { kind: 'comparative', },
    signal,
    perCallTimeoutMs,
    l,
    stageSettings,
  }: ForeignBorrowed<{
    readonly client: ModelClient;
    readonly sourceText: string;
    readonly archiveText: string;
    readonly baseText: string;
    readonly syntax?: SliceSyntax;
    readonly lineStructured: boolean;
    readonly identityContext?: string;
    readonly referenceContext?: string;
    readonly archiveDisputeNote?: string;
    readonly acceptedRepairClaimsNote?: string;
    readonly disputedWordings?: readonly DisputedWording[];
    readonly sliceIndex: number;
    readonly config?: ConsolidationPolishConfig;
    readonly eligible?: boolean;
    readonly mode?: RefineStageMode;
    readonly signal: AbortSignal;
    readonly perCallTimeoutMs: number;
    readonly l: Logger;
    readonly stageSettings: StageSettings;
  }>,
): Promise<ConsolidationPolish> {
  if ((syntax === 'front-matter') || (!eligible))
    return unpolishedBaseline((syntax === undefined) ? {} : { syntax, },);
  if (config === undefined) {
    return {
      kind: 'not-run',
      reason: 'not-configured',
    };
  }
  /**
   Initial exploratory generation, selection and comparative fidelity gate.
   */
  const initial = await runConsolidationPolishRound({
    client,
    sourceText,
    archiveText,
    baseText,
    // NO SYNTAX ROLE: front matter, the only one, took the `unpolishedBaseline`
    // return, so the round only ever sees body prose.
    lineStructured,
    ...((identityContext === undefined) ? {} : { identityContext, }),
    ...((referenceContext === undefined) ? {} : { referenceContext, }),
    ...((archiveDisputeNote === undefined) ? {} : { archiveDisputeNote, }),
    ...((acceptedRepairClaimsNote === undefined) ? {} : { acceptedRepairClaimsNote, }),
    ...((disputedWordings === undefined) ? {} : { disputedWordings, }),
    mode,
    sliceIndex,
    config,
    signal,
    perCallTimeoutMs,
    l,
    stageSettings,
  },);
  /**
   Independent absolute review of exact initial would-ship text.
   */
  const initialConfirmed = await confirmAbsoluteNaturalness({
    client,
    modelIds: config.gateModelIds,
    subject: {
      sourceText,
      candidateText: initial.text,
      // EVERY BODY BLOCK (artifact generation ten), so a reviewer can locate a
      // finding in a blockquote or a list as well as in a paragraph the polish
      // could have edited; the artifact reader recomputes the same set.
      paragraphs: reviewParagraphsOf({ text: initial.text, }),
      lineStructured,
      ...((identityContext === undefined) ? {} : { identityContext, }),
    },
    signal,
    exchangeTimeoutMs: perCallTimeoutMs,
    l,
    stageSettings,
  },);
  /**
   Decisive review after optional acceptance confirmation, recorded whatever
   it says: a rejection or an unheard review roster becomes located findings
   on the settlement while the gated round-one text ships.
   */
  const { review: finalReview, } = initialConfirmed;
  /**
   Evidence the non-accepting verdicts add to the settlement.
   */
  const reviewEvidence = finalReview.verdict === 'acceptable'
    ? []
    : [
      ...describeReviewFindings({ review: finalReview, },),
      finalReview.verdict === 'quorum-not-met'
        ? 'absolute naturalness review quorum not met; recorded as evidence'
        : 'absolute naturalness rejection recorded as evidence; the gated text ships',
    ];
  return {
    kind: 'settled',
    baseText,
    proposedText: initial.proposedText,
    text: initial.text,
    changed: initial.text !== baseText,
    refinersHeard: initial.refinersHeard,
    contributors: initial.contributors,
    rounds: initial.rounds,
    ...((initial.gate === undefined) ? {} : { gate: initial.gate, }),
    review: {
      correctionCount: 0,
      corrections: [],
      rounds: [finalReview,],
      confirmations: initialConfirmed.confirmations,
    },
    findings: [
      ...objectionPolishFindings({
        mode,
        corrected: initial.text !== baseText,
      },),
      ...initial.findings,
      ...reviewEvidence,
    ],
  };
}

//endregion Consolidation naturalness polish
