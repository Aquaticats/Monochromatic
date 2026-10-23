import { tagged, } from '@monochromatic-dev/module-logger/ts';

import {
  createProbeRunClient,
  readRunCorpusPinSetting,
  reportingRefusals,
  RUN_PER_CALL_TIMEOUT_MS,
  RUN_ROSTER,
} from '@monochromatic-dev/translation-repair-run/ts';
import { runCoverageControl, } from './coverage-control-probe-run.ts';
import { coverageControlHolds, } from './coverage-control.ts';
import { readStageSettings, } from '@monochromatic-dev/translation-repair-stage/ts';

//region Coverage control probe
// Asks whether the coverage roster can vote absence at all: it deletes the
// spans the roster itself anchored on and asks again. SPENDS QUOTA. Wiring
// only: the walk is `coverage-control-probe-run.ts`, the passages
// `coverage-control-probe-cases.ts`, the printed lines
// `coverage-control-probe-lines.ts`.

if (import.meta.main)
  await reportingRefusals({
    what: 'coverage-control-probe',
    argv: process.argv,
    env: process.env,
    run: function runCoverageControlProbe({ line, },) {
      /**
       Corpus clone and commit this run reads, after any environment
       override, read inside the refusal boundary (ledger B302).
       */
      const { pin, } = readRunCorpusPinSetting({ env: process.env, },);
      return runCoverageControl({
        line,
        pin,
        newClient: createProbeRunClient,
        holds: coverageControlHolds,
        roster: RUN_ROSTER,
        exchangeTimeoutMs: RUN_PER_CALL_TIMEOUT_MS,
        l: tagged({ tag: 'coverage-control', },),
        stageSettings: readStageSettings({ env: process.env, },),
      },);
    },
  },);

//endregion Coverage control probe
