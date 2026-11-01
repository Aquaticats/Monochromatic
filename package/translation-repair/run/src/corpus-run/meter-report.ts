import { reportingRefusals, } from './cli-refusal.ts';
import type { CommandLineOf, } from './command-lines.ts';
import { reportMeters, } from './meter-report-run.ts';

//region Meter report
// Reads run logs and says how much of the time each provider could be spent
// on, and the longest stretch it could not.
//
// THIS EXISTS TO PRICE A SEAT. Three writer seats sit on models only the second
// provider serves, and the argument for them rested on a 40-round quality pass
// plus an availability adjustment that was reasoned about rather than measured.
// A seat on a provider that is out a third of the time is a different seat from
// one on a provider that is out an hour a week, and nothing could tell those
// apart from a run log until the reading was promoted out of debug.
//
// SPENDS NO QUOTA AND TOUCHES NO MODEL. It reads files.
//
// WHAT IT WILL NOT DO IS TURN A SPARSE RECORD INTO A CONFIDENT NUMBER. Readings
// happen when a run asks for one, so the record is dense while work is running
// and empty otherwise, and every outage is reported as a range with its open
// ends named. A duty cycle here is availability WHEN WE WERE ASKING, which is
// the quantity that prices a seat, and is not the same as availability.
//
// THIS FILE IS ONLY THE WIRING. `meter-report-run.ts` holds the procedure,
// `meter-report-provider.ts` one provider's report, `meter-report-level.ts` what
// its meter read and `meter-report-text.ts` the spans, instants and outages.

/**
 Reports the named logs and leaves the exit code the report returns.

 @param line - the report's command line, read whole by `reportingRefusals`

 @example
 ```ts
 await main({ line, },);
 ```
 */
async function main({ line, }: { readonly line: CommandLineOf<'meter-report'>; },): Promise<void> {
  process.exitCode = await reportMeters({ line, },);
}

if (import.meta.main)
  await reportingRefusals({
    what: 'meter-report',
    argv: process.argv,
    env: process.env,
    run: main,
  },);

//endregion Meter report
