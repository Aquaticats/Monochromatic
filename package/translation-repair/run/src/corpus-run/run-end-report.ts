import { tagged, } from '@monochromatic-dev/logger-core/ts';

import { allInInputOrder, } from '@monochromatic-dev/translation-repair-document-structure/ts';
import { errorName, } from '@monochromatic-dev/translation-repair-markup/ts';
import { contextRoot, } from '@monochromatic-dev/translation-repair-typography/ts';

//region Run end reports
// WHAT A RUN SAYS WHEN IT ENDS, registered where the thing to report lives and
// printed where the run ends.
//
// WHY A REGISTRY. The run client is built deep inside a stage, while the
// command that owns the run's end is `reportingRefusals`, far above it with no
// handle on the client. NeuralWatt's plan-energy report (the difference in the
// account's charged energy between the run's first call and its end,
// `neuralwatt-quota.ts` in the provider package) needs the client that took
// the starting reading, so the client registers a report when it is built and
// the command's end runs whatever was registered. The seat tally
// (`RUN_SEATS`) is the same shape: process-wide, because a process runs one
// command.
//
// ONE REPORT PER KEY, the later replacing the earlier, so a run that builds
// its client twice reports once, from the client that is current. Running the
// reports EMPTIES the registry: a second command in the same process (a test
// file) never repeats the first one's lines.

/**
 Logger root for the end reports.
 */
const l = contextRoot({ tag: 'translation-repair', },);

/**
 One thing a run says at its end.

 @example
 ```ts
 const report: RunEndReport = { key: 'neuralwatt-plan-energy', report: async () => 'a line', };
 ```
 */
export type RunEndReport = {
  /**
   Name the report is registered under, naming it in a line that says it failed.
   */
  readonly key: string;

  /**
   Builds the report's line.
   */
  readonly report: () => Promise<string>;
};

/**
 Reports registered and not yet run, by key, in registration order.
 */
const registered = new Map<string, RunEndReport>();

/**
 Registers a report for the end of the run, replacing one of the same key.

 @param key - name the report is kept under

 @param report - builds the line the run's end prints

 @example
 ```ts
 registerRunEndReport({ key: 'neuralwatt-plan-energy', report, },);
 ```
 */
export function registerRunEndReport({
  key,
  report,
}: RunEndReport,): void {
  registered.set(
    key,
    {
      key,
      report,
    },
  );
}

/**
 Runs every registered report once, in registration order, and empties the
 registry.

 A REPORT THAT FAILS IS SAID BY ITS KEY AND THE NAME OF THE FAILURE, never the
 failure's words, which can carry a provider's reply; the reports after it
 still run.

 @returns One line per report

 @example
 ```ts
 for (const line of await runEndReports()) console.error(line);
 ```
 */
export async function runEndReports(): Promise<readonly string[]> {
  /**
   Logger pre-tagged with this function's name.
   */
  const rl = tagged({
    tag: runEndReports.name,
    l,
  },);

  /**
   The reports taken off the registry, which is empty from here on.
   */
  const taken = [...registered.values(),];
  registered.clear();
  return await allInInputOrder({
    members: taken.map(async function lineOf({
      key,
      report,
    },): Promise<string> {
      try {
        return await report();
      }
      catch (error) {
        rl.warn(`the ${key} end report failed (${errorName({ error, },)})`,);
        return `${key}: this end report could not be built (${errorName({ error, },)})`;
      }
    },),
  },);
}

//endregion Run end reports
