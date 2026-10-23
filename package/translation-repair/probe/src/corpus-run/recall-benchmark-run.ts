import { mkdir, } from 'node:fs/promises';

import type { ModelClient, } from '@monochromatic-dev/translation-repair-provider/ts';
import type { CorpusPin, } from '@monochromatic-dev/translation-repair-corpus-source/ts';
import { runRepairBenchmark, } from '@monochromatic-dev/translation-repair-repair/ts';
import { StatedRefusalError, } from '@monochromatic-dev/translation-repair-glossary/ts';
import { persistRecallScorecard, } from './recall-scorecard-store.ts';
import {
  chooseRecallEntries,
  ENTRIES_PER_BAND,
} from './recall-benchmark-choose.ts';
import { SEEDS_PER_ENTRY, } from './recall-benchmark-entry.ts';
import {
  recallPlanLine,
  recallScorecardLines,
  recallStartLine,
  refuseUnmeasuredScorecard,
} from './recall-benchmark-report.ts';
import {
  type CommandLineOf,
  RECALL_JUDGE_MODEL_IDS,
  RUN_CALL_CONFIG,
  RUN_MODELS,
  RUN_PER_CALL_TIMEOUT_MS,
} from '@monochromatic-dev/translation-repair-run/ts';
import type { StageSettings, } from '@monochromatic-dev/translation-repair-stage/ts';

//region Recall benchmark run
// Measures what precision cannot see: of the defects that ARE present, how many
// does the pipeline find, and how many does it actually repair.
//
// Precision is measured on the pipeline's own output, so a pipeline that
// accepts almost nothing scores beautifully and repairs nothing. This runner
// plants known omissions into a clean translation, runs the whole repair loop
// on the seeded pair, and grades restoration against the deletions it made, so
// the denominator is defects that certainly exist rather than defects the
// pipeline chose to report.
//
// Run with `mise run //package/translation-repair/probe:recall-benchmark`
// (append `-- --plan` for a zero-quota setup check).

/**
 Milliseconds in one second.
 */
const MS_PER_SECOND = 1_000;

/**
 Seconds in one minute.
 */
const SECONDS_PER_MINUTE = 60;

/**
 Minutes in one hour.
 */
const MINUTES_PER_HOUR = 60;

/**
 Hours the whole benchmark may run.

 Raised from 4 on run 001's own timing: it settled seven of nine entries in
 252 minutes and recorded the other two as skipped, coverage 0.778. Detection
 has to be re-measured anyway after the slice-index fix, and the rerun also
 carries the ensemble and the naturalness lane, both of which only add wall
 time, so a four-hour budget would lose more than two entries next time.
 Coverage is the thing this protects; the plan is flat rate, so a longer run
 costs nothing but waiting.
 */
const BUDGET_HOURS = 12;

/**
 Wall budget for the whole benchmark; entries the budget cannot fit record as
 skipped and the scorecard reports the resulting coverage honestly.
 */
const RUN_BUDGET_MS = BUDGET_HOURS
  * MINUTES_PER_HOUR
  * SECONDS_PER_MINUTE
  * MS_PER_SECOND;

/**
 Runs the recall benchmark over a band-stratified corpus sample and writes its
 scorecard beside the other run artifacts.

 @param line - the benchmark's command line, read whole by `reportingRefusals`

 @param runsDir - durable, gitignored output root

 @param readTip - reads the pipeline commit recorded into the scorecard

 @param pin - corpus clone and commit the entries are read at

 @param newClient - builds the one client the whole run shares

 @param now - reads the instant as an ISO stamp, which names the scorecard
 file at the start and dates its end

 @param stageSettings - straggler windows and contest ledger the entry file
 read for this launch (`StageSettings`)

 @throws {@link StatedRefusalError} when the
 corpus at the pin yields no entry to seed, with or without `--plan`, before
 any client is built or anything is kept

 @throws {@link StatedRefusalError} when a run
 over chosen entries dispatched none or planted no seed, after keeping its
 record

 @example
 ```ts
 await runRecallBenchmark({
   line,
   runsDir,
   readTip: readHeadSha,
   pin,
   newClient: createProbeRunClient,
   now,
   stageSettings,
 },);
 ```
 */
export async function runRecallBenchmark(
  {
    line,
    runsDir,
    readTip,
    pin,
    newClient,
    now,
    stageSettings,
  }: {
    readonly line: CommandLineOf<'recall-benchmark'>;
    readonly runsDir: string;
    readonly readTip: () => Promise<string>;
    readonly pin: CorpusPin;
    readonly newClient: () => ModelClient;
    readonly now: () => string;
    readonly stageSettings: StageSettings;
  },
): Promise<void> {
  /**
   When this run began, which names its scorecard file.
   */
  const startedAt = now();

  /**
   Pipeline tip recorded into the scorecard.
   */
  const tip = await readTip();

  /**
   Seeded entries chosen per band, and each band's count.
   */
  const choice = await chooseRecallEntries({ pin, },);

  /**
   Entries the benchmark will seed, in corpus order.
   */
  const { chosen, } = choice;
  console.log(recallStartLine({
    tip,
    choice,
    budgetMs: RUN_BUDGET_MS,
  },),);

  // A RUN OVER NOTHING IS REFUSED HERE, WITH OR WITHOUT `--plan`, before a
  // client is built or anything is kept. A corpus that yields no entry is a
  // wrong clone or commit, and a run over it would plant no seed and measure
  // nothing. Until 2026-10-06 only the plan refused here; the run built a
  // client, ran nothing, kept an empty scorecard and refused after it.
  if (chosen.length === 0)
    throw new StatedRefusalError({
      says: 'no entry at the pin can be seeded, so the benchmark would plant no seed and measure nothing; '
        + 'check that the corpus clone and commit it reads hold entries with both pages and a sentence '
        + 'worth deleting',
    },);
  await mkdir(
    runsDir,
    { recursive: true, },
  );

  /**
   Shared client using measured production provider concurrency.
   */
  const client = newClient();

  if (line.switched('plan',)) {
    console.log(recallPlanLine({
      tip,
      chosen,
    },),);
    return;
  }

  /**
   Graded attempts and the aggregate scorecard.
   */
  const {
    records,
    scorecard,
  } = await runRepairBenchmark({
    client,
    entries: chosen,
    models: RUN_MODELS,
    judgeModelIds: RECALL_JUDGE_MODEL_IDS,
    signal: new AbortController().signal,
    perCallTimeoutMs: RUN_PER_CALL_TIMEOUT_MS,
    runBudgetMs: RUN_BUDGET_MS,
    stageSettings,
  },);

  /**
   Where the scorecard was kept: a stamped name of its own, written
   atomically, so a rerun sits beside the run it is compared against rather
   than over it.
   */
  const keptAt = await persistRecallScorecard({
    runsDir,
    record: {
      startedAt,
      finishedAt: now(),
      tip,
      corpusSha: pin.commitSha,
      callConfig: RUN_CALL_CONFIG,
      entriesPerBand: ENTRIES_PER_BAND,
      seedsPerEntry: SEEDS_PER_ENTRY,
      scorecard,
      records,
    },
  },);

  refuseUnmeasuredScorecard({
    scorecard,
    keptAt,
  },);

  for (const scorecardLine of recallScorecardLines({
    scorecard,
    keptAt,
  },)) {
    console.log(scorecardLine,);
  }
}

//endregion Recall benchmark run
