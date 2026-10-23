import { join, } from 'node:path';

import { monotonicMs, } from '@monochromatic-dev/translation-repair-markup/ts';
import type {
  CorpusPair,
  EntryOutcome,
  OverlapSetting,
  ParkAnswer,
} from '@monochromatic-dev/translation-repair-slice-cache/ts';
import type { RunClient, } from '@monochromatic-dev/translation-repair-run/ts';
import type { PipelineDigest, } from '@monochromatic-dev/translation-repair-glossary/ts';
import type { StageSettings, } from '@monochromatic-dev/translation-repair-stage/ts';
import type { CorpusPin, } from '@monochromatic-dev/translation-repair-corpus-source/ts';
import { retireSettledEntryCache, } from './pass-entry-caches.ts';
import { withEntryBound, } from './pass-entry-bound.ts';
import { runEntryPipeline, } from './pass-entry.ts';
import { entryOverlap, } from './pass-overlap.ts';
import type { PassOutsideReads, } from './pass-outside-reads.ts';
import type {
  PassPictureSources,
  PassVisualEvidenceReader,
} from './pass-visual-evidence.ts';

//region Pass entry settlement
// THE ENTRY'S OUTER SHELL, split from `pass-entry.ts` at its line budget: it narrows the client's bound to the entry's
// own cap (`pass-entry-bound.ts`), runs the pipeline, and retires the cache of an entry that settled.

/**
 Runs one chosen entry to settlement, then retires its cache if it settled.

 Never throws for a failed entry. A pass over a corpus stops for a broken SCHEDULER, not for a broken document: an entry
 that aborts on its ceiling or raises out of a stage records `status=ERROR` and the pass continues to the next one.
 Anything raised here would therefore end the run, which is why nothing is.

 Returns scheduling disposition so quality interruption cannot masquerade as
 cache-progress reason for fresh whole-entry attempt.

 @param client - shared model client

 @param entry - corpus pair to settle, text already read

 @param artifactsDir - directory one JSON per settled entry is written into

 @param publishDir - root of the mirrored corpus tree each fixed page is written into

 @param declinedDir - directory a decline record is written into when the
 archive's note says the whole page is the author's own English

 @param sliceCacheDir - root under which this entry claims its own cache subdirectory

 @param tip - repository head recorded into the artifact

 @param pipelineDigest - identity of the built pipeline, which also generation
 -stamps the slice cache so a changed pipeline cannot resume foreign slices

 @param hardCapMs - time this entry may run before its exchanges abort

 @param baseSignal - abort every entry deadline forwards from; the pass aborts it only when the dry router
 stops the run (`pass-dry-stop.ts`), so apart from a per-entry timeout that is the one abort that ever fires

 @param pin - corpus clone and commit the pass read the entry at: the one the
 entry read inside its refusal boundary in a run, an invented one in a test.
 REQUIRED, as `outsideReads` is: the artifact and the decline record name its
 commit and the pictures are read from its clone, which a module constant
 resolved at load supplied until 2026-10-10 (ledger B302, B381)

 @param outsideReads - what preparation reads from outside the pipeline:
 the readers the pass's entry file builds over its environment in a run, a
 test's own readers in a test. REQUIRED, as
 it is of the preparation (ledger X19): this driver once handed the run's
 readers down itself, so its unit test prepared every entry through the key
 the suite inherits, the real caches and the corpus clone (ledger M68)

 @param pictureSources - where the entry's picture bytes and their OCR text
 come from: `RUN_PICTURE_SOURCES` in a run, a test's own in a test. REQUIRED,
 as `outsideReads` is: an entry that names pictures and runs without the
 evidence seam would otherwise read them from the corpus through the real
 programs (ledger M70, M113)

 @param overlap - slices each per-slice driver may keep in flight, beside
 where the value came from, which the pass's entry file read once
 (`readPassOverlap`) and every entry logs on its own `OVERLAP` line

 @param stageSettings - straggler windows and contest ledger the entry file
 read for this launch (`readStageSettings`)

 @param endRun - told of the failure that ended this entry when it ends the run, so the pass stops starting entries

 @param parked - asked after `endRun`: answers the return the pass is now parked for when it is parked for the
 failure, so the entry goes back to the front of its queue, which the TALLY line prints as `PARKED` with that return

 @returns Settlement, resumable operational failure, or stopped incomplete work

 @example
 ```ts
 const outcome = await settleEntry({
   client, entry, artifactsDir, publishDir, declinedDir, sliceCacheDir, tip, pipelineDigest, pin, hardCapMs, baseSignal,
   outsideReads, pictureSources: RUN_PICTURE_SOURCES, overlap, stageSettings,
 },);
 ```
 */
export async function settleEntry(
  {
    client,
    entry,
    artifactsDir,
    publishDir,
    declinedDir,
    sliceCacheDir,
    tip,
    pipelineDigest,
    pin,
    hardCapMs,
    baseSignal,
    outsideReads,
    pictureSources,
    visualEvidenceReader,
    overlap,
    stageSettings,
    endRun,
    parked,
  }: {
    readonly client: RunClient;
    readonly entry: CorpusPair;
    readonly artifactsDir: string;
    readonly publishDir: string;
    readonly declinedDir: string;
    readonly sliceCacheDir: string;
    readonly tip: string;
    readonly pipelineDigest: PipelineDigest;
    readonly pin: CorpusPin;
    readonly hardCapMs: number;
    readonly baseSignal: AbortSignal;
    readonly outsideReads: PassOutsideReads;
    readonly pictureSources: PassPictureSources;
    readonly visualEvidenceReader?: PassVisualEvidenceReader;
    readonly overlap: OverlapSetting;
    readonly stageSettings: StageSettings;
    readonly endRun?: (args: { readonly error: unknown; },) => void;
    readonly parked?: (args: { readonly error: unknown; },) => ParkAnswer;
  },
): Promise<EntryOutcome> {
  /**
   When this entry began, on the monotonic clock, which its own hard cap counts from.
   */
  const startedAt = monotonicMs();

  /**
   Slice overlap the pass read, logged for this entry and shared by every
   per-slice driver.
   */
  const entrySlices = entryOverlap({
    entryId: entry.id,
    setting: overlap,
  },);

  /**
   Per-entry slice-cache directory; earlier runs' finished slices live here so
   a large document resumes instead of restarting.
   */
  const entryCacheDir = join(
    sliceCacheDir,
    entry.id,
  );

  /**
   Whether this entry reached its artifact.
   */
  const outcome = await runEntryPipeline({
    // THE SEAT READER'S BOUND INSIDE THIS ENTRY is the smaller of the pass's and what the entry's own cap has left.
    client: withEntryBound({
      client,
      hardCapMs,
      startedAt,
      now: monotonicMs,
    },),
    entry,
    artifactsDir,
    publishDir,
    declinedDir,
    entryCacheDir,
    tip,
    pipelineDigest,
    pin,
    hardCapMs,
    baseSignal,
    overlap: entrySlices,
    outsideReads,
    pictureSources,
    stageSettings,
    ...((visualEvidenceReader === undefined) ? {} : { visualEvidenceReader, }),
    ...((endRun === undefined) ? {} : { endRun, }),
    ...((parked === undefined) ? {} : { parked, }),
  },);
  if (outcome.kind !== 'settled') {
    // The cache is what makes operational resume cheaper. Stopped quality work
    // also keeps evidence but does not earn another whole-entry attempt.
    return outcome;
  }

  await retireSettledEntryCache({
    entryId: entry.id,
    dir: entryCacheDir,
  },);
  return outcome;
}

//endregion Pass entry settlement
