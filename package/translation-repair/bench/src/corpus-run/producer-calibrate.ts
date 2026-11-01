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
} from '@monochromatic-dev/translation-repair-run/ts';
import { runProducerCalibrate, } from './producer-calibrate-run.ts';
import { readStageSettings, } from '@monochromatic-dev/translation-repair-stage/ts';

//region Producer calibrate
// The producer calibration, as a command: the wiring only. Which models write
// and why is in `producer-calibrate-run.ts`; what one slice does is in
// `producer-calibrate-round.ts`. This file names what only a real process has:
// the run client, the pinned corpus and the repository's head.

/**
 Hands the real process's client, corpus and head to the calibration.

 @param line - the calibration's command line, read whole by `reportingRefusals`

 @example
 ```ts
 await main({ line, },);
 ```
 */
function main({ line, }: { readonly line: CommandLineOf<'producer-calibrate'>; },): Promise<void> {
  /**
   Corpus clone and commit this run reads, after any environment
   override, read inside the refusal boundary (ledger B302).
   */
  const { pin, } = readRunCorpusPinSetting({ env: process.env, },);
  return runProducerCalibrate({
    line,
    newClient: createProbeRunClient,
    drawSlices: function drawPinned({ count, }: { readonly count: number; },): Promise<readonly BenchSlice[]> {
      // The slices every model writes are drawn from the pin read here.
      return sampleBenchSlices({
        count,
        pin,
      },);
    },
    readHead: readHeadSha,
    stageSettings: readStageSettings({ env: process.env, },),
  },);
}

if (import.meta.main)
  await reportingRefusals({
    what: 'producer-calibrate',
    argv: process.argv,
    env: process.env,
    run: main,
  },);

//endregion Producer calibrate
