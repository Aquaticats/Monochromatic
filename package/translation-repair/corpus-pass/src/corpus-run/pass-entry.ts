import { tagged, } from '@monochromatic-dev/module-logger/ts';

import { armCallDeadline, } from '@monochromatic-dev/translation-repair-provider/ts';
import { monotonicMs, } from '@monochromatic-dev/translation-repair-markup/ts';
import { repairClaimEvidenceOf, } from '@monochromatic-dev/translation-repair-archive/ts';
import {
  type CorpusPair,
  entryArchiveOriginalOf,
  type EntryOutcome,
  recordEntryDecline,
  settledTallyLine,
  tallyCaughtEntry,
} from '@monochromatic-dev/translation-repair-slice-cache/ts';
import type { RunClient, } from '@monochromatic-dev/translation-repair-run/ts';
import { runPassPreparation, } from './pass-prepare-reseat.ts';
import { frontMatterSliceIndexes, } from '@monochromatic-dev/translation-repair-slice-reading/ts';
import { settledPageArtifact, } from './pass-page-assembly.ts';
import { assertPageGuards, } from './pass-page-guards.ts';
import { projectLanes, } from '@monochromatic-dev/translation-repair-artifact/ts';
import { runPassContest, } from './pass-contest.ts';
import { foldPassCarried, } from './pass-carried-fold.ts';
import { admitPassInsertions, } from './pass-insertion-reseat.ts';
import { runPassSettlement, } from './pass-settlement.ts';
import type { PipelineDigest, } from '@monochromatic-dev/translation-repair-glossary/ts';
import {
  persistSettledEntry,
  printSettledLines,
} from './pass-entry-persist.ts';
import { unfilledPageFindings, } from '@monochromatic-dev/translation-repair-page-check/ts';
import { createPassPictureReader, } from './pass-seated-pictures.ts';
import type { PassOutsideReads, } from './pass-outside-reads.ts';
import type { StageSettings, } from '@monochromatic-dev/translation-repair-stage/ts';
import type {
  PassPictureSources,
  PassVisualEvidenceReader,
} from './pass-visual-evidence.ts';
import { openEntryCaches, } from './pass-entry-caches.ts';
import { readLanesSeats, } from './pass-reseat.ts';
import { runPassLanes, } from './pass-lanes.ts';
import {
  type CorpusPin,
  preparedSheetFactsOf,
} from '@monochromatic-dev/translation-repair-corpus-source/ts';

//region Pass entry

// ONE entry, start to settled: its cache, its deadline, its artifact and its
// TALLY line.
//
// Split from `corpus-pass.ts` at the line cap, on the seam between SCHEDULING
// and SETTLING. What survives in the pass is the part that decides which entry
// runs next and when to stop starting them; what moved here is everything that
// happens once an entry has been chosen, and that part never reads the
// scheduler's state.
//
// The seam matters beyond the line count: an entry either settles completely or
// records an ERROR, and nothing between those two outcomes is allowed to reach
// the artifacts directory. Keeping the whole of that in one function is what
// makes the rule checkable by reading, since the write is the last thing the
// success path does and the failure path has no write at all.

/**
 Runs one chosen entry as far as its artifact, and says whether it got there.

 SEPARATE FROM THE CACHE DISCARD that follows it, which is the whole reason
 this function exists rather than one longer body. The discard is destructive
 and belongs only to the settled path, and a `catch` wide enough to cover both
 cannot tell a pipeline failure from a failed unlink.

 @param client - shared model client

 @param entry - corpus pair to settle, text already read

 @param artifactsDir - directory one JSON per settled entry is written into

 @param publishDir - root of the mirrored corpus tree each fixed page is written into

 @param declinedDir - directory a decline record is written into when the
 archive's note says the whole page is the author's own English

 @param entryCacheDir - this entry's own cache directory

 @param tip - repository head recorded into the artifact

 @param pipelineDigest - identity of the built pipeline, which also generation
 -stamps the slice cache so a changed pipeline cannot resume foreign slices

 @param pin - corpus clone and commit the entry was read at, whose commit the
 artifact or decline record names and whose clone the pictures are read from

 @param hardCapMs - time this entry may run before its exchanges abort

 @param baseSignal - abort this entry's deadline forwards from

 @param overlap - most slices each per-slice driver keeps in flight

 @param outsideReads - what preparation reads from outside the pipeline

 @param pictureSources - where the entry's picture bytes and their OCR text come from

 @param stageSettings - straggler windows and contest ledger the entry file
 read for this launch

 @param endRun - told of the failure that ended the entry when it ends the run (`tallyCaughtEntry` says when), so the
 pass stops however the failure reached the entry

 @param parked - asked after `endRun`: whether the pass is now parked for the failure and the entry goes back to the
 front of its queue, which the TALLY line prints as `PARKED`

 @returns Whether an artifact was written

 @example
 ```ts
 const outcome = await runEntryPipeline({ client, entry, artifactsDir, entryCacheDir, ... },);
 ```
 */
export async function runEntryPipeline(
  {
    client,
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
    overlap,
    outsideReads,
    pictureSources,
    visualEvidenceReader,
    stageSettings,
    endRun,
    parked,
  }: {
    readonly client: RunClient;
    readonly entry: CorpusPair;
    readonly artifactsDir: string;
    readonly publishDir: string;
    readonly declinedDir: string;
    readonly entryCacheDir: string;
    readonly tip: string;
    readonly pipelineDigest: PipelineDigest;
    readonly pin: CorpusPin;
    readonly hardCapMs: number;
    readonly baseSignal: AbortSignal;
    readonly overlap: number;
    readonly outsideReads: PassOutsideReads;
    readonly pictureSources: PassPictureSources;
    readonly visualEvidenceReader?: PassVisualEvidenceReader;
    readonly stageSettings: StageSettings;
    readonly endRun?: (args: { readonly error: unknown; },) => void;
    readonly parked?: (args: { readonly error: unknown; },) => boolean;
  },
): Promise<EntryOutcome> {
  /**
   Start time of this entry, for its duration.
   */
  const t0 = monotonicMs();

  /**
   Per-entry hard-cap deadline. Disposal at return defuses the timer and
   detaches its listener; the repo bans try/finally, so cleanup rides on
   `using` instead.

   ARMED BEFORE THE CACHE OPENS rather than after. Opening reads and may
   discard a directory of settled slices, and on a large entry that is real
   time; a ceiling armed afterwards would not be counting it, so the cap
   would mean something slightly different for a resumed entry than for a
   fresh one.
   */
  using deadline = armCallDeadline({
    signal: baseSignal,
    timeoutMs: hardCapMs,
    label: entry.id,
  },);
  try {
    /**
     What the archive's translators' notes say about whose text the page is,
     read BEFORE ANY PURCHASE: a page the note calls the author's own English
     is declined here, and nothing else in this function runs for it (the owner's rule of
     2026-09-08, `pass-decline.ts`).
     */
    const archiveOriginal = entryArchiveOriginalOf({
      entry,
      l: tagged({ tag: entry.id, },),
    },);
    if (archiveOriginal.kind === 'whole-page')
      return await recordEntryDecline({
        entry,
        declinedDir,
        publishDir,
        tip,
        pipelineDigest,
        corpusSha: pin.commitSha,
        note: archiveOriginal.note,
        startedAt: t0,
      },);

    /**
     The four stores this entry opens before its lanes run
     (`pass-entry-caches.ts`); the pairing, contest and consolidation stores
     open where those stages start.

     INSIDE the try, because opening touches the filesystem and can fail.
     Opened outside, one unreadable cache directory ended the whole pass at
     whatever entry happened to hold it, which is the opposite of this
     function's contract.
     */
    const {
      sliceCache,
      translateSliceCache,
      refineSliceCache,
      readingCache,
    } = await openEntryCaches({
      entryCacheDir,
      pipelineDigest,
    },);

    /**
     Entry-scoped evidence reader shared by archive review and final slices.
     */
    const readPictures = createPassPictureReader({
      client,
      entryId: entry.id,
      pin,
      cache: readingCache,
      signal: deadline.callSignal,
      l: tagged({ tag: entry.id, },),
      pictureSources,
      ...((visualEvidenceReader === undefined) ? {} : { visualEvidenceReader, }),
    },);

    /**
     Slicing BOTH lanes run over, prepared once here rather than inside
     either.

     That is the entire reason the driver exists: one slicing, one alignment
     and one identity block mean a difference between the two documents is a
     difference between the LANES rather than between two runs of the aligner.

     No slice budget is passed, and that is checked rather than assumed:
     `prepareDocumentPair` defaults to the same `SLICE_CHAR_BUDGET` that
     `repairTranslation` passed down when it did this itself, so entries
     settled before and after this change were sliced the same way.

     THE ROSTER DECIDES WHICH PARAGRAPH RENDERS WHICH, per
     `doc/decision/llm-assisted-block-pairing.md`, because the deterministic
     scorer is exhausted on this corpus: block kind is constant across
     paragraphs, Chinese and English prose share no Latin tokens, and length
     alone reached four correct pairings in eight on `saurikissa` and went no
     further at any weight. Six of eleven slices there paired unrelated
     paragraphs, and every stage downstream then behaved correctly on wrong
     input. A section the roster cannot pair keeps the scorer and says so.

     THE SEATS ARE READ INSIDE (`pass-prepare-reseat.ts`), off the meters
     first of all, and every stage that asks the roster re-seats under a hold
     (ledger X12).

     EVERY PREPARATION FINDING RIDES `paired.alignmentFindings` into the
     artifact (ledger B54); a list returned beside it went unread here.
     */
    const { prepared: paired, } = await runPassPreparation({
      client,
      entryId: entry.id,
      entryCacheDir,
      pipelineDigest,
      readPictures,
      sourceText: entry.sourceText,
      // Normalized inside (`pass-prepare.ts`): the archive both deciders judge
      // is the archive as prepared, never these bytes.
      targetText: entry.targetText,
      signal: deadline.callSignal,
      outsideReads,
      stageSettings,
    },);
    /**
     Archive after preparation-stage review corrections.
     */
    const settledArchiveText = paired.targetText;

    /**
     Complete reviewed visual evidence required before insertion and lanes,
     read by the readers the meters seat (`pass-seated-pictures.ts`): a model
     withheld on the provider that would serve it reads no picture either.
     */
    const pictureReadings = await readPictures({ slices: paired.slices, },);

    /**
     The lanes' judge benches, read off Synthetic's meter (`run-seats.ts`).
     The contest and the consolidation seams read their own: XIEPT2 on
     2026-09-03 ran Synthetic dry seven minutes into a 219-minute entry.
     */
    const lanesSeating = await readLanesSeats({
      client,
      signal: deadline.callSignal,
      entryId: entry.id,
    },);

    /**
     Semantic and deterministic proof for every source-only slice.

     BOUGHT BEFORE THE LANES so known omission is licensed for translation or
     unresolved placement pauses stage as incomplete. Coverage roster searches
     whole target, independent of pairing;
     page shortfall or a missing destination supplies second corroboration.
     */
    const admissionAsRead = await admitPassInsertions({
      client,
      prepared: paired,
      modelIds: lanesSeating.seats
        .roster,
      overlap,
      signal: deadline.callSignal,
      entryId: entry.id,
      stageSettings,
    },);

    /**
     The slicing with every carried passage folded into the neighbour whose
     archive span renders it (class one hundred ten, mikaela12): both lanes
     then write that neighbour from the source it actually renders, instead
     of dropping the passage the archive merged in. The fold's lines print
     under the entry (`pass-carried-fold.ts`).
     */
    const {
      prepared,
      admission: translateInsertionAdmission,
    } = foldPassCarried({
      paired,
      admission: admissionAsRead,
      l: tagged({ tag: entry.id, },),
    },);

    /**
     What both lanes made of that slicing, with neither preferred, on the
     benches and re-seat hooks the lanes reading supplied (`pass-lanes.ts`,
     the seam a test drives: ledger X14).
     */
    const lanes = await runPassLanes({
      client,
      prepared,
      lanesSeating,
      pictureReadings,
      signal: deadline.callSignal,
      overlap,
      repairSliceCache: sliceCache,
      refineSliceCache,
      translateSliceCache,
      translateInsertionAdmission,
      entryId: entry.id,
      stageSettings,
    },);

    // AFTER the lanes return and BEFORE anything is written, which is the only
    // place this check belongs. Both drivers deliberately let a fully cached
    // lane finish after an abort, so a gate BETWEEN the lanes would discard
    // work already bought; and an entry whose ceiling fired mid-document has a
    // half-run to report, not an artifact.
    deadline.callSignal
      .throwIfAborted();

    // A RECORDED GAP, NOT A REFUSAL. Under the no-loop design of 2026-09-01 a
    // source passage still unfilled after the single round and its one
    // follow-up ships as a gap the artifact records (`lanes.translate.unfilled`
    // and the lane's findings), because an insertion is recovered supplementary
    // content whose absence is a recorded gap, not a missing required page. The
    // refusal that stood here (2026-08-27) dropped XIEPT2 after 35 minutes on
    // 2026-09-02 over one passage two judge rounds tied on, and an earlier
    // XIEPT2 attempt after four hours forty-eight minutes.
    /**
     Source passages translation could not fill, each named in the log.
     */
    const { unfilled, } = lanes.translate;
    for (const finding of unfilledPageFindings({ unfilled, },)) {
      tagged({ tag: entry.id, },)
        .warn(`entry ${entry.id}: ${finding}`,);
    }

    /**
     Both ledgers as version 2 rows, beside the comparison they derive.

     DERIVED HERE AND HANDED TO THE BUILDER`S OWN CALL rather than passed
     along, because the builder derives it again from the same function. The
     contest needs it first, to know which slices are worth asking about.
     */
    const projected = projectLanes({ lanes, },);

    /**
     Syntax-bearing metadata slice indexes shared by final quality stages.
     */
    const frontMatterSlices = frontMatterSliceIndexes({
      slices: prepared.slices,
    },);

    /**
     What the roster said at every slice the two lanes worded differently.
     */
    const contestSlices = await runPassContest({
      client,
      lanes,
      projected,
      frontMatterSlices,
      lineStructuredSlices: prepared.lineStructuredSliceIndices,
      sheetFacts: preparedSheetFactsOf({ prepared, },),
      ...((prepared.declaredNamePairs === undefined)
        ? {}
        : { declaredNamePairs: prepared.declaredNamePairs, }),
      entryCacheDir,
      pipelineDigest,
      signal: deadline.callSignal,
      overlap,
      l: tagged({ tag: entry.id, },),
      stageSettings,
    },);

    /**
     Third rendering and final naturalness decisions for contested slices, then
     the page-level consistency of the settled slices (`pass-settlement.ts`):
     repeated source wording and refrains the slices render differently, put to
     the judge bench and carried to one rendering (the owner's ruling of
     2026-10-11). BEFORE THE PAGE IS COMPOSED, so the page passes and the page
     guards run over its result. A run always ships: trouble in the consistency
     pass leaves the page as it stood and says so in the artifact.
     */
    const settlement = await runPassSettlement({
      client,
      prepared,
      projected,
      contests: contestSlices,
      // THE SAME READING THE LANES MADE, so the consolidation's stand-in is
      // the text the translate lane judged over (class one hundred seven),
      // and beside it the claims the repair panel accepted, disputing the
      // archive or not, for the polish gate that decides a rewording of the
      // consolidated text (CuspariaKLSY21 slice 0, 2026-10-21).
      ...repairClaimEvidenceOf({ repair: lanes.repair, },),
      frontMatterSlices,
      pictureReadings,
      entryCacheDir,
      pipelineDigest,
      signal: deadline.callSignal,
      overlap,
      l: tagged({ tag: entry.id, },),
      stageSettings,
    },);

    /**
     Time this entry took, both lanes, the contest, the consolidation and the consistency pass included, on `monotonicMs` (ledger B78).
     */
    const durationMs = monotonicMs() - t0;

    /**
     Rich artifact for later grading (`pass-entry-artifact.ts`).
     */
    const artifact = settledPageArtifact({
      entryId: entry.id,
      tip,
      pipelineDigest,
      corpusSha: pin.commitSha,
      durationMs,
      prepared,
      lanes,
      contestSlices,
      ...settlement,
      targetText: settledArchiveText,
      l: tagged({ tag: entry.id, },),
    },);

    // This entry's TALLY line, read off the artifact BEFORE it is written.
    //
    // NOT INLINED INTO THE `console.log` THAT PRINTS IT, which is where it sat until
    // 2026-08-22 and where the obvious tidying would put it back. The line asks
    // what each slice would carry, and that question raises
    // `UnansweredContestSliceError` on a document whose lanes differ at a slice
    // the contest names nowhere. Raised after the write, that lands in the entry's
    // `catch`, which prints `status=ERROR` for an entry whose complete artifact is
    // already on disk: every later reader would then find a settled file the
    // pass reported as failed.
    //
    // Asking first makes the refusal truthful. A contest that cannot account for
    // a slice it was obliged to decide has not settled the document, and no
    // artifact should claim it did; the stage caches still hold every answer, so
    // a re-run reproduces the contradiction rather than losing it.
    assertPageGuards({
      artifact,
      slices: prepared.slices,
      targetText: settledArchiveText,
      carried: translateInsertionAdmission.carried ?? [],
    },);
    /**
     Tally proven readable before any persistence.
     */
    const tally = settledTallyLine({ artifact, },);

    // BEFORE THE ARTIFACT, for the same reason the tally line is read before it,
    // and for one more. A pass builds its skip set from the artifacts already on
    // disk, so an entry whose artifact exists is never attempted again. Written
    // second, a crash between the two writes would leave that entry done forever
    // with no page ever produced; written first, every entry the pass calls
    // settled has its page, by construction rather than by luck.
    printSettledLines({
      entryId: entry.id,
      tally,
      published: await persistSettledEntry({
        artifact,
        slices: prepared.slices,
        archiveText: settledArchiveText,
        sourceText: entry.sourceText,
        entryId: entry.id,
        publishDir,
        artifactsDir,
        l: tagged({ tag: entry.id, },),
        ...((prepared.archiveOriginalSpans === undefined)
          ? {}
          : { archiveOriginalSpans: prepared.archiveOriginalSpans, }),
      },),
    },);
    return { kind: 'settled', };
  }
  catch (error) {
    return tallyCaughtEntry({
      entryId: entry.id,
      error,
      durationMs: monotonicMs() - t0,
      aborted: deadline.callSignal
        .aborted,
      ...((endRun === undefined) ? {} : { endRun, }),
      ...((parked === undefined) ? {} : { parked, }),
    },);
  }
}

//endregion Pass entry
