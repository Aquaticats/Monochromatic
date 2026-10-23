import { logger, } from '@monochromatic-dev/module-logger/ts';

import {
  createProbeRunClient,
  reportingRefusals,
  RUN_PER_CALL_TIMEOUT_MS,
  RUN_ROSTER,
} from '@monochromatic-dev/translation-repair-run/ts';
import { reportModelHealth, } from './model-health-report.ts';

//region Model health
// Asks every model on the roster for one trivial structured answer, and reports
// what came back: `model-health-probe.ts` asks one model, and
// `model-health-report.ts` walks the roster and counts who answered.

/**
 Builds the run client, asks the whole roster and leaves the exit code behind.

 @example
 ```ts
 await askRoster();
 ```
 */
async function askRoster(): Promise<void> {
  process.exitCode = await reportModelHealth({
    client: createProbeRunClient(),
    roster: RUN_ROSTER,
    timeoutMs: RUN_PER_CALL_TIMEOUT_MS,
    l: logger,
  },);
}

if (import.meta.main)
  await reportingRefusals({
    what: 'model-health',
    argv: process.argv,
    env: process.env,
    run: askRoster,
  },);

//endregion Model health
