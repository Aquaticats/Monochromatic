import {
  createProbeRunClient,
  reportingRefusals,
} from '@monochromatic-dev/translation-repair-run/ts';
import { runCheckerSensitivity, } from './checker-sensitivity-run.ts';
import { readStageSettings, } from '@monochromatic-dev/translation-repair-stage/ts';

//region Checker sensitivity
// Asks whether the resolution checkers can say NO.
//
// Across real-corpus artifacts the checkers called 2215 of 2257 accepted issues
// resolved, 98.1 percent. That number is quoted as evidence the repair works,
// and it cannot carry that weight until the stage is shown to discriminate: a
// checker that answers `fixed` to everything produces exactly this rate whether
// or not anything was repaired, and `resolvedIssueIds` feeds both candidate
// selection and the milestone's headline.
//
// EVERY SHEET HERE IS HAND-WRITTEN FIXTURE TEXT, so each call passes
// `UNATTRIBUTED_TEXT`: no roster model wrote a word of it and no verdict is
// discounted. Attributing any of it would measure the self-certification
// discount rather than the checkers, which is the one thing this probe asks.
//
// The probe sensitivity check settled the same question for the introduced-
// defect stage and was worth every one of its three calls. This is the same
// experiment aimed at the older, more load-bearing stage.
//
// Cat-themed invention throughout. No corpus text, licensed or otherwise, and
// nothing is written.

/**
 Hands the run's client builder to the cases.

 @example
 ```ts
 await main();
 ```
 */
function main(): Promise<void> {
  return runCheckerSensitivity({
    newClient: createProbeRunClient,
    stageSettings: readStageSettings({ env: process.env, },),
  },);
}

// Guarded so this runs only when INVOKED. Unguarded it ran on IMPORT, so
// anything pulling this module into the bundle performed the whole task as a
// side effect of loading the library: for the probing scripts that means live
// model calls, and for every one of them it means writing files.
if (import.meta.main)
  await reportingRefusals({
    what: 'checker-sensitivity',
    argv: process.argv,
    env: process.env,
    run: main,
  },);

//endregion Checker sensitivity
