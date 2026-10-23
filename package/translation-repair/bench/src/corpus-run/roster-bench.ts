import { writeBenchReport, } from './bench-report.ts';
import {
  type BenchSlice,
  sampleBenchSlices,
} from './bench-sample.ts';
import {
  type CommandLineOf,
  createProbeRunClient,
  readHeadSha,
  readRunCorpusPinSetting,
  reportingRefusals,
  resolveRunsDir,
  RUN_ROSTER,
} from '@monochromatic-dev/translation-repair-run/ts';
import { runRosterBench, } from './roster-bench-run.ts';
import { readStageSettings, } from '@monochromatic-dev/translation-repair-stage/ts';

//region Roster bench
// The roster width bench, as a command: the wiring only. What it asks and
// what it prints is in `roster-bench-run.ts`, one slice at one width is in
// `roster-bench-round.ts`, and the row it records is in `roster-bench-row.ts`.
// This file names what only a real process has: the run client, the pinned
// corpus, the roster, the repository's head and the clock.
//
// SPENDS QUOTA. Point `TRANSLATION_REPAIR_RUNS_DIR` at a throwaway directory.

/**
 Hands the real process's client, corpus, roster, clock and runs directory to
 the bench.

 @param line - the bench's command line, read whole by `reportingRefusals`

 @example
 ```ts
 await main({ line, },);
 ```
 */
async function main({ line, }: { readonly line: CommandLineOf<'roster-bench'>; },): Promise<void> {
  /**
   Corpus clone and commit this run reads, after any environment
   override, read inside the refusal boundary (ledger B302).
   */
  const { pin, } = readRunCorpusPinSetting({ env: process.env, },);
  await runRosterBench({
    line,
    roster: RUN_ROSTER,
    newClient: createProbeRunClient,
    drawSlices: function drawPinned({ count, }: { readonly count: number; },): Promise<readonly BenchSlice[]> {
      // The slices every width sees are drawn from the pin read here.
      return sampleBenchSlices({
        count,
        pin,
      },);
    },
    readHead: readHeadSha,
    writeReport: writeBenchReport,
    clock: performance,
    runsDir: await resolveRunsDir(),
    stageSettings: readStageSettings({ env: process.env, },),
  },);
}

if (import.meta.main)
  await reportingRefusals({
    what: 'roster-bench',
    argv: process.argv,
    env: process.env,
    run: main,
  },);

//endregion Roster bench
