import { sampleBenchSlices, } from './bench-sample.ts';
import {
  type CommandLineOf,
  createProbeRunClient,
  readHeadSha,
  readRunCorpusPinSetting,
  reportingRefusals,
  resolveRunsDir,
} from '@monochromatic-dev/translation-repair-run/ts';
import { widthControlHolds, } from './editor-width-control.ts';
import { gatherWidthInput, } from './editor-width-input.ts';
import { runEditorWidthProbe, } from './editor-width-probe-run.ts';
import { writeWidthReport, } from './editor-width-report.ts';
import { runWidthSlice, } from './editor-width-slice.ts';
import { readStageSettings, } from '@monochromatic-dev/translation-repair-stage/ts';

//region Editor width probe
// Does seating more EDITORS buy a better repair, with the judging panel held
// fixed: the narrow arm seats what the run configuration seats as editors, the
// wide arm the whole roster, and the command writes its report into the runs
// directory. `editor-width-probe-run.ts` holds the whole account.
//
// SPENDS QUOTA. Point `TRANSLATION_REPAIR_RUNS_DIR` at a throwaway directory.

/**
 Hands the run what only a real process has: the way to build the client, the
 pinned corpus, the commit read, the stages that ask a model or write a report,
 and the runs directory the environment names, resolved before any client is
 built.

 @param line - the probe's command line, read whole by `reportingRefusals`

 @example
 ```ts
 await main({ line, },);
 ```
 */
async function main({ line, }: { readonly line: CommandLineOf<'editor-width-probe'>; },): Promise<void> {
  /**
   Corpus clone and commit this run reads, after any environment
   override, read inside the refusal boundary (ledger B302).
   */
  const { pin, } = readRunCorpusPinSetting({ env: process.env, },);
  await runEditorWidthProbe({
    line,
    newClient: createProbeRunClient,
    drawSample: sampleBenchSlices,
    pin,
    readHeadSha,
    controlHolds: widthControlHolds,
    gather: gatherWidthInput,
    runSlice: runWidthSlice,
    writeReport: writeWidthReport,
    runsDir: await resolveRunsDir(),
    stageSettings: readStageSettings({ env: process.env, },),
  },);
}

if (import.meta.main)
  await reportingRefusals({
    what: 'editor-width-probe',
    argv: process.argv,
    env: process.env,
    run: main,
  },);

//endregion Editor width probe
