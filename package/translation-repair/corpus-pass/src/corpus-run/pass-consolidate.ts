import type { Logger, } from '@monochromatic-dev/logger-core/ts';
import type { ArchiveDispute, } from '@monochromatic-dev/translation-repair-archive/ts';

import { consolidateDocument, } from '../consolidate-driver.ts';
import { configuredConsolidationPolish, } from '@monochromatic-dev/translation-repair-consolidate/ts';
import { sliceNeighbourContexts, } from '@monochromatic-dev/translation-repair-repair/ts';
import {
  type PairedReading,
  slicePictureContexts,
} from '@monochromatic-dev/translation-repair-slice-reading/ts';
import type {
  ArtifactConsolidateSlice,
  ProjectedLanes,
} from '@monochromatic-dev/translation-repair-artifact/ts';
import type { ArtifactContestSlice, } from '@monochromatic-dev/translation-repair-lane-contest/ts';
import { openConsolidateCache, } from '@monochromatic-dev/translation-repair-slice-cache/ts';
import type { PipelineDigest, } from '@monochromatic-dev/translation-repair-glossary/ts';
import {
  readJudgeSeats,
  RUN_PER_CALL_TIMEOUT_MS,
  type RunClient,
} from '@monochromatic-dev/translation-repair-run/ts';
import {
  consolidationHooksFor,
  slateReachOf,
} from './pass-consolidate-reseat.ts';
import type { StageSettings, } from '@monochromatic-dev/translation-repair-stage/ts';
import {
  type PreparedDocumentPair,
  preparedSheetFactsOf,
} from '@monochromatic-dev/translation-repair-corpus-source/ts';

//region Corpus pass consolidation

/**
 Runs production consolidation and final naturalness polish for one entry.

 MODEL ROLES, WINDOWS, CACHE AND DOCUMENT FACTS ARE ASSEMBLED HERE so pass
 orchestration makes one call rather than duplicating stage interface details.
 This is same seam at which all four positional maps are simultaneously known.

 READS ITS OWN BENCH off every provider's meter as it stands when
 consolidation starts, for the reason `runPassContest` gives: a reading taken
 at the lanes is hours old by now on a long entry (`run-seats.ts`).

 @param client - shared provider client

 @param prepared - shared source-target preparation

 @param projected - both lane ledgers and comparison

 @param archiveDisputes - slices whose archive rendering the repair lane's
 adjudicators disputed, whose repair text stands in for the archive (class
 one hundred seven)

 @param acceptedRepairClaimNotes - claims that panel accepted per slice,
 which the final polish gate reads

 @param contests - lane contest records

 @param frontMatterSlices - syntax-bearing metadata positions

 @param pictureReadings - paired OCR evidence

 @param entryCacheDir - per-entry cache root

 @param pipelineDigest - generation stamp for cache

 @param signal - entry cancellation

 @param overlap - most slices in flight

 @param l - entry logger

 @param stageSettings - straggler windows and contest ledger the entry file
 read for this launch (`StageSettings`)

 @returns Consolidation records in comparison order

 @example
 ```ts
 const slices = await runPassConsolidation({ client, prepared, projected, contests, frontMatterSlices, pictureReadings, entryCacheDir, pipelineDigest, signal, overlap, l, stageSettings, });
 ```
 */
export async function runPassConsolidation(
  {
    client,
    prepared,
    projected,
    contests,
    frontMatterSlices,
    pictureReadings,
    entryCacheDir,
    pipelineDigest,
    signal,
    overlap,
    archiveDisputes,
    acceptedRepairClaimNotes,
    l,
    stageSettings,
  }: {
    readonly client: RunClient;
    readonly archiveDisputes?: ReadonlyMap<number, ArchiveDispute>;
    readonly acceptedRepairClaimNotes?: ReadonlyMap<number, string>;
    readonly prepared: PreparedDocumentPair;
    readonly projected: ProjectedLanes;
    readonly contests: readonly ArtifactContestSlice[];
    readonly frontMatterSlices: ReadonlySet<number>;
    readonly pictureReadings: ReadonlyMap<string, PairedReading>;
    readonly entryCacheDir: string;
    readonly pipelineDigest: PipelineDigest;
    readonly signal: AbortSignal;
    readonly overlap: number;
    readonly l: Logger;
    readonly stageSettings: StageSettings;
  },
): Promise<readonly ArtifactConsolidateSlice[]> {
  /**
   This phase's judge benches, read now.
   */
  const seats = await readJudgeSeats({
    client,
    phase: 'consolidation',
    signal,
    l,
  },);
  /**
   Production naturalness roles and document guard facts, always configured:
   the run's roster seats its refiners (`RunRepairModels`).
   */
  const polishConfig = configuredConsolidationPolish({
    prepared,
    models: seats.repairModels,
    gateModelIds: seats.lateJudges,
  },);
  return await consolidateDocument({
    client,
    projected,
    contests,
    // EVERY CHUNK WAITS OUT A NAMED HOLD that keeps the slate from quorum
    // (`pass-consolidate-reseat.ts`).
    beforeSlice: consolidationHooksFor({
      client,
      signal,
      prepared,
      l,
      startedSlateReachable: slateReachOf({ seats, },),
    },)
      .beforeSlice,
    // THE WHOLE SEATED ROSTER WRITES, the slate judges judge and the late
    // judges gate: GLM-5.3-Flash keeps its writing seats and left every judge
    // seat on 2026-09-02, the reason on `WIDE_SEAT_DROPPED` in
    // `run-config.ts`; the Hyper-slow judge sits only while Synthetic serves
    // it, the select-slow judge judges slates only then, and a model withheld
    // on the provider that would serve it writes nothing (`run-seats.ts`).
    modelIds: seats.writers,
    judgeModelIds: seats.slateJudges,
    polishConfig,
    frontMatterSlices,
    lineStructuredSlices: prepared.lineStructuredSliceIndices,
    // A linked title naming a declared person takes the declared form on
    // every text the consolidation validates (class one hundred fourteen).
    ...((prepared.declaredNamePairs === undefined) ? {} : { declaredNamePairs: prepared.declaredNamePairs, }),
    pictureContextBySlice: slicePictureContexts({
      slices: prepared.slices,
      readings: pictureReadings,
    },),
    neighbourContextBySlice: sliceNeighbourContexts({
      slices: prepared.slices,
    },),
    sheetFacts: preparedSheetFactsOf({ prepared, },),
    cache: await openConsolidateCache({
      dir: entryCacheDir,
      generation: pipelineDigest,
    },),
    ...((archiveDisputes === undefined) ? {} : { archiveDisputes, }),
    ...((acceptedRepairClaimNotes === undefined) ? {} : { acceptedRepairClaimNotes, }),
    signal,
    perCallTimeoutMs: RUN_PER_CALL_TIMEOUT_MS,
    overlap,
    l,
    stageSettings,
  },);
}

//endregion Corpus pass consolidation
