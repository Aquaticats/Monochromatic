import type { Logger, } from '@monochromatic-dev/logger-core/ts';
import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';

import type { ModelClient, } from '@monochromatic-dev/translation-repair-provider/ts';
import { frontMatterRepairOutcome, } from './front-matter-repair.ts';
import type { ChunkPair, } from '@monochromatic-dev/translation-repair-slice-reading/ts';
import type { PreparedDocumentPair, } from '@monochromatic-dev/translation-repair-corpus-source/ts';
import { exchangeFailureLogText, } from '@monochromatic-dev/translation-repair-provider-stream/ts';
import {
  failureWritten,
  type StageSettings,
} from '@monochromatic-dev/translation-repair-stage/ts';
import {
  cacheRefusalLine,
  cacheRefusalsOf,
} from './repair-cache-gate.ts';
import { repairChunk, } from './repair-chunk.ts';
import type {
  ChunkRepairOutcome,
  RepairModels,
  RepairSliceSeating,
} from '@monochromatic-dev/translation-repair-assembly/ts';
import {
  assertSettledRecordAgrees,
  type SliceCache,
} from '@monochromatic-dev/translation-repair-translate-floor/ts';

//region Repair slice buy
// What one repair-lane slice costs when neither disk nor a twin can answer:
// the whole accuracy loop, abort normalization, agreement guard and persistence
// gate.
//
// CACHE ELIGIBILITY ALSO DECIDES THE IN-RUN MEMO. This preserves the sequential
// driver's behavior even when no external cache was supplied: an eligible
// outcome was put into `settledByKey` after the optional persist and a twin
// reused it. A refused outcome was not, so its twin asked again. The promise
// memo keeps that cold-run behavior while the first twin is still buying.

/**
 Buys one repair slice and persists it when every stage reached quorum.

 @param client - injected model client

 @param prepared - document slice belongs to

 @param models - repair role roster

 @param reseat - reads the seating again at the checker stage, so a chunk
 in flight when a provider runs dry asks the bench a fresh reading seats
 (class one hundred nine)

 @param slice - slice being repaired

 @param key - cross-run key outcome is stored under

 @param neighbouringIncumbentText - archive English of adjacent passages

 @param neighbouringSourceText - original of adjacent passages

 @param documentSourceText - same-entry source evidence identical to the value used in the cache key

 @param sliceCache - optional cross-run cache

 @param signal - entry deadline and caller abort

 @param perCallTimeoutMs - deadline per exchange

 @param l - repair-lane logger

 @param stageSettings - straggler windows and contest ledger the entry file
 read for this launch (`StageSettings`)

 @returns Settled repair outcome, whether or not cache gate accepted it

 @throws Whatever `signal.reason` carries when caller aborts before or during
 purchase

 @example
 ```ts
 const outcome = await buyRepairSlice({ ... });
 ```
 */
export async function buyRepairSlice(
  {
    client,
    prepared,
    models,
    reseat,
    slice,
    key,
    neighbouringIncumbentText,
    neighbouringSourceText,
    documentSourceText,
    sliceCache,
    signal,
    perCallTimeoutMs,
    l,
    stageSettings,
  }: ForeignBorrowed<{
    readonly client: ModelClient;
    readonly prepared: PreparedDocumentPair;
    readonly models: RepairModels;
    readonly reseat?: () => Promise<RepairSliceSeating>;
    readonly slice: ChunkPair;
    readonly key: string;
    readonly neighbouringIncumbentText: string;
    readonly neighbouringSourceText: string;
    readonly documentSourceText: string;
    readonly sliceCache?: SliceCache<ChunkRepairOutcome>;
    readonly signal: AbortSignal;
    readonly perCallTimeoutMs: number;
    readonly l: Logger;
    readonly stageSettings: StageSettings;
  }>,
): Promise<ChunkRepairOutcome> {
  /**
   Global index every outcome and replacement names.
   */
  const { sliceIndex, } = slice.target;

  // Cached slices may finish under an already spent signal. A purchase may not.
  signal.throwIfAborted();
  if (slice.syntax === 'front-matter') {
    /**
     Archive metadata retained by repair lane.
     */
    const { text: targetText, } = slice.target;
    return frontMatterRepairOutcome({
      sliceIndex,
      targetText,
    },);
  }

  /**
   Fresh outcome from full repair stage sequence, normalizing an abort to its
   own reason rather than whichever torn-down exchange surfaced first.
   */
  const outcome = await (async function repairUnderSignal(): Promise<ChunkRepairOutcome> {
    try {
      return await repairChunk({
        client,
        sliceIndex,
        sourceText: slice.source
          .text,
        targetText: slice.target
          .text,
        lineStructured: prepared.lineStructuredSliceIndices
          .has(sliceIndex,),
        declaredNames: prepared.declaredNames,
        ...((prepared.declaredSubjectForms === undefined) ? {} : { declaredSubjectForms: prepared.declaredSubjectForms, }),
        neighbouringIncumbentText,
        neighbouringSourceText,
        ...(documentSourceText === '' ? {} : { documentSourceText, }),
        models,
        ...((reseat === undefined) ? {} : { reseat, }),
        ...((prepared.identityContext === undefined)
          ? {}
          : { identityContext: prepared.identityContext, }),
        ...((prepared.referenceContext === undefined)
          ? {}
          : { referenceContext: prepared.referenceContext, }),
        ...((prepared.attestedDetails === undefined)
          ? {}
          : { attestedDetails: prepared.attestedDetails, }),
        signal,
        perCallTimeoutMs,
        l,
        stageSettings,
      },);
    }
    catch (error) {
      // A provider fault under a live signal keeps its own identity.
      if (!signal.aborted)
        throw error;
      // A FAILURE A LOWER LAYER ALREADY WROTE keeps its words on that line
      // alone, such as the refusal the seat reading writes when the checker
      // stage's re-seat stops the entry (ledger B413 left this open, B415 ran
      // it); this line then says only that the chunk was abandoned, as the
      // translate attempt's does (`translate-slice-attempt.ts`).
      /**
       What surfaced, in parentheses, where no lower layer wrote it.
       */
      const surfaced = failureWritten({ error, },) ? '' : ` (${exchangeFailureLogText({ error, },)})`;
      l.warn(`chunk ${String(sliceIndex,)}: abandoned by the caller's abort${surfaced}`,);
      throw signal.reason;
    }
  })();

  // Every abandoned exchange reaches stages as silence. Without this check a
  // spent run could persist an ordinary unchanged outcome nobody decided on.
  signal.throwIfAborted();

  assertSettledRecordAgrees({
    lane: 'repair',
    sliceIndex,
    changed: outcome.changed,
    decidedText: outcome.repairedText,
    incumbentText: slice.target
      .text,
  },);

  /**
   Reasons this outcome cannot be resumed by a warm run or reused by a twin.
   */
  const refusals = cacheRefusalsOf({ outcome, },);
  if (refusals.length > 0) {
    l.warn(cacheRefusalLine({
      sliceIndex,
      refusals,
      changed: outcome.changed,
    },),);
    return outcome;
  }

  // Checked again at write boundary so no abort can land between agreement and
  // persistence and turn a partially heard slice into resumable work.
  signal.throwIfAborted();
  await sliceCache?.persist({
    key,
    serialized: JSON.stringify(
      outcome,
      undefined,
      2,
    ),
  },);
  return outcome;
}

//endregion Repair slice buy
