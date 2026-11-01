import {
  type CommandLineOf,
  reportingRefusals,
  resolveRunsDir,
} from '@monochromatic-dev/translation-repair-run/ts';
import { reportStandings, } from './editor-standing-read-run.ts';

//region Editor standing read
// THE EDITOR STANDING THAT COSTS NOTHING, read off work already paid for: every
// settled artifact's repair rounds, ranked per seat and per pipeline digest.
//
// SPENDS NO QUOTA AND TOUCHES NO MODEL. It reads files.
//
// THIS FILE IS ONLY THE WIRING. `editor-standing-read-run.ts` holds the
// procedure, `editor-standing-read-listing.ts` which files are artifacts,
// `editor-standing-read-artifact.ts` what one artifact turned out to be, and
// `editor-standing-read-seat.ts` what one digest prints.

/**
 Reports the named archives, or the runs directory, and leaves the exit code
 the report returns.

 @param line - the report's command line, read whole by `reportingRefusals`

 @example
 ```ts
 await main({ line, },);
 ```
 */
async function main({ line, }: { readonly line: CommandLineOf<'editor-standing-read'>; },): Promise<void> {
  process.exitCode = await reportStandings({
    line,
    resolveRuns: resolveRunsDir,
  },);
}

if (import.meta.main)
  await reportingRefusals({
    what: 'editor-standing-read',
    argv: process.argv,
    env: process.env,
    run: main,
  },);

//endregion Editor standing read
