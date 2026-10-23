import { readContestLedger, } from '@monochromatic-dev/translation-repair-house-rule/ts';
import {
  adoptCalibrationGrace,
  CALIBRATION_STRAGGLER_GRACE_MS,
  type CalibrationGrace,
  readWriterGrace,
  type WriterGrace,
} from '@monochromatic-dev/translation-repair-glossary/ts';
import { sampleBenchSlices, } from './bench-sample.ts';
import {
  type CommandLineOf,
  createProbeRunClient,
  readRunCorpusPinSetting,
  reportingRefusals,
} from '@monochromatic-dev/translation-repair-run/ts';
import { runEditorCalibrate, } from './editor-calibrate-run.ts';
import { readOverlap, } from '@monochromatic-dev/translation-repair-slice-cache/ts';

//region Editor calibrate
// Which of the roster should edit, measured on the editor's own job: every
// model edits and judges every slice drawn from the pinned corpus, and the
// command prints the editor standing, the refiner standing and what shipped
// with no vote behind it. `editor-calibrate-run.ts` holds the whole account.
//
// SPENDS QUOTA. Point `TRANSLATION_REPAIR_RUNS_DIR` at a throwaway directory.

/**
 Reads how many slices may be in flight from this process's environment.

 @param fallback - slices in flight when the environment says nothing

 @returns Valid overlap

 @throws StatedRefusalError when the variable is not a whole number written in
 digits, or is below one

 @example
 ```ts
 const overlap = overlapFromEnvironment({ fallback: CALIBRATION_OVERLAP, },);
 ```
 */
function overlapFromEnvironment({ fallback, }: { readonly fallback: number; },): number {
  return readOverlap({
    fallback,
    env: process.env,
  },);
}

/**
 Adopts the calibration's straggler window from this process's environment.

 @returns Window the calibration's rounds run under, and where it came from

 @throws StatedRefusalError when a launch set the variable to something no
 dial can read

 @example
 ```ts
 const grace = calibrationGraceFromEnvironment();
 ```
 */
function calibrationGraceFromEnvironment(): CalibrationGrace {
  return adoptCalibrationGrace({ env: process.env, },);
}

/**
 Reads both windows the calibration's lane runs under from this process's
 environment, the round window with the calibration's own fallback.

 @returns Both windows and the writers' source

 @throws StatedRefusalError when either dial is set to something no dial can
 read

 @example
 ```ts
 const windows = windowsFromEnvironment();
 ```
 */
function windowsFromEnvironment(): WriterGrace {
  return readWriterGrace({
    env: process.env,
    roundFallback: CALIBRATION_STRAGGLER_GRACE_MS,
  },);
}

/**
 Hands the run what only a real process has: the overlap dial from its
 environment, the pinned corpus, the two straggler windows and the client,
 each a reader over this process's environment the run calls where the
 command reads it, and the contest ledger, read here.

 @param line - the calibration's command line, read whole by `reportingRefusals`

 @example
 ```ts
 await main({ line, },);
 ```
 */
function main({ line, }: { readonly line: CommandLineOf<'editor-calibrate'>; },): Promise<void> {
  /**
   Corpus clone and commit this run reads, after any environment
   override, read inside the refusal boundary (ledger B302).
   */
  const { pin, } = readRunCorpusPinSetting({ env: process.env, },);
  return runEditorCalibrate({
    line,
    readOverlap: overlapFromEnvironment,
    drawSample: sampleBenchSlices,
    pin,
    adoptGrace: calibrationGraceFromEnvironment,
    readWriterGrace: windowsFromEnvironment,
    newClient: createProbeRunClient,
    contestLedger: readContestLedger({ env: process.env, },),
  },);
}

if (import.meta.main)
  await reportingRefusals({
    what: 'editor-calibrate',
    argv: process.argv,
    env: process.env,
    run: main,
  },);

//endregion Editor calibrate
