import { tagged, } from '@monochromatic-dev/module-logger/ts';

import {
  createProbeRunClient,
  readRunCorpusPinSetting,
  reportingRefusals,
  RUN_MODELS,
  RUN_PER_CALL_TIMEOUT_MS,
} from '@monochromatic-dev/translation-repair-run/ts';
import { probeTranslate, } from './translate-probe-run.ts';
import { readStageSettings, } from '@monochromatic-dev/translation-repair-stage/ts';

//region Translate probe
// PROTOTYPE for the translate-first re-design: asks whether a translate stage
// can render a section the corpus never translated, over the sparsest aligned
// section of the entry named by `PROBE_ENTRY`. SPENDS QUOTA. Wiring only: the walk is
// `translate-probe-run.ts`, the printed lines `translate-probe-lines.ts` and
// the choice of section `translate-probe-coverage.ts`.

/**
 Entry carrying the worst measured coverage gap.
 */
const PROBE_ENTRY = 'XingZ60';

// Guarded so this runs only when INVOKED. Unguarded it ran on IMPORT, so
// anything pulling this module into the bundle performed the whole task as a
// side effect of loading the library: for the probing scripts that means live
// model calls, and for every one of them it means writing files.
if (import.meta.main)
  await reportingRefusals({
    what: 'translate-probe',
    argv: process.argv,
    env: process.env,
    run: function runTranslateProbe() {
      /**
       Corpus clone and commit this run reads, after any environment
       override, read inside the refusal boundary (ledger B302).
       */
      const { pin, } = readRunCorpusPinSetting({ env: process.env, },);
      return probeTranslate({
        entryId: PROBE_ENTRY,
        pin,
        newClient: createProbeRunClient,
        editorModelIds: RUN_MODELS.editorModelIds,
        perCallTimeoutMs: RUN_PER_CALL_TIMEOUT_MS,
        log: tagged({ tag: 'translate-probe', },),
        stageSettings: readStageSettings({ env: process.env, },),
      },);
    },
  },);

//endregion Translate probe
