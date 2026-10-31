import { contextRoot, } from '@monochromatic-dev/translation-repair-typography/ts';
import { fetchTransport, } from '@monochromatic-dev/translation-repair-provider-stream/ts';
import { reportingRefusals, } from './cli-refusal.ts';
import { sampleIntoRunLog, } from './budget-sample-run.ts';
import { resolveRunsDir, } from './run-config.ts';

//region Budget sample
// Takes one reading of every provider's meter and leaves it in the records of
// the runs directory: `budget-sample-keys.ts` reads the four keys and
// `budget-sample-run.ts` is the procedure.

/**
 Logger root for this probe.
 */
const l = contextRoot({ tag: 'translation-repair', },);

/**
 Samples the meters over the process's own environment and the live transport,
 into the records of the runs directory (`TRANSLATION_REPAIR_RUNS_DIR`, else the
 worktree's default).

 @example
 ```ts
 await sampleOverTheEnvironment();
 ```
 */
async function sampleOverTheEnvironment(): Promise<void> {
  await sampleIntoRunLog({
    runDir: await resolveRunsDir(),
    env: process.env,
    transport: fetchTransport,
    l,
  },);
}

if (import.meta.main)
  await reportingRefusals({
    what: 'budget-sample',
    argv: process.argv,
    env: process.env,
    run: sampleOverTheEnvironment,
  },);

//endregion Budget sample
