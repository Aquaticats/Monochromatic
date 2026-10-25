import type { Logger, } from '@monochromatic-dev/module-logger/ts';
import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';

import type {
  AdjudicatedIssue,
  EditableEnvelope,
  RosterModelId,
} from '@monochromatic-dev/translation-repair-shared-contract/ts';
import {
  buildLicensedQuotes,
  buildRemovableQuotes,
} from '@monochromatic-dev/translation-repair-glossary/ts';
import type {
  PatchOutcome,
  PreservationMode,
} from '@monochromatic-dev/translation-repair-page-pass/ts';
import type { ModelClient, } from '@monochromatic-dev/translation-repair-provider/ts';
import {
  gatherStageVoices,
  type StageGather,
  type StageSettings,
} from '@monochromatic-dev/translation-repair-stage/ts';
import {
  buildEditorMessages,
  type EditorPromptPlan,
} from './edit-prompt.ts';
import {
  EDITOR_RESPONSE_FORMAT,
  type EditorReportWire,
  isEditorReportWire,
} from './edit-wire.ts';
import {
  buildEditorCandidates,
  type EditorCandidateSet,
} from './editor-candidates.ts';

//region Editor gather
// The part of the editor stage that comes before any judge: build the sheet, ask
// the editors, and pass what they said through the apply gate. Kept apart from
// `runEditorStage` so the stage reads as the sequence of decisions, and so a
// probe that seats one editor card can ask exactly what the stage asks and
// score what the gate lets through, without a judge in the call.

/**
 What asking the editors produced, before selection.

 @example
 ```ts
 const { built, gather, } = await gatherEditorProposals({ ... },);
 ```
 */
export type EditorProposals = {
  /**
   The sheet every editor was asked, with the envelope order replies resolve through.
   */
  readonly plan: EditorPromptPlan;

  /**
   Editor replies after retry-to-quorum.
   */
  readonly gather: StageGather<EditorReportWire>;

  /**
   Gate every proposal passed, the one a stage re-applying a subset of a patch must apply again.
   */
  readonly preservation: PreservationMode;

  /**
   The translation untouched, which a stage ships when nothing survived the gate.
   */
  readonly unchanged: PatchOutcome;

  /**
   One gated patch per heard editor, in roster order, with the wire findings its reply raised.
   */
  readonly built: EditorCandidateSet;
};

/**
 Asks the editors one chunk's sheet and gates what they propose, without judging.

 @param client - injected model client

 @param editorModelIds - editors proposing candidates

 @param editorRuleAddendum - extra rule line for prompt calibration

 @param sourceText - original chunk text

 @param targetText - translation chunk text the envelopes were cut from

 @param envelopes - non-overlapping envelopes in document order

 @param issues - adjudicated issues the envelopes serve

 @param neighbouringSourceText - source context also supplied to selecting judges

 @param neighbouringIncumbentText - placement context for editors

 @param documentSourceText - same-entry evidence the preservation gate reads markup against

 @param identityContext - declared names and handles, for the editors (ledger S14)

 @param referenceContext - what the pages the original cites say, for the same readers (ledger S14)

 @param signal - caller abort honored by every exchange

 @param perCallTimeoutMs - deadline per exchange

 @param l - pipeline logger

 @param stageSettings - straggler windows and contest ledger the entry file
 read for this launch (`StageSettings`)

 @returns The sheet, the heard replies, the gate and the gated candidates

 @example
 ```ts
 const { built, } = await gatherEditorProposals({ ... },);
 ```
 */
export async function gatherEditorProposals(
  {
    client,
    editorModelIds,
    editorRuleAddendum,
    sourceText,
    targetText,
    envelopes,
    issues,
    neighbouringIncumbentText,
    neighbouringSourceText,
    documentSourceText,
    identityContext,
    referenceContext,
    signal,
    perCallTimeoutMs,
    l,
    stageSettings,
  }: ForeignBorrowed<{
    readonly client: ModelClient;
    readonly editorModelIds: readonly RosterModelId[];
    readonly editorRuleAddendum?: string;
    readonly neighbouringIncumbentText?: string;
    readonly neighbouringSourceText?: string;
    readonly documentSourceText?: string;
    readonly identityContext?: string;
    readonly referenceContext?: string;
    readonly sourceText: string;
    readonly targetText: string;
    readonly envelopes: readonly EditableEnvelope[];
    readonly issues: readonly AdjudicatedIssue[];
    readonly signal: AbortSignal;
    readonly perCallTimeoutMs: number;
    readonly l: Logger;
    readonly stageSettings: StageSettings;
  }>,
): Promise<EditorProposals> {
  /**
   Declared names and references for the editors, each absent when the page
   has none (ledger S14).
   */
  const pageFragment = {
    ...((identityContext === undefined) ? {} : { identityContext, }),
    ...((referenceContext === undefined) ? {} : { referenceContext, }),
  };

  /**
   Editor sheet shared by every editor, so their candidates answer the same
   question and stay comparable.
   */
  const plan = buildEditorMessages({
    sourceText,
    targetText,
    envelopes,
    issues,
    ...pageFragment,
    ...(editorRuleAddendum === undefined ? {} : { editorRuleAddendum, }),
    ...((neighbouringSourceText === undefined) ? {} : { neighbouringSourceText, }),
    ...((neighbouringIncumbentText === undefined) ? {} : { neighbouringIncumbentText, }),
  },);

  /**
   Editor replies after retry-to-quorum.
   */
  const gather = await gatherStageVoices({
    client,
    modelIds: editorModelIds,
    messages: plan.messages,
    signal,
    exchangeTimeoutMs: perCallTimeoutMs,
    responseFormat: EDITOR_RESPONSE_FORMAT,
    validate: isEditorReportWire,
    stage: 'editor',
    // Retries stop at QUORUM, which on this three-editor roster is two voices.
    // This stage waited for every voice from 2026-08-12 until the user removed
    // that option on 2026-08-15: one model degrading for a day made every
    // editor gather spend four deadlines chasing it, and the ensemble property
    // the wait protected is already held by a quorum of two.
    //
    // A WRITER ROUND, so the third voice is a whole candidate and waits under
    // the writer window, built in since 2026-09-06 (`writer-grace-override.ts`).
    graceMs: stageSettings
      .grace
      .writerMs,
    // A launch that pins the editor bench's rotation (a fixture or a calibration that must seat the same editors whatever the
    // sheet says) hands the seat down in its settings; the prompt decides otherwise.
    ...((stageSettings.editorBenchStart === undefined) ? {} : { benchStart: stageSettings.editorBenchStart, }),
    l,
  },);

  /**
   Untouched outcome shared by both early exits.
   */
  const unchanged: PatchOutcome = {
    patchedText: targetText,
    applied: [],
    rejected: [],
  };

  /**
   Defect text each envelope's issues quoted, which the preservation gate
   treats as licensed to disappear, the addition claims' quotes, whose markup
   atoms alone an edit may remove, and the original the markup is read
   against: the whole source document where the chunk has it, since an atom
   copied from the source is copied from the page rather than the slice
   (ledger L4).
   */
  const preservation: PreservationMode = {
    mode: 'enforce',
    licensedQuotes: buildLicensedQuotes({
      envelopes,
      issues,
    },),
    removableQuotes: buildRemovableQuotes({
      envelopes,
      issues,
    },),
    sourceText: documentSourceText ?? sourceText,
  };

  /**
   One gated patch per heard editor, in roster order.
   */
  const built = buildEditorCandidates({
    voices: gather.voices,
    editorModelIds,
    promptEnvelopes: plan.envelopes,
    targetText,
    envelopes,
    preservation,
  },);

  return {
    plan,
    gather,
    preservation,
    unchanged,
    built,
  };
}

//endregion Editor gather
