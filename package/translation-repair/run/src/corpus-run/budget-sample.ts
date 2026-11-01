import { contextRoot, } from '@monochromatic-dev/translation-repair-typography/ts';
import { fetchTransport, } from '@monochromatic-dev/translation-repair-provider-stream/ts';
import { reportingRefusals, } from './cli-refusal.ts';
import { sampleBudgets, } from './budget-sample-run.ts';

//region Budget sample
// Takes one reading of every provider's meter and leaves it in the log:
// `budget-sample-keys.ts` reads the four keys and `budget-sample-run.ts` is the
// procedure.

/**
 Logger root for this probe.
 */
const l = contextRoot({ tag: 'translation-repair', },);

/**
 Samples the meters over the process's own environment and the live transport.
 Not `async`: it hands the promise on, so a run that refuses before its first
 call leaves no continuation behind it that nothing runs.

 @example
 ```ts
 await sampleOverTheEnvironment();
 ```
 */
function sampleOverTheEnvironment(): Promise<void> {
  return sampleBudgets({
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
