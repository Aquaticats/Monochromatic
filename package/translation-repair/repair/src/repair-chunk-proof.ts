import type { Logger, } from '@monochromatic-dev/logger-core/ts';
import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';

import type {
  AdjudicatedIssue,
  EditableEnvelope,
} from '@monochromatic-dev/translation-repair-shared-contract/ts';
import type { ModelClient, } from '@monochromatic-dev/translation-repair-provider/ts';
import {
  collectRepairRegions,
  PRODUCTION_PRIOR_ISSUE_DISCLOSURE,
  runIntroducedDefectProbe,
} from '@monochromatic-dev/translation-repair-adjudication/ts';
import { checkerBenchAtStage, } from './repair-checker-reseat.ts';
import type { NeighbourWindow, } from './repair-chunk-evidence.ts';
import type {
  RepairModels,
  RepairSliceSeating,
} from '@monochromatic-dev/translation-repair-assembly/ts';
import {
  type CheckerStageResult,
  runCheckerStage,
} from './repair-edit-stages.ts';
import type { runEditorStage, } from './repair-editor-stage.ts';
import type { StageSettings, } from '@monochromatic-dev/translation-repair-stage/ts';

//region Chunk proof
// SPLIT OUT OF `repair-chunk.ts` for the file-length cap when class one
// hundred nine added the checker stage's re-seat. The proof is what runs
// after the editor has written: the checkers over the patched candidate and
// the shadow probe over the regions the edit replaced, both on the bench as
// seated AT THIS STAGE rather than at the chunk's start.

/**
 Envelopes the editor may write into, as `patch-model.ts` derives them.
 */
type Envelopes = readonly EditableEnvelope[];

/**
 What the editor stage returned.
 */
type EditorOutcome = Awaited<ReturnType<typeof runEditorStage>>;

/**
 Who wrote the text answering for each issue, as the checker sheet names it.
 */
type Authorship = Parameters<typeof runCheckerStage>[0]['authorship'];

/**
 Checker proof, the regions the edit replaced, and the probe over them.
 */
export type ChunkProof = {
  /**
   Checker proof over the patched candidate.
   */
  readonly checker: CheckerStageResult;
  /**
   Regions the accuracy stage replaced.
   */
  readonly repairRegions: ReturnType<typeof collectRepairRegions>;
  /**
   Shadow-mode audit of damage the edit itself caused.

   Nothing downstream reads this to decide what ships, on purpose: see
   `introduced-defect-probe.ts` for why an unmeasured probe must not gate.
   */
  readonly introducedDefects: Awaited<ReturnType<typeof runIntroducedDefectProbe>>;
};

/**
 Proves one chunk's patched candidate: the checker stage and the introduced
 defect probe, both on the checker bench the seating names now.

 @param client - injected model client

 @param models - roster the chunk was seated with

 @param reseat - reads the seating as of now (class one hundred nine)

 @param sourceText - original of this chunk

 @param targetText - archive English of this chunk, the probe's baseline

 @param envelopes - envelopes the editor wrote into

 @param editor - what the editor stage returned

 @param acceptedIssues - issues the panel accepted, the checkers' work list

 @param authorship - who wrote the text answering for each issue

 @param neighbours - auditor's window, the very one the critic, panel and
 editor were shown, since a probe without it was measured reporting nothing
 about a duplication whose other half sits in the slice next door, which no
 setting could have fixed

 @param identityContext - declared names and handles for the checkers and
 the probe, or the empty string on a page declaring none (ledger H8, L14)

 @param referenceContext - what the pages the original links say, for the
 checkers; absent when it links nowhere (ledger L14)

 @param signal - caller abort honoured by every exchange

 @param perCallTimeoutMs - deadline per exchange

 @param l - pipeline logger

 @param stageSettings - straggler windows and contest ledger the entry file
 read for this launch (`StageSettings`)

 @returns Checker proof, replaced regions and the probe's audit

 @example
 ```ts
 const { checker, repairRegions, introducedDefects, } = await proveRepairedChunk({ ... },);
 ```
 */
export async function proveRepairedChunk(
  {
    client,
    models,
    reseat,
    sourceText,
    targetText,
    envelopes,
    editor,
    acceptedIssues,
    authorship,
    neighbours,
    identityContext,
    referenceContext,
    signal,
    perCallTimeoutMs,
    l,
    stageSettings,
  }: ForeignBorrowed<{
    readonly client: ModelClient;
    readonly models: RepairModels;
    readonly reseat: () => Promise<RepairSliceSeating>;
    readonly sourceText: string;
    readonly targetText: string;
    readonly envelopes: Envelopes;
    readonly editor: EditorOutcome;
    readonly acceptedIssues: readonly AdjudicatedIssue[];
    readonly authorship: Authorship;
    readonly neighbours: NeighbourWindow;
    readonly identityContext: string;
    readonly referenceContext?: string;
    readonly signal: AbortSignal;
    readonly perCallTimeoutMs: number;
    readonly l: Logger;
    readonly stageSettings: StageSettings;
  }>,
): Promise<ChunkProof> {
  /**
   Checkers as seated now, read again at the stage so a chunk in flight at
   a dry-out asks the bench a fresh reading seats (class one hundred nine).
   */
  const stageCheckers = await checkerBenchAtStage({
    models,
    reseat,
    l,
  },);

  /**
   Checker proof over the patched candidate.
   */
  const checker = await runCheckerStage({
    client,
    checkerModelIds: stageCheckers,
    sourceText,
    patchedText: editor.patch
      .patchedText,
    issues: acceptedIssues,
    authorship,
    // THE PANEL'S EVIDENCE (ledger L14): the checkers' worse ballots strip an
    // edit (ledger L3), and one reading without the declared names would count
    // a declared handle kept as written as damage.
    identityContext,
    ...((referenceContext === undefined) ? {} : { referenceContext, }),
    signal,
    perCallTimeoutMs,
    l,
    stageSettings,
  },);

  /**
   Regions the accuracy stage replaced.
   */
  const repairRegions = collectRepairRegions({
    envelopes,
    applied: editor.patch
      .applied,
  },);

  /**
   Shadow-mode audit of damage the edit itself caused.
   */
  const introducedDefects = await runIntroducedDefectProbe({
    client,
    proberModelIds: stageCheckers,
    sourceText,
    baselineText: targetText,
    regions: repairRegions,
    issues: acceptedIssues,
    ...neighbours,
    identityContext,
    // Withheld on purpose: rendering the accepted issues into the prompt was
    // measured to silence this stage, and `introduced-defect-screen.ts` now
    // dismisses a claim that merely restates one.
    disclosure: PRODUCTION_PRIOR_ISSUE_DISCLOSURE,
    signal,
    perCallTimeoutMs,
    l,
    stageSettings,
  },);

  return {
    checker,
    repairRegions,
    introducedDefects,
  };
}

//endregion Chunk proof
