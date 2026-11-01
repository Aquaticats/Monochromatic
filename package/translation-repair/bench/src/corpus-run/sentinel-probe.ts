import { repairTranslation, } from '@monochromatic-dev/translation-repair-repair/ts';
import { monotonicMs, } from '@monochromatic-dev/translation-repair-markup/ts';
import {
  createProbeRunClient,
  readRunCorpusPinSetting,
  reportingRefusals,
  RUN_MODELS,
  RUN_PER_CALL_TIMEOUT_MS,
} from '@monochromatic-dev/translation-repair-run/ts';
import { probeCorpusEntries, } from './sentinel-probe-run.ts';
import { readStageSettings, } from '@monochromatic-dev/translation-repair-stage/ts';

//region Sentinel probe
// Runs a set of named, known-behavior corpus entries through the pipeline and
// prints one PROBE line each (status, issue counts, findings). Wiring only:
// the walk is `sentinel-probe-run.ts` and the line is `sentinel-probe-line.ts`.
// Run it with `mise run //package/translation-repair/bench:sentinel-probe -- Anilovr Aniloviraw`.

if (import.meta.main)
  await reportingRefusals({
    what: 'sentinel-probe',
    argv: process.argv,
    env: process.env,
    run: function runSentinelProbe({ line, },) {
      /**
       Corpus clone and commit this run reads, after any environment
       override, read inside the refusal boundary (ledger B302).
       */
      const { pin, } = readRunCorpusPinSetting({ env: process.env, },);
      return probeCorpusEntries({
        line,
        pin,
        newClient: createProbeRunClient,
        repair: repairTranslation,
        models: RUN_MODELS,
        perCallTimeoutMs: RUN_PER_CALL_TIMEOUT_MS,
        now: monotonicMs,
        stageSettings: readStageSettings({ env: process.env, },),
      },);
    },
  },);

//endregion Sentinel probe
