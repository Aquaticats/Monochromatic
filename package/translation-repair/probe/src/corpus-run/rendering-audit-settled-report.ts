import {
  type CommandLineOf,
  reportingRefusals,
  resolveRunsDir,
} from '@monochromatic-dev/translation-repair-run/ts';
import { runSettledReport, } from './rendering-audit-settled-report-run.ts';

//region Settled audit report
// Prints the readings the settled audit owes, from a run already on disk.
//
// SPENDS NOTHING. The rows were bought once; every question anyone asks of them
// afterwards should be free, or it will not get asked twice.
//
// READS THE NEWEST RUN by default, because these probes accumulate on purpose:
// the store keeps every run so a verdict can be compared against the one it was
// bought to be compared against, and a reader that silently merged them would
// undo that.
//
// `--against <run>` PAIRS TWO RUNS subject by subject, which is the only way to
// state the spread this instrument moves through on unchanged input. Without
// it, the archive-versus-fresh comparison is a difference with no scale to read
// it against.

/**
 Hands the report the command line and the way to name the runs directory.

 @param line - the report's command line, read whole by `reportingRefusals`

 @example
 ```ts
 await main({ line, },);
 ```
 */
function main({ line, }: { readonly line: CommandLineOf<'rendering-audit-settled-report'>; },): Promise<void> {
  return runSettledReport({
    line,
    resolveRuns: resolveRunsDir,
  },);
}

if (import.meta.main)
  await reportingRefusals({
    what: 'rendering-audit-settled-report',
    argv: process.argv,
    env: process.env,
    run: main,
  },);

//endregion Settled audit report
