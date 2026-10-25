import { textsInCodePointOrder, } from '@monochromatic-dev/translation-repair-markup/ts';
import type { Logger, } from '@monochromatic-dev/module-logger/ts';
import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';

import type {
  AdjudicatedIssue,
  EditableEnvelope,
  ReadonlyEntry,
  RosterModelId,
} from '@monochromatic-dev/translation-repair-shared-contract/ts';
import { wordForCount, } from '@monochromatic-dev/translation-repair-typography/ts';
import {
  buildLicensedQuotes,
  buildRemovableQuotes,
} from '@monochromatic-dev/translation-repair-glossary/ts';
import type {
  PatchOutcome,
  PreservationMode,
} from '@monochromatic-dev/translation-repair-page-pass/ts';
import {
  NOBODY_WROTE_IT,
  type ShippedProducer,
} from './editor-selection-result.ts';
import type { ModelClient, } from '@monochromatic-dev/translation-repair-provider/ts';
import { buildEditorMessages, } from './edit-prompt.ts';
import {
  EDITOR_RESPONSE_FORMAT,
  isEditorReportWire,
} from './edit-wire.ts';
import {
  buildChunkCandidates,
  buildEditorCandidates,
  chunkCandidateOf,
  pickFallbackCandidate,
} from './editor-candidates.ts';
import {
  applyCandidate,
  selectChunkPatch,
  selectPerEnvelope,
} from './editor-ensemble.ts';
import {
  assertJudgeableEditorRoster,
  type ProducerRosterError,
  type RepairJudgedRound,
} from '@monochromatic-dev/translation-repair-assembly/ts';
import {
  gatherStageVoices,
  type StageSettings,
} from '@monochromatic-dev/translation-repair-stage/ts';

//region Editor stage
// Several editors rewrite one chunk's envelopes independently, every proposal
// passes the same deterministic apply gate, and judges choose what ships. An
// editor may sit among the judges, and its ballot for its own candidate counts
// at the self-vote weight (the user ruling of 2026-08-15). This is the stage
// that used to be one model deciding alone.
//
// The stage never returns the untouched translation just because judging failed
// to converge: a round the judges cannot decide ships the fallback repair. It
// returns the untouched translation when every judge declines every candidate
// (ledger L14(c)), and when no editor produced an operation that survived the
// gate, which is the condition the single-editor stage exited on.

/**
 Everything the editor stage produced for one chunk.

 @example
 ```ts
 const { patch, heardEditors, } = await runEditorStage({ ... },);
 ```
 */
export type EditorStageResult = {
  /**
   Apply-gate outcome of the winning candidate;
   unchanged text when every editor voice was lost or every operation was
   refused.
   */
  readonly patch: PatchOutcome;

  /**
   Editors whose reply arrived and validated.
   */
  readonly heardEditors: number;

  /**
   Judged rounds from both selection passes, per-envelope first and the
   whole-chunk round last, in the order they ran.
   */
  readonly rounds: readonly RepairJudgedRound[];

  /**
   Wire irregularities and selection telemetry in scorecard-stable wording.
   */
  readonly findings: readonly string[];

  /**
   Who wrote {@link EditorStageResult.patch}, absent when the untouched
   translation ships.

   The one authority on authorship for this stage. Reading it off the rounds
   instead misses the editor whose patch ships when the whole-chunk judges
   decline, and credits envelope winners whose text lost to a rival
   whole-chunk proposal.
   */
  readonly shippedProducer: ShippedProducer;

  /**
   Gate the patch passed, so a stage re-applying a subset of it (the
   worse-voted strip, ledger L3) applies the same gate. A SUBSET OF A GATED
   PATCH IS NOT GATED BY CONSTRUCTION: an edit whose lost markup a sibling
   edit wrote stands only while that sibling does (ledger L4).
   */
  readonly preservation: PreservationMode;
};

/**
 Runs the editor ensemble over one chunk and returns the patch that ships.

 @param client - injected model client

 @param editorModelIds - editors proposing candidates

 @param judgeModelIds - whole roster selection draws judges from

 @param editorRuleAddendum - extra rule line for prompt calibration

 @param sourceText - original chunk text

 @param targetText - translation chunk text the envelopes were cut from

 @param envelopes - non-overlapping envelopes in document order

 @param issues - adjudicated issues the envelopes serve

 @param neighbouringSourceText - source context also supplied to selecting judges

 @param neighbouringIncumbentText - placement context for editors

 @param documentSourceText - same-entry evidence forwarded to selectors, not an editor coverage expansion

 @param identityContext - declared names and handles, for the editors and both selections (ledger S14)

 @param referenceContext - what the pages the original cites say, for the same readers (ledger S14)

 @param signal - caller abort honored by every exchange

 @param perCallTimeoutMs - deadline per exchange

 @param l - pipeline logger

 @param stageSettings - straggler windows and contest ledger the entry file
 read for this launch (`StageSettings`)

 @returns Winning patch plus findings

 @throws {@link ProducerRosterError} when either
 roster repeats a model, no editor is seated, or fewer than two judges are
 seated, so no winner could ever be named by two ballots

 @example
 ```ts
 const editor = await runEditorStage({ ... },);
 ```
 */
export async function runEditorStage(
  {
    client,
    editorModelIds,
    judgeModelIds,
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
    readonly judgeModelIds: readonly RosterModelId[];
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
): Promise<EditorStageResult> {
  assertJudgeableEditorRoster({
    editorModelIds,
    judgeModelIds,
  },);

  /**
   Declared names and references for the editors and both selections, each
   absent when the page has none (ledger S14).
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

  /**
   Candidates that actually repair something; one that landed no operation is
   the untouched translation, which competes downstream anyway.
   */
  const repairing = built.candidates
    .filter(function landedWork(candidate,) {
      return candidate.patch
        .applied
        .length
        > 0;
    },);

  /**
   Findings shared by every exit after the fan-out.
   */
  const stageFindings = [
    ...gather.findings,
    ...built.findings,
    `editor-candidates (${String(gather.voices
      .length,)}/${String(editorModelIds.length,)} heard, ${
      String(repairing.length,)
    } repairing)`,
  ];
  if (repairing.length === 0) {
    l.info('editor stage: no operation survived the gate',);
    return {
      patch: unchanged,
      heardEditors: gather.voices
        .length,
      rounds: [],
      findings: stageFindings,
      // No operation survived the gate, so the untouched translation ships.
      shippedProducer: NOBODY_WROTE_IT,
      preservation,
    };
  }

  /**
   Per-envelope winners assembled into one operation set.
   */
  const perEnvelope = await selectPerEnvelope({
    client,
    candidates: repairing,
    envelopes,
    judgeModelIds,
    sourceText,
    targetText,
    ...((neighbouringSourceText === undefined) ? {} : { neighbouringSourceText, }),
    ...((documentSourceText === undefined) ? {} : { documentSourceText, }),
    ...pageFragment,
    signal,
    perCallTimeoutMs,
    l,
    stageSettings,
  },);

  /**
   Composite candidate, scored through the same gate as a model's own.
   */
  const composite = applyCandidate({
    targetText,
    envelopes,
    operations: perEnvelope.operations,
    preservation,
  },);

  /**
   Distinct whole-chunk proposals for the judges.
   */
  const chunkSet = buildChunkCandidates({
    candidates: repairing,
    composite,
    contributors: perEnvelope.contributors,
  },);

  /**
   Winning patch, or the strongest editor patch when judges declined,
   alongside the findings from judging it.
   */
  const chunkSelection = await selectChunkPatch({
    ...((neighbouringSourceText === undefined) ? {} : { neighbouringSourceText, }),
    ...((documentSourceText === undefined) ? {} : { documentSourceText, }),
    ...pageFragment,
    client,
    candidates: chunkSet.candidates,
    judgeModelIds,
    sourceText,
    indecisionFallback: chunkCandidateOf(pickFallbackCandidate({ candidates: repairing, },),),
    rejectionFallback: unchanged,
    signal,
    perCallTimeoutMs,
    l,
    stageSettings,
  },);

  /**
   Patch that ships.
   */
  const { patch, } = chunkSelection;

  /**
   Distinct whole-chunk proposals the judges chose among.
   */
  const candidateCount = chunkSet.candidates
    .length;
  l.info(
    `editor stage: ${String(patch.applied
      .length,)} applied, ${
      String(patch.rejected
        .length,)
    } rejected across ${String(candidateCount,)} distinct ${
      wordForCount({
        count: candidateCount,
        one: 'candidate',
        many: 'candidates',
      },)
    }`,
  );

  return {
    patch,
    shippedProducer: chunkSelection.shippedProducer,
    preservation,
    heardEditors: gather.voices
      .length,
    rounds: [
      ...perEnvelope.rounds,
      ...chunkSelection.rounds,
    ],
    findings: [
      ...stageFindings,
      // Judge fan-out losses from both selection passes. These were built and
      // discarded until 2026-08-13: the selection path had no findings channel
      // at all, so a judge going silent left no trace anywhere durable.
      ...perEnvelope.findings,
      ...chunkSelection.findings,
      `editor-envelope-select (${String(perEnvelope.soleCount,)} sole, ${
        String(perEnvelope.judgedCount,)
      } judged, ${String(perEnvelope.declinedCount,)} declined)`,
      `editor-chunk-select (${String(candidateCount,)} distinct, ${String(chunkSet.collapsed,)} collapsed)`,
      // WHY an operation was refused, not merely how many were. The count went
      // into a log line and the reasons went nowhere, so the preservation gate
      // could reject every edit in a run and the artifact would look ordinary.
      // Counted by reason rather than listed, since one bad editor can produce
      // many refusals of one kind and the shape is what a reader needs. Sorted
      // as finished lines: they share one prefix and each kind appears once, so
      // their code-point order is the kinds' order (ledger B122).
      ...textsInCodePointOrder({ texts: [...patch.rejected
        .reduce(
        function tally(
          counts: Map<string, number>,
          rejection,
        ) {
        /**
         Where the reason's parenthetical detail begins, when it has one.
         */
          const detailAt = rejection.reason
            .indexOf(' (',);
        /**
         Reason with its parenthetical detail stripped, so counts group.
         */
          const kind = (detailAt === (-1))
            ? rejection.reason
            : rejection.reason
              .slice(
              0,
              detailAt,
              );
          return counts.set(
            kind,
            (counts.get(kind,) ?? 0) + 1,
          );
        },
        new Map<string, number>(),
        ),]
        .map(function toFinding([kind, count,]: ReadonlyEntry<string, number>,): string {
          return `editor-rejected ${kind} (${String(count,)})`;
        },), },),
    ],
  };
}

//endregion Editor stage
