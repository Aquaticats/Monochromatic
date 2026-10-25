import type { Logger, } from '@monochromatic-dev/module-logger/ts';
import type { DeclaredNamePair, } from '@monochromatic-dev/translation-repair-house-rule/ts';

import type { DocumentLanesResult, } from '@monochromatic-dev/translation-repair-translate-lane/ts';
import { contestDocumentLanes, } from '../lane-contest-driver.ts';
import { damageClaimLinesBySlice, } from '@monochromatic-dev/translation-repair-repair/ts';
import {
  type ArtifactContestSlice,
  withTenseCandidates,
} from '@monochromatic-dev/translation-repair-lane-contest/ts';
import type { ProjectedLanes, } from '@monochromatic-dev/translation-repair-artifact/ts';
import type { PreparedSheetFacts, } from '@monochromatic-dev/translation-repair-corpus-source/ts';
import { openLaneContestCache, } from '@monochromatic-dev/translation-repair-slice-cache/ts';
import { archiveDisputeNotesOf, } from '@monochromatic-dev/translation-repair-archive/ts';
import type { PipelineDigest, } from '@monochromatic-dev/translation-repair-glossary/ts';
import {
  readJudgeSeats,
  RUN_PER_CALL_TIMEOUT_MS,
  type RunClient,
} from '@monochromatic-dev/translation-repair-run/ts';
import {
  contestHooksFor,
  contestReachOf,
} from './pass-contest-reseat.ts';
import type {
  FanOutMode,
  StageSettings,
} from '@monochromatic-dev/translation-repair-stage/ts';

//region Corpus pass lane contest

/**
 Runs the lane contest for one entry: what the late judges said at every
 slice the two lanes worded differently.

 THE SEAM BESIDE `runPassConsolidation`, for the same reason: the judges, the
 cache, the evidence and the window are assembled here so the entry driver
 makes one call.

 READS ITS OWN BENCH off every provider's meter as it stands when the contest
 starts, not the lanes' reading: XIEPT2 on 2026-09-03 read wet at the lanes,
 ran Synthetic dry seven minutes later, and sat the Hyper-slow judge through
 the contest and the consolidation on Hyper (`run-seats.ts`).

 @param client - shared provider client

 @param lanes - both lanes as they returned, read for the repair lane's
 probe reports

 @param projected - both lane ledgers and the comparison the contest reads

 @param frontMatterSlices - syntax-bearing metadata positions

 @param lineStructuredSlices - positions the line-structure rule governs, so
 the contest's publication verdict refuses a winner merging their lines

 @param sheetFacts - what the page tells every sheet and key: the declared names,
 whether the original's entry records a death, the names the translation
 declares for its subject, and what the pages the original cites say; each
 left out where the page lacks it

 @param entryCacheDir - per-entry cache root

 @param pipelineDigest - generation stamp for the cache

 @param signal - entry cancellation

 @param overlap - most slices in flight

 @param l - entry logger

 @param stageSettings - straggler windows and contest ledger the entry file
 read for this launch (`StageSettings`)

 @param fanOut - seats a contest round asks, left to the contest's default window
 when absent; a case that must see every judge asked passes the whole bench,
 since the default window is rotated by the sheet text

 @returns Contest records in comparison order

 @example
 ```ts
 const contests = await runPassContest({ client, lanes, projected, frontMatterSlices, lineStructuredSlices, entryCacheDir, pipelineDigest, signal, overlap, l, stageSettings, },);
 ```
 */
export async function runPassContest(
  {
    client,
    lanes,
    projected,
    frontMatterSlices,
    lineStructuredSlices,
    sheetFacts,
    declaredNamePairs,
    entryCacheDir,
    pipelineDigest,
    signal,
    overlap,
    l,
    stageSettings,
    fanOut,
  }: {
    readonly client: RunClient;
    readonly lanes: DocumentLanesResult;
    readonly projected: ProjectedLanes;
    readonly frontMatterSlices: ReadonlySet<number>;
    readonly lineStructuredSlices: ReadonlySet<number>;
    readonly sheetFacts?: PreparedSheetFacts;
    readonly declaredNamePairs?: readonly DeclaredNamePair[];
    readonly entryCacheDir: string;
    readonly pipelineDigest: PipelineDigest;
    readonly signal: AbortSignal;
    readonly overlap: number;
    readonly l: Logger;
    readonly stageSettings: StageSettings;
    readonly fanOut?: FanOutMode;
  },
): Promise<readonly ArtifactContestSlice[]> {
  /**
   This phase's judge benches, read now.
   */
  const seats = await readJudgeSeats({
    client,
    phase: 'lane contest',
    signal,
    l,
  },);
  /**
   Records the judges' ballots made, before the tense finding is added.
   */
  const contested = await contestDocumentLanes({
    client,
    projected,
    // JUDGES ONLY: the roster less GLM-5.3-Flash since 2026-09-02 (the reason
    // on `WIDE_SEAT_DROPPED` in `run-config.ts`), less the Hyper-slow judge
    // while Synthetic is dry (`run-seats.ts`).
    modelIds: seats.lateJudges,
    frontMatterSlices,
    lineStructuredSlices,
    // The probe's corroborated claims, shown to the judges and acted on by
    // nobody (`repair-damage-evidence.ts`).
    damageClaimsBySlice: damageClaimLinesBySlice({ lane: lanes.repair, },),
    // The accepted additions against each disputed archive rendering, read
    // off the repair chunks (class one hundred eight).
    disputeNotesBySlice: archiveDisputeNotesOf({
      chunks: lanes.repair
        .chunks,
    },),
    ...((sheetFacts === undefined) ? {} : { sheetFacts, }),
    // A linked title naming a declared person takes the declared form on the
    // contest winner too (class one hundred fourteen).
    ...((declaredNamePairs === undefined) ? {} : { declaredNamePairs, }),
    cache: await openLaneContestCache({
      dir: entryCacheDir,
      generation: pipelineDigest,
    },),
    signal,
    perCallTimeoutMs: RUN_PER_CALL_TIMEOUT_MS,
    overlap,
    // EVERY SLICE RE-SEATS UNDER A HOLD (ledger X12,
    // `pass-contest-reseat.ts`).
    beforeSlice: contestHooksFor({
      client,
      signal,
      l,
      startedReachable: contestReachOf({ seats, },),
    },)
      .beforeSlice,
    l,
    stageSettings,
    // Conditional spread keeps the knob absent instead of undefined.
    ...((fanOut === undefined) ? {} : { fanOut, }),
  },);
  // THE FINDING IS ADDED HERE, beside the wording each slice offered, so the
  // artifact records the candidates under the labels the judges read them by
  // (`lane-contest-tense.ts`) and no verdict reads them.
  return withTenseCandidates({
    slices: contested,
    comparison: projected.comparison,
    lifeEnded: sheetFacts?.lifeEnded === true,
    ...((sheetFacts?.subjectNames === undefined) ? {} : { subjectNames: sheetFacts.subjectNames, }),
  },);
}

//endregion Corpus pass lane contest
