import { logger, } from '@monochromatic-dev/module-logger/ts';

import { readPoolPolicy, } from '@monochromatic-dev/translation-repair-artifact-pool/ts';
import {
  createProbeRunClient,
  reportingRefusals,
  resolveRunsDir,
  RUN_MODELS,
  RUN_PER_CALL_TIMEOUT_MS,
} from '@monochromatic-dev/translation-repair-run/ts';
import { sampleDamage, } from './damage-sample-run.ts';
import { textSettingOf, } from '@monochromatic-dev/translation-repair-glossary/ts';
import { readStageSettings, } from '@monochromatic-dev/translation-repair-stage/ts';

//region Damage sample
// Draws shipped regions at random and asks a human the same source-anchored
// question the probe asks: `damage-sample-pool.ts` collects the regions,
// `damage-sample-draw.ts` draws them and `damage-sample-run.ts` is the
// procedure.

/**
 Samples damage over the runs directory, the draw seed and the pool's
 generation policy of the environment.

 @example
 ```ts
 await sampleOverTheEnvironment();
 ```
 */
async function sampleOverTheEnvironment(): Promise<void> {
  await sampleDamage({
    runsDir: await resolveRunsDir(),
    // Draw seed, overridable so a later round can draw a fresh sample; an
    // exported-empty variable reads as unset (ledger D15).
    seed: textSettingOf({
      env: process.env,
      name: 'DAMAGE_SAMPLE_SEED',
      fallback: 'damage-round-one',
    },),
    openClient: createProbeRunClient,
    proberModelIds: RUN_MODELS.checkerModelIds,
    perCallTimeoutMs: RUN_PER_CALL_TIMEOUT_MS,
    l: logger,
    policy: readPoolPolicy({ env: process.env, },),
    // The checkout this command's build stands in, where a required commit is
    // resolved and placed; the pool took it from its own location until
    // 2026-10-10.
    repository: import.meta.dirname,
    stageSettings: readStageSettings({ env: process.env, },),
  },);
}

// Guarded so this runs only when INVOKED. Unguarded it ran on IMPORT, so
// anything pulling this module into the bundle performed the whole task as a
// side effect of loading the library: for the probing scripts that means live
// model calls, and for every one of them it means writing files.
if (import.meta.main)
  await reportingRefusals({
    what: 'damage-sample',
    argv: process.argv,
    env: process.env,
    run: sampleOverTheEnvironment,
  },);

//endregion Damage sample
