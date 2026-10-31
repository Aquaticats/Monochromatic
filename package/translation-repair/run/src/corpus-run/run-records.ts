import { reportingRefusals, } from './cli-refusal.ts';
import type { CommandLineOf, } from './command-lines.ts';
import { printRunRecords, } from './run-records-run.ts';

//region Run records
// Prints the records a pass kept in its run folder the way the console showed them. THIS FILE IS ONLY THE WIRING:
// `run-records-run.ts` holds the procedure and says why the raw files are not read on a terminal.

/**
 Prints the named records.

 @param line - command line, read whole by `reportingRefusals`

 @example
 ```ts
 await main({ line, },);
 ```
 */
async function main({ line, }: { readonly line: CommandLineOf<'run-records'>; },): Promise<void> {
  await printRunRecords({ line, },);
}

if (import.meta.main)
  await reportingRefusals({
    what: 'run-records',
    argv: process.argv,
    env: process.env,
    run: main,
  },);

//endregion Run records
