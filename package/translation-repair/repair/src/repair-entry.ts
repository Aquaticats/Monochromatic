import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';

import type { ModelClient, } from '@monochromatic-dev/translation-repair-provider/ts';
import { prepareDocumentPair, } from '@monochromatic-dev/translation-repair-adjudication/ts';
import type {
  ChunkRepairOutcome,
  RepairModels,
  RepairTranslationResult,
} from '@monochromatic-dev/translation-repair-assembly/ts';
import { repairPreparedDocument, } from './repair-translation.ts';
import {
  SLICE_CHAR_BUDGET,
  type SliceCache,
} from '@monochromatic-dev/translation-repair-translate-floor/ts';
import type { StageSettings, } from '@monochromatic-dev/translation-repair-stage/ts';

//region Repair entry
// The text-in entry point for the repair lane, kept apart from the driver it
// delegates to.
//
// SPLIT OUT rather than shrunk: `repair-translation.ts` reached its 300 line
// budget, and the two halves divide on a real boundary. This one takes two
// documents and decides how to slice them; the driver takes slices already
// decided and runs the roster over them. A caller running BOTH lanes must not
// come through here, because preparing twice is how the lanes come to disagree
// about what a slice is.

/**
 Repairs one translation against its original, preparing the pair first.

 The entry point for a caller running the repair lane ALONE. A caller running
 both lanes prepares once and calls {@link repairPreparedDocument} directly,
 so the two lanes cannot disagree about what a slice is.

 @param client - injected model client

 @param sourceText - original document, front matter included

 @param targetText - translation under repair, front matter included

 @param models - role roster

 @param signal - caller abort honored by every exchange

 @param perCallTimeoutMs - deadline per exchange

 @param sliceCharBudget - target-side characters one paragraph-bound
 slice aims for; defaults to {@link SLICE_CHAR_BUDGET}

 @param sliceCache - optional cross-run cache; resumes finished slices
 and persists newly finished ones so a large document survives aborts

 @param overlap - most slices in flight; defaults to one, while corpus pass
 owns its environment-backed dial separately

 @param stageSettings - straggler windows and contest ledger the entry file
 read for this launch (`StageSettings`)

 @returns Repaired candidate plus adjudicated issues and completion status

 @example
 ```ts
 const result = await repairTranslation({
   client,
   sourceText,
   targetText,
   models,
   signal,
   perCallTimeoutMs,
   stageSettings,
 },);
 ```
 */
export async function repairTranslation(
  {
    client,
    sourceText,
    targetText,
    models,
    signal,
    perCallTimeoutMs,
    sliceCharBudget = SLICE_CHAR_BUDGET,
    sliceCache,
    overlap = 1,
    stageSettings,
  }: ForeignBorrowed<{
    readonly client: ModelClient;
    readonly sourceText: string;
    readonly targetText: string;
    readonly models: RepairModels;
    readonly signal: AbortSignal;
    readonly perCallTimeoutMs: number;
    readonly sliceCharBudget?: number;
    readonly sliceCache?: SliceCache<ChunkRepairOutcome>;
    readonly overlap?: number;
    readonly stageSettings: StageSettings;
  }>,
): Promise<RepairTranslationResult> {
  return await repairPreparedDocument({
    client,
    prepared: prepareDocumentPair({
      sourceText,
      targetText,
      sliceCharBudget,
    },),
    models,
    signal,
    perCallTimeoutMs,
    ...(sliceCache === undefined ? {} : { sliceCache, }),
    overlap,
    stageSettings,
  },);
}

//endregion Repair entry
