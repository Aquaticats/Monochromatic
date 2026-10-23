import {
  createProbeRunClient,
  reportingRefusals,
} from '@monochromatic-dev/translation-repair-run/ts';
import {
  PRODUCTION_LIST,
  SENSITIVITY_ARMS,
} from './probe-sensitivity-arms.ts';
import { runSensitivity, } from './probe-sensitivity-run.ts';
import { readStageSettings, } from '@monochromatic-dev/translation-repair-stage/ts';

//region Probe sensitivity
// Asks whether the introduced-defect probe can detect damage AT ALL.
//
// Run 007 reported no claims of any kind across its first eight regions: every
// prober cast an explicit negative and nothing was corroborated, contradicted,
// or even unanchored. Two readings fit that. The repairs may genuinely be
// clean, having already passed an editor ensemble, a judge selection, and a
// checker stage. Or the three defenses built into the prompt against reporting
// the pre-existing defect may have over-corrected into an instrument that never
// claims anything, in which case a whole round of zeros would be
// indistinguishable from a stage that is silently broken.
//
// Waiting for the round to end does not separate those. Injecting damage does:
// a probe that misses an obvious fabrication is not conservative, it is deaf.
//
// Probe inputs live in `probe-sensitivity-input.ts` and are cat-themed invention.
// NO corpus text, licensed or otherwise, takes part, and this writes nothing.
//
// THE ARMS LIVE IN `probe-sensitivity-arms.ts`, as data with a test,
// because this file once built its arms inline, and its `prior=shown` arm sent
// the same prompt as its `prior=absent` arm because it relied on a default
// that had flipped. Every arm now names the list it sends, and the test holds
// the name to the value before a run spends anything.

/**
 Hands the run's client to the arms and has them asked in order.

 @example
 ```ts
 await main();
 ```
 */
function main(): Promise<void> {
  return runSensitivity({
    // One client for the whole instrument, counted on the run-wide seat tally.
    client: createProbeRunClient(),
    arms: SENSITIVITY_ARMS,
    production: PRODUCTION_LIST,
    stageSettings: readStageSettings({ env: process.env, },),
  },);
}

// Guarded so this runs only when INVOKED. Unguarded it ran on IMPORT, so
// anything pulling this module into the bundle performed the whole task as a
// side effect of loading the library: for the probing scripts that means live
// model calls, and for every one of them it means writing files.
if (import.meta.main)
  await reportingRefusals({
    what: 'probe-sensitivity',
    argv: process.argv,
    env: process.env,
    run: main,
  },);

//endregion Probe sensitivity
