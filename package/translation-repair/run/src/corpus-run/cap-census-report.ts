import type { CapSample, } from './cap-census-read.ts';
import { capCensus, } from './cap-census-rule.ts';
import { capCensusPassRunReading, } from './cap-census-pass-run.ts';
import { printCapCensus, } from './cap-census-print.ts';
import {
  capCensusLogsUnder,
  reportCapCensusUnreadable,
} from './cap-census-walk.ts';
import type { CommandLineOf, } from './command-lines.ts';

//region Cap census report
// THE PRE-LAUNCH READING OF THE COMPLETION CAPS (ledger P10). Reads pass-run
// logs, applies the cap rule per seat, and says where a card and its calls
// disagree. Spends no quota and touches no model.
//
// PASS-RUN LOGS ONLY, as the 2026-09-28 re-read learned (ledger P7): suite,
// build and prototype logs carry fixture `SPEND` lines, so a log counts only
// when the first line among its first two thousand that opens `START tip=`
// holds a commit id a repository could give (`cap-census-pass-run.ts`, ledger
// B963): a suite prints the START line of a fixture pass, with a placeholder id.
//
// ONE LOG AT A TIME. The agent root holds thousands of logs, some large; each
// is opened only far enough to read its head, and a pass-run log's text is
// dropped once its samples are read, so memory holds samples, not logs.
//
// PRINTS IDS AND NUMBERS. A run log holds unlicensed corpus wording, and
// nothing here reads past the two line kinds the census needs.

/**
 Reads the named logs and prints the census.

 Returns nothing: the report on stdout IS the output.

 @param line - census's command line, read whole by `reportingRefusals`,
 which refuses it when no path is named

 @example
 ```ts
 await reportCapCensus({ line, },);
 ```
 */
export async function reportCapCensus({ line, }: { readonly line: CommandLineOf<'cap-census'>; },): Promise<void> {
  /**
   Files or directories named on the command line.
   */
  const roots = line.positionals;

  /**
   Every log under the roots.
   */
  const {
    logs,
    unreadable,
    danglingLinks,
  } = await capCensusLogsUnder({ roots, },);

  /**
   Paths the walk could not read, and the logs that would not open after it
   found them.
   */
  const unreadableCount = { paths: unreadable, };

  /**
   Completed calls across every pass-run log.
   */
  const samples: CapSample[] = [];

  /**
   Pass-run logs read, one entry each.
   */
  const passRunLogs: string[] = [];

  /**
   Lines those logs left out for a stamp the logger did not write.
   */
  const unstamped = { lines: 0, };
  for (const path of logs) {
    /* oxlint-disable no-await-in-loop -- one log at a time, so memory holds samples rather than every log's text */
    /**
     This log's samples, or that it is no pass run.
     */
    const read = await (async function readOrCount(): Promise<Awaited<ReturnType<typeof capCensusPassRunReading>> | 'unreadable'> {
      try {
        return await capCensusPassRunReading({ path, },);
      }
      catch (error) {
        // A LOG THE CENSUS CANNOT OPEN IS COUNTED, NOT FATAL, as a path the
        // walk cannot read is: one sealed log must not end a census of
        // thousands.
        reportCapCensusUnreadable({
          path,
          error,
        },);
        return 'unreadable';
      }
    })();
    /* oxlint-enable no-await-in-loop */
    if (read === 'unreadable')
      unreadableCount.paths += 1;
    else if (read !== 'not-a-pass-run') {
      passRunLogs.push(path,);
      unstamped.lines += read.unstampedLines;
      // ONE AT A TIME, NOT SPREAD: one log can hold more calls than a call's
      // argument list takes.
      for (const sample of read.samples)
        samples.push(sample,);
    }
  }
  printCapCensus({
    census: capCensus({ samples, },),
    tally: {
      logs: logs.length,
      passRunLogs: passRunLogs.length,
      samples: samples.length,
      unreadable: unreadableCount.paths,
      danglingLinks,
      unstampedLines: unstamped.lines,
    },
  },);
}

//endregion Cap census report
