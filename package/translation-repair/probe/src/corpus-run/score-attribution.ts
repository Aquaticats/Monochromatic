import { readPoolPolicy, } from '@monochromatic-dev/translation-repair-artifact-pool/ts';
import {
  reportingRefusals,
  resolveRunsDir,
} from '@monochromatic-dev/translation-repair-run/ts';
import { printAttribution, } from './score-attribution-run.ts';

//region Score attribution
// Wiring only: the runs directory and the pool's generation policy come from
// the environment here, and the report itself is `score-attribution-run.ts`.

/**
 Reads a run's artifacts and prints per-critic calibration.

 @example
 ```ts
 await main();
 ```
 */
async function main(): Promise<void> {
  await printAttribution({
    runsDir: await resolveRunsDir(),
    policy: readPoolPolicy({ env: process.env, },),
    // The checkout this command's build stands in, where a required commit is
    // resolved and placed; the pool took it from its own location until
    // 2026-10-10.
    repository: import.meta.dirname,
  },);
}

// Guarded so this runs only when INVOKED. Unguarded it ran on IMPORT, so
// anything pulling this module into the bundle performed the whole task as a
// side effect of loading the library: for the probing scripts that means live
// model calls, and for every one of them it means writing files.
if (import.meta.main)
  await reportingRefusals({
    what: 'score-attribution',
    argv: process.argv,
    env: process.env,
    run: main,
  },);

//endregion Score attribution
