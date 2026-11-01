import {
  reportingRefusals,
  resolveRunsDir,
} from '@monochromatic-dev/translation-repair-run/ts';
import { textSettingOf, } from '@monochromatic-dev/translation-repair-glossary/ts';
import { printVerifyScore, } from './score-verify-run.ts';

//region Score verify
// Wiring only: the runs directory and the sheet name come from the
// environment here, and the report itself is `score-verify-run.ts`.

/**
 Reports what the graded sheet says about the unlabelled probe.

 @example
 ```ts
 await main();
 ```
 */
async function main(): Promise<void> {
  await printVerifyScore({
    runsDir: await resolveRunsDir(),
    // Which sheet to score, so one scorer serves every sheet this formatter
    // writes; an exported-empty variable reads as unset (ledger D15).
    basename: textSettingOf({
      env: process.env,
      name: 'VERIFY_SHEET_BASENAME',
      fallback: 'probe-verify',
    },),
  },);
}

// Guarded so this runs only when INVOKED. Unguarded it ran on IMPORT, so
// anything pulling this module into the bundle performed the whole task as a
// side effect of loading the library: for the probing scripts that means live
// model calls, and for every one of them it means writing files.
if (import.meta.main)
  await reportingRefusals({
    what: 'score-verify',
    argv: process.argv,
    env: process.env,
    run: main,
  },);

//endregion Score verify
