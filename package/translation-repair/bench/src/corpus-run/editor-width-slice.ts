import type { Logger, } from '@monochromatic-dev/logger-core/ts';
import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';

import type { ModelClient, } from '@monochromatic-dev/translation-repair-provider/ts';
import type { RosterModelId, } from '@monochromatic-dev/translation-repair-shared-contract/ts';
import { runArm, } from './editor-width-arm.ts';
import { bothOrders, } from './editor-width-contest.ts';
import type { WidthProbeInput, } from './editor-width-input.ts';
import {
  classifyWidths,
  type WidthRow,
} from './editor-width-model.ts';
import type { StageSettings, } from '@monochromatic-dev/translation-repair-stage/ts';

//region Editor width slice
// One slice run at both editor widths, plus the rounds that say whether a
// difference between them means anything.
//
// THREE EDITOR STAGES RUN PER SLICE, not two. The third repeats the narrow arm
// so the comparison has a null band of its own. Without it the headline number,
// how often widening changed the shipped text, cannot be told apart from the
// lane simply disagreeing with itself.
//
// The three run one after another rather than at once, so all of them meet the
// same provider conditions. This is the lesson `roster-bench.ts` records about
// interleaving: a provider measured to degrade by the day and to shed bursts
// under load will otherwise write its weather into whichever arm ran later.

/**
 Runs one slice at both widths and reads the result.

 @param client - injected model client

 @param input - slice with its accepted issues

 @param narrowEditorIds - seats for the narrow arm

 @param wideEditorIds - seats for the wide arm

 @param judgeModelIds - panel, held fixed throughout, which is what makes a
 difference between the arms attributable to the seats

 @param signal - cancellation

 @param l - logger

 @param stageSettings - straggler windows and contest ledger the entry file
 read for this launch (`StageSettings`)

 @returns Row this slice contributed

 @example
 ```ts
 const row = await runWidthSlice({ client, input, narrowEditorIds, wideEditorIds, judgeModelIds, signal, l, stageSettings, },);
 ```
 */
export async function runWidthSlice(
  {
    client,
    input,
    narrowEditorIds,
    wideEditorIds,
    judgeModelIds,
    signal,
    l,
    stageSettings,
  }: ForeignBorrowed<{
    readonly client: ModelClient;
    readonly input: WidthProbeInput;
    readonly narrowEditorIds: readonly RosterModelId[];
    readonly wideEditorIds: readonly RosterModelId[];
    readonly judgeModelIds: readonly RosterModelId[];
    readonly signal: AbortSignal;
    readonly l: Logger;
    readonly stageSettings: StageSettings;
  }>,
): Promise<WidthRow> {
  /**
   The narrow arm.
   */
  const narrow = await runArm({
    client,
    input,
    editorModelIds: narrowEditorIds,
    judgeModelIds,
    signal,
    l,
    stageSettings,
  },);

  /**
   The narrow arm again, which is the null band.
   */
  const narrowAgain = await runArm({
    client,
    input,
    editorModelIds: narrowEditorIds,
    judgeModelIds,
    signal,
    l,
    stageSettings,
  },);

  /**
   The wide arm.
   */
  const wide = await runArm({
    client,
    input,
    editorModelIds: wideEditorIds,
    judgeModelIds,
    signal,
    l,
    stageSettings,
  },);

  /**
   Which of the three cases this slice is.
   */
  const comparison = classifyWidths({
    narrowText: narrow.text,
    wideText: wide.text,
  },);

  /**
   Everything the head-to-head decided, absent where none was earned.

   Slices whose arms shipped the same text are answered already, and judging
   a text against itself would spend twelve ballots to learn that twice.
   */
  const contested = (comparison === 'differs')
    ? await bothOrders({
      client,
      input,
      narrow,
      wide,
      judgeModelIds,
      signal,
      l,
      stageSettings,
    },)
    : {
      verdict: 'not-run' as const,
      usableBallots: 0,
    };

  /**
   Issues the editors were actually given work by.
   */
  const acceptedIssues = input
    .issues
    .filter(function isAccepted(issue,) {
      return issue.status === 'accepted';
    },)
    .length;

  return {
    entryId: input.entryId,
    sliceIndex: input.sliceIndex,
    acceptedIssues,
    comparison,
    heardNarrow: narrow.heard,
    heardWide: wide.heard,
    narrowShipped: narrow.text !== '',
    wideShipped: wide.text !== '',
    narrowRepeatAgreed: narrow.text === narrowAgain.text,
    verdict: contested.verdict,
    usableBallots: contested.usableBallots,
    narrowProducers: narrow.producers,
    wideProducers: wide.producers,
  };
}

//endregion Editor width slice
