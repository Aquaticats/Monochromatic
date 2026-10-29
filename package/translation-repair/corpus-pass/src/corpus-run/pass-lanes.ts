import { tagged, } from '@monochromatic-dev/logger-core/ts';

import {
  sliceTagged,
  wordForCount,
} from '@monochromatic-dev/translation-repair-typography/ts';
import type {
  InsertionAdmission,
  PreparedDocumentPair,
} from '@monochromatic-dev/translation-repair-corpus-source/ts';
import {
  type DocumentLanesResult,
  runDocumentLanes,
  type TranslateSliceRecord,
} from '@monochromatic-dev/translation-repair-translate-lane/ts';
import type { PairedReading, } from '@monochromatic-dev/translation-repair-slice-reading/ts';
import type { RefinedSliceSettlement, } from '@monochromatic-dev/translation-repair-repair/ts';
import type { ChunkRepairOutcome, } from '@monochromatic-dev/translation-repair-assembly/ts';
import type { SliceCache, } from '@monochromatic-dev/translation-repair-translate-floor/ts';
import {
  type JudgeSeats,
  RUN_PER_CALL_TIMEOUT_MS,
  type RunClient,
} from '@monochromatic-dev/translation-repair-run/ts';
import type { LanesHooks, } from './pass-reseat.ts';
import type { StageSettings, } from '@monochromatic-dev/translation-repair-stage/ts';

//region Pass lanes
// THE LANES' SEAM (ledger X14): `runPassEntry` hands both lanes their benches
// and their re-seat hooks here, in one place a test can drive. Spread inline
// inside `runPassEntry`, a dropped hook survived every guard, since no test
// drives that function.

/**
 Both lanes' benches as `readLanesSeats` read them, with the hooks that
 re-seat each lane under a hold.

 @example
 ```ts
 const lanesSeating: LanesSeating = await readLanesSeats({ client, signal, entryId, },);
 ```
 */
export type LanesSeating = {
  /**
   Benches as of the lanes' reading.
   */
  readonly seats: JudgeSeats;
  /**
   Hooks each lane calls before a chunk or slice.
   */
  readonly lanesHooks: LanesHooks;
};

/**
 Runs both lanes over one entry's slicing on the benches and hooks the
 entry's lanes reading supplied.

 @param client - run client whose chat surface both lanes ask

 @param prepared - slicing both lanes run over

 @param lanesSeating - benches and re-seat hooks from `readLanesSeats`

 @param pictureReadings - what reading produced per picture

 @param signal - entry deadline and caller abort

 @param overlap - lanes' slice overlap for this entry

 @param repairSliceCache - repair lane's per-slice store

 @param refineSliceCache - refinement's per-slice store

 @param translateSliceCache - translate lane's per-slice store

 @param translateInsertionAdmission - source-only passages licensed for translation

 @param entryId - entry the log lines are tagged with

 @param stageSettings - straggler windows and contest ledger the entry file
 read for this launch (`StageSettings`)

 @returns What both lanes made of the slicing, with neither preferred

 @example
 ```ts
 const lanes = await runPassLanes({ client, prepared, lanesSeating, pictureReadings, signal, entryId, stageSettings, },);
 ```
 */
export async function runPassLanes(
  {
    client,
    prepared,
    lanesSeating,
    pictureReadings,
    signal,
    overlap,
    repairSliceCache,
    refineSliceCache,
    translateSliceCache,
    translateInsertionAdmission,
    entryId,
    stageSettings,
  }: {
    readonly client: RunClient;
    readonly prepared: PreparedDocumentPair;
    readonly lanesSeating: LanesSeating;
    readonly pictureReadings: ReadonlyMap<string, PairedReading>;
    readonly signal: AbortSignal;
    readonly overlap?: number;
    readonly repairSliceCache?: SliceCache<ChunkRepairOutcome>;
    readonly refineSliceCache?: SliceCache<RefinedSliceSettlement>;
    readonly translateSliceCache?: SliceCache<TranslateSliceRecord>;
    readonly translateInsertionAdmission?: InsertionAdmission;
    readonly entryId: string;
    readonly stageSettings: StageSettings;
  },
): Promise<DocumentLanesResult> {
  /**
   Entry logger both lanes write to, which names the attempt inside one. The lanes slice-tag the logger they are
   handed, and a line passes through a slice-tagged logger once, so their lines read as they always have (ledger B963).
   */
  const l = sliceTagged({ l: tagged({ tag: entryId, },), },);
  l.debug(`${runPassLanes.name}: running both lanes over ${String(prepared.slices
    .length,)} ${
    wordForCount({
      count: prepared.slices
        .length,
      one: 'slice',
      many: 'slices',
    },)
  }`,);
  return await runDocumentLanes({
    client,
    prepared,
    repairModels: lanesSeating.seats
      .repairModels,
    translateModels: lanesSeating.seats
      .translateModels,
    // EACH LANE RE-SEATS UNDER A HOLD (ledger X12 and H5).
    ...lanesSeating.lanesHooks,
    pictureReadings,
    signal,
    perCallTimeoutMs: RUN_PER_CALL_TIMEOUT_MS,
    ...((overlap === undefined) ? {} : { overlap, }),
    ...((repairSliceCache === undefined) ? {} : { repairSliceCache, }),
    ...((refineSliceCache === undefined) ? {} : { refineSliceCache, }),
    ...((translateSliceCache === undefined) ? {} : { translateSliceCache, }),
    ...((translateInsertionAdmission === undefined) ? {} : { translateInsertionAdmission, }),
    l,
    stageSettings,
  },);
}

//endregion Pass lanes
