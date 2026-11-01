import type {
  CorpusPin,
  readCorpusFile,
} from '@monochromatic-dev/translation-repair-corpus-source/ts';
import { StatedRefusalError, } from '@monochromatic-dev/translation-repair-glossary/ts';
import {
  assertRepairMeasurable,
  countUnrecordedRepairs,
  DEFAULT_SAMPLE_SEED,
  DEFAULT_SAMPLE_SIZE,
  drawStratifiedSample,
} from '@monochromatic-dev/translation-repair-house-rule/ts';
import type { PoolPolicy, } from '@monochromatic-dev/translation-repair-artifact-pool/ts';
import type { CommandLineOf, } from '@monochromatic-dev/translation-repair-run/ts';
import { poolBandLines, } from './draw-sample-bands.ts';
import { readDrawPool, } from './draw-sample-pool.ts';
import { writeDrawSheets, } from './draw-sample-sheets.ts';

//region Draw sample run
// Reads every settled artifact, bands each entry by its zh source bytes,
// flattens accepted issues into grading candidates, and draws the stratified
// precision sample into a grading sheet written OUTSIDE the repo (the sheet
// quotes UNLICENSED corpus text). Default run is PRELIMINARY validation over
// whatever has settled; pass `--final` once every band has enough CONTRIBUTING
// entries to spread the draw, and only once the pass writing artifacts has
// stopped, since a live run can add one after the directory is read. Readiness
// is judged on contributing entries rather than accepted counts because the
// draw round-robins across entries. A preliminary run draws with a different
// seed on purpose, so repeated previews can never become a way of choosing the
// gate sample. The reconcile in `draw-entry-load.ts` aborts loudly if a parsed accepted count
// disagrees with the artifact's own tally, so the sample is never silently
// short.

/**
 Draws the stratified precision sample and writes the grading sheet outside
 the repo. `--final` writes the gate sheet, otherwise a labelled preliminary
 sheet.

 @param line - the draw's command line, read whole by `reportingRefusals`

 @param runsDir - durable, gitignored output root

 @param pin - corpus pin each entry's source is read at and whose commit the
 sheets record

 @param readSource - reads a corpus file at the pin: `readCorpusFile` in a run

 @param policy - generation policy the entry read from its environment
 (`readPoolPolicy`), handed to the pool

 @param repository - checkout a required commit is resolved in and every
 recorded commit is placed against, which the entry names: the package's own
 in a run, a throwaway one in a case

 @throws {@link StatedRefusalError} When a final draw's sample is empty, or
 carries issues with no recorded repair, the count of them named, and for what
 the pool and the sheet paths refuse with

 @example
 ```ts
 await drawGradingSample({ line, runsDir, pin, readSource: readCorpusFile, policy, repository, },);
 ```
 */
export async function drawGradingSample(
  {
    line,
    runsDir,
    pin,
    readSource,
    policy,
    repository,
  }: {
    readonly line: CommandLineOf<'draw-sample'>;
    readonly runsDir: string;
    readonly pin: CorpusPin;
    readonly readSource: typeof readCorpusFile;
    readonly policy: PoolPolicy;
    readonly repository: string;
  },
): Promise<void> {
  /**
   Whether this run writes the final gate sheet rather than a preliminary one.
   */
  const isFinal = line.switched('final',);

  /**
   Seed the DRAW uses, which is deliberately NOT the gate seed on a
   preliminary run.

   A preliminary draw exists to check that the sheets render and that the pool
   reconciles, and it is run repeatedly while the pool grows. Drawing it with
   the gate seed would make each one a preview of the gate sample over the
   pool of the moment, and choosing when to finalize after seeing those
   previews is selecting the sample on its contents. The file naming still
   keys on {@link DEFAULT_SAMPLE_SEED} so one round cannot target another
   round's path; only the shuffle differs.
   */
  const drawSeed = isFinal
    ? DEFAULT_SAMPLE_SEED
    : `${DEFAULT_SAMPLE_SEED}-preliminary`;

  /**
   What the runs directory holds that this draw may sample.
   */
  const {
    eligible,
    names,
    entries,
    pool,
  } = await readDrawPool({
    runsDir,
    pin,
    readSource,
    policy,
    repository,
  },);

  for (const bandLine of poolBandLines({ entries, },))
    console.log(bandLine,);

  /**
   The drawn stratified sample.
   */
  const sample = drawStratifiedSample({
    candidates: pool,
    size: DEFAULT_SAMPLE_SIZE,
    seed: drawSeed,
  },);

  /**
   Sampled items carrying no recorded repair at all, which is what a draw over
   pre-recording artifacts looks like.
   */
  const unrecorded = countUnrecordedRepairs({ sample, },);

  /**
   Pool-wide candidates carrying no recorded repair.

   Reported because {@link assertRepairMeasurable} only inspects what was
   DRAWN, so pre-recording candidates left in the pool escape it whenever the
   seed happens not to select them. Seeing the pool figure says whether a
   clean sample means a clean pool or a lucky draw, and it is the number that
   predicts whether the final draw will abort.
   */
  const unrecordedPool = countUnrecordedRepairs({ sample: pool, },);
  if (isFinal && (sample.length === 0)) {
    // AN EMPTY GATE SHEET IS NOT HARMLESS: a final sheet is created exclusively
    // and then refused for ever, so one with no item in it would stand in the
    // real draw's way, and `assertRepairMeasurable` passes an empty sample.
    throw new StatedRefusalError({
      says: 'refusing a final draw: the settled entries hold no accepted issue to sample, and a gate sheet with no '
        + 'item would be protected from overwrite and block the real draw. Settle more entries, or draw without '
        + '--final to check the pool.',
    },);
  }

  // THE REFUSAL THIS RAISES SAYS WHAT TO DO NEXT, and its class
  // (`UnmeasurableRepairError`) is a stated refusal, so the command reports
  // it as declined: a fault report would send an operator looking for a bug
  // in the command.
  if (isFinal)
    assertRepairMeasurable({ sample, },);

  /**
   Where the three files of this draw landed.
   */
  const {
    outPath,
    repairPath,
    manifestPath,
  } = await writeDrawSheets({
    runsDir,
    isFinal,
    drawSeed,
    sample,
    eligible,
    names,
    corpusSha: pin.commitSha,
  },);

  console.log(
    `SAMPLE final=${String(isFinal,)} seed=${drawSeed} pool=${
      String(pool.length,)
    } drawn=${String(sample.length,)} unrecordedRepairs=${
      String(unrecorded,)
    } unrecordedInPool=${String(unrecordedPool,)} out=${outPath} repairOut=${
      repairPath
    } manifest=${manifestPath}`,
  );
}

//endregion Draw sample run
