import {
  type CommandLineOf,
  createProbeRunClient,
  readHeadSha,
  readRunCorpusPinSetting,
  reportingRefusals,
  resolveRunsDir,
} from '@monochromatic-dev/translation-repair-run/ts';
import { nowAsIso, } from './probe-run-clock.ts';
import { runRecallBenchmark, } from './recall-benchmark-run.ts';
import { readStageSettings, } from '@monochromatic-dev/translation-repair-stage/ts';

//region Recall benchmark
// The recall benchmark, as a command: the wiring only. How entries are chosen
// and seeded is in `recall-benchmark-choose.ts` and `recall-benchmark-entry.ts`,
// what is printed in `recall-benchmark-report.ts`, and the run itself in
// `recall-benchmark-run.ts`. This file names what only a real process has: the
// runs directory, the pinned corpus, the repository's head, the run client
// and the clock.
//
// Run with `mise run //package/translation-repair/probe:recall-benchmark`
// (append `-- --plan` for a zero-quota setup check).

/**
 Hands the real process's places and client to the benchmark.

 @param line - the benchmark's command line, read whole by `reportingRefusals`

 @example
 ```ts
 await main({ line, },);
 ```
 */
async function main({ line, }: { readonly line: CommandLineOf<'recall-benchmark'>; },): Promise<void> {
  /**
   Corpus clone and commit this run reads, after any environment
   override, read inside the refusal boundary (ledger B302).
   */
  const { pin, } = readRunCorpusPinSetting({ env: process.env, },);
  return runRecallBenchmark({
    line,
    runsDir: await resolveRunsDir(),
    readTip: readHeadSha,
    pin,
    newClient: createProbeRunClient,
    now: nowAsIso,
    stageSettings: readStageSettings({ env: process.env, },),
  },);
}

if (import.meta.main)
  await reportingRefusals({
    what: 'recall-benchmark',
    argv: process.argv,
    env: process.env,
    run: main,
  },);

//endregion Recall benchmark
