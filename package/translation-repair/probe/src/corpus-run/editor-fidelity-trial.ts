import type { Logger, } from '@monochromatic-dev/module-logger/ts';
import { nonNullishOrThrow, } from '@monochromatic-dev/module-or-throw/ts';

import type { ModelClient, } from '@monochromatic-dev/translation-repair-provider/ts';
import { gatherEditorProposals, } from '@monochromatic-dev/translation-repair-repair/ts';
import type { RosterModelId, } from '@monochromatic-dev/translation-repair-shared-contract/ts';
import type { StageSettings, } from '@monochromatic-dev/translation-repair-stage/ts';
import {
  type EditorFidelityScore,
  scoreEditedText,
} from '../editor-fidelity-score.ts';
import { sheetFieldsOf, } from '../editor-fidelity-context.ts';
import type { EditorFidelityTrial, } from '../editor-fidelity-trials.ts';

//region Editor fidelity trial
// ONE CARD SEATED ALONE ON ONE TRIAL, asked what the pipeline's editor stage asks an editor and read by what the
// apply gate lets through.
//
// THE STEP IS THE STAGE'S OWN. `gatherEditorProposals` builds the sheet (with `buildEditorMessages`, the builder the
// corpus pass uses) from the section, the accepted issue, the envelope and the context of the page, asks the editors
// through the gather (its retry rounds and its recovery round included) and gates what they propose, and
// `runEditorStage` takes the same step before it selects. Nothing after it runs here: no critic, no panel, no judge and
// no checker, so the only model asked is the card.
//
// FOUR READINGS, EACH ITS OWN. UNUSABLE: no reply the gather could read, or edits proposed and none landed (the stage
// drops an edit naming a region that is not on the sheet, and the apply gate refuses one that leaves its envelope or
// breaks the preservation rules). REFUSED: the editor was heard and proposed no edit. ANSWERED: an edit landed, and
// the text is scored.

/**
 How one trial read.

 @example
 ```ts
 const reading: EditorTrialReading = { kind: 'refused', findings: [], };
 ```
 */
export type EditorTrialReading =
  | {
    /**
     No usable reply: nothing the gather could read, or edits proposed and none landed.
     */
    readonly kind: 'unusable';

    /**
     Which of the two it was.
     */
    readonly cause: 'no-reply' | 'edits-refused';

    /**
     Edits the card proposed that did not land, dropped on the sheet or refused by the apply gate.
     */
    readonly rejected: number;

    /**
     Wire irregularities and gate findings, in scorecard-stable wording.
     */
    readonly findings: readonly string[];
  }
    | {
    /**
     The editor was heard and proposed no edit.
     */
      readonly kind: 'refused';

    /**
     Wire irregularities, in scorecard-stable wording.
     */
      readonly findings: readonly string[];
    }
    | {
    /**
     An edit landed and the patched text was scored.
     */
      readonly kind: 'answered';

    /**
     The deterministic reading of the patched text.
     */
      readonly score: EditorFidelityScore;

    /**
     Edits the apply gate applied.
     */
      readonly applied: number;

    /**
     Edits it refused beside them.
     */
      readonly rejected: number;

    /**
     Wire irregularities, in scorecard-stable wording.
     */
      readonly findings: readonly string[];
    };

/**
 Seats one card on one trial and reads what it did.

 @param client - client the card is asked through

 @param cardId - the card, seated alone

 @param trial - trial the card is asked

 @param signal - caller abort honored by every exchange

 @param perCallTimeoutMs - deadline per exchange

 @param l - pipeline logger

 @param stageSettings - straggler windows and contest ledger the entry file read for this launch (`StageSettings`)

 @returns How the trial read

 @example
 ```ts
 const reading = await runEditorTrial({ client, cardId, trial, signal, perCallTimeoutMs, l, stageSettings, },);
 ```
 */
export async function runEditorTrial(
  {
    client,
    cardId,
    trial,
    signal,
    perCallTimeoutMs,
    l,
    stageSettings,
  }: {
    readonly client: ModelClient;
    readonly cardId: RosterModelId;
    readonly trial: EditorFidelityTrial;
    readonly signal: AbortSignal;
    readonly perCallTimeoutMs: number;
    readonly l: Logger;
    readonly stageSettings: StageSettings;
  },
): Promise<EditorTrialReading> {
  /**
   The sheet, the heard reply and the gated candidate, as the editor stage builds them.
   */
  const proposals = await gatherEditorProposals({
    client,
    editorModelIds: [cardId,],
    sourceText: trial.sourceText,
    targetText: trial.damagedText,
    envelopes: trial.envelopes,
    issues: trial.issues,
    ...sheetFieldsOf({ context: trial.context, },),
    signal,
    perCallTimeoutMs,
    l,
    stageSettings,
  },);
  /**
   Everything the gather and the gate said about the reply.
   */
  const findings = [
    ...proposals.gather
      .findings,
    ...proposals.built
      .findings,
  ];
  /**
   The one candidate a heard card builds, absent when nobody was heard.
   */
  const [candidate,] = proposals.built
    .candidates;
  if (candidate === undefined) {
    return {
      kind: 'unusable',
      cause: 'no-reply',
      rejected: 0,
      findings,
    };
  }
  /**
   What the apply gate made of the card's edits.
   */
  const { patch, } = candidate;
  /**
   Edits the heard card proposed, before the sheet's regions and the gate had their say.
   */
  const proposed = nonNullishOrThrow(proposals.gather
    .voices[0],)
    .value
    .edits
    .length;
  if (patch.applied
    .length
    === 0) {
    if (proposed > 0) {
      return {
        kind: 'unusable',
        cause: 'edits-refused',
        rejected: proposed,
        findings: [
          ...findings,
          ...patch.rejected
            .map(function reasonOf(rejection,): string {
              return rejection.reason;
            },),
        ],
      };
    }
    return {
      kind: 'refused',
      findings,
    };
  }
  return {
    kind: 'answered',
    score: scoreEditedText({
      trial,
      patchedText: patch.patchedText,
    },),
    applied: patch.applied
      .length,
    rejected: patch.rejected
      .length,
    findings,
  };
}

//endregion Editor fidelity trial
