import type { Logger, } from '@monochromatic-dev/logger-core/ts';
import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';

import type { PatchOutcome, } from '@monochromatic-dev/translation-repair-page-pass/ts';
import { producerModelIds, } from '@monochromatic-dev/translation-repair-house-rule/ts';
import type { ModelClient, } from '@monochromatic-dev/translation-repair-provider/ts';
import {
  type EditorStageResult,
  runEditorStage,
} from '@monochromatic-dev/translation-repair-repair/ts';
import type { RosterModelId, } from '@monochromatic-dev/translation-repair-shared-contract/ts';
import type { WidthProbeInput, } from './editor-width-input.ts';
import { RUN_PER_CALL_TIMEOUT_MS, } from '@monochromatic-dev/translation-repair-run/ts';
import type { StageSettings, } from '@monochromatic-dev/translation-repair-stage/ts';

//region Editor width arm
// One editor roster's attempt at one slice.
//
// An arm is the whole repair stage run at one width: those editors propose,
// the fixed panel ranks, and one patch ships. The width is the only thing that
// varies between arms, which is what makes a difference between them
// attributable to the seats rather than to the sheet, the panel, or the work.

/**
 What one arm shipped, reduced to what the comparison reads.
 */
export type ArmOutcome = {
  /**
   Text that shipped, blank when the arm changed nothing.
   */
  readonly text: string;

  /**
   Whole patch, kept so the head-to-head can put it on a slate unaltered.
   */
  readonly patch: PatchOutcome;

  /**
   Editors heard out of those seated.
   */
  readonly heard: number;

  /**
   Models that wrote the shipped text.
   */
  readonly producers: readonly RosterModelId[];
};

/**
 Reduces a stage result to the fields the comparison needs.

 READS THE PRODUCER OFF THE RESULT rather than indexing the slate.
 `selectedIndex` is one-based over a slate the round may have reordered, which
 is a documented trap; `shippedProducer` is the stage's own answer.

 @param stage - what one arm returned

 @param targetText - translation as it stood, so an unchanged arm reads blank

 @returns Arm reduced for comparison

 @example
 ```ts
 const arm = readArm({ stage, targetText, },);
 ```
 */
function readArm(
  {
    stage,
    targetText,
  }: {
    readonly stage: EditorStageResult;
    readonly targetText: string;
  },
): ArmOutcome {
  /**
   Producer the stage recorded for the text it shipped.
   */
  const {
    shippedProducer,
    patch,
    heardEditors,
  } = stage;

  /**
   Text this arm produced, which is the incumbent when nothing applied.
   */
  const { patchedText, } = patch;

  return {
    // An arm that shipped the translation untouched shipped no repair, and
    // `classifyWidths` reads that as blank so two such arms are reported as
    // nothing shipped rather than as agreement about a repair.
    text: (patchedText === targetText) ? '' : patchedText,
    patch,
    heard: heardEditors,
    producers: (shippedProducer.kind === 'unattributed')
      ? []
      : producerModelIds(shippedProducer,),
  };
}

/**
 Runs one arm end to end.

 @param client - injected model client

 @param input - slice with its accepted issues

 @param editorModelIds - seats for this arm, the variable under test

 @param judgeModelIds - panel, held fixed across every arm

 @param signal - cancellation

 @param l - logger

 @param stageSettings - straggler windows and contest ledger the entry file
 read for this launch (`StageSettings`)

 @returns Arm reduced for comparison

 @example
 ```ts
 const narrow = await runArm({ client, input, editorModelIds, judgeModelIds, signal, l, stageSettings, },);
 ```
 */
export async function runArm(
  {
    client,
    input,
    editorModelIds,
    judgeModelIds,
    signal,
    l,
    stageSettings,
  }: ForeignBorrowed<{
    readonly client: ModelClient;
    readonly input: WidthProbeInput;
    readonly editorModelIds: readonly RosterModelId[];
    readonly judgeModelIds: readonly RosterModelId[];
    readonly signal: AbortSignal;
    readonly l: Logger;
    readonly stageSettings: StageSettings;
  }>,
): Promise<ArmOutcome> {
  /**
   What this arm's editors and the fixed panel settled on.
   */
  const stage = await runEditorStage({
    client,
    editorModelIds,
    judgeModelIds,
    sourceText: input.sourceText,
    targetText: input.targetText,
    envelopes: input.envelopes,
    issues: input.issues,
    signal,
    perCallTimeoutMs: RUN_PER_CALL_TIMEOUT_MS,
    l,
    stageSettings,
  },);

  return readArm({
    stage,
    targetText: input.targetText,
  },);
}

//endregion Editor width arm
