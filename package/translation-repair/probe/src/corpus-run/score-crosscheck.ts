import { readPoolPolicy, } from '@monochromatic-dev/translation-repair-artifact-pool/ts';
import {
  reportingRefusals,
  resolveRunsDir,
  RUN_MODELS,
} from '@monochromatic-dev/translation-repair-run/ts';
import { printCrosscheck, } from './score-crosscheck-run.ts';

//region Score crosscheck
// Wiring only: the runs directory, the judge roster and the pool's generation
// policy are named here, and the report itself is `score-crosscheck-run.ts`.

/**
 Reads a run's artifacts and prints the crosscheck population.

 @example
 ```ts
 await main();
 ```
 */
async function main(): Promise<void> {
  await printCrosscheck({
    runsDir: await resolveRunsDir(),
    roster: RUN_MODELS.judgeModelIds,
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
    what: 'score-crosscheck',
    argv: process.argv,
    env: process.env,
    run: main,
  },);

//endregion Score crosscheck
