import { wordForCount, } from '@monochromatic-dev/translation-repair-typography/ts';
import type { CommandLineOf, } from '@monochromatic-dev/translation-repair-run/ts';
import { readReportArguments, } from './rendering-audit-settled-args.ts';
import { repeatBandOf, } from './rendering-audit-settled-band.ts';
import {
  printBand,
  printRelations,
  printRelocations,
  printSplit,
  printVoices,
} from './rendering-audit-settled-print.ts';
import {
  rateByVoice,
  splitFor,
} from './rendering-audit-settled-read.ts';
import { relationTallyOf, } from './rendering-audit-settled-relation.ts';
import { auditRelocationPairs, } from './rendering-audit-settled-relocation.ts';
import { auditRepeatsWithin, } from './rendering-audit-settled-repeat.ts';
import {
  newestRun,
  printAcross,
  readRunRows,
} from './rendering-audit-settled-runs.ts';

//region Settled audit report run
// Reads a persisted run and prints what it amounts to, once the process has
// handed over the command line and a way to name the runs directory.
//
// SPENDS NOTHING. The rows were bought once; every question anyone asks of them
// afterwards should be free, or it will not get asked twice.

/**
 Reads a persisted run and prints what it amounts to.

 @param line - the report's command line, read whole by `reportingRefusals`

 @param resolveRuns - names the runs directory, called only when no run was
 named, since a named run needs no directory

 @example
 ```ts
 await runSettledReport({ line, resolveRuns, },);
 ```
 */
export async function runSettledReport(
  {
    line,
    resolveRuns,
  }: {
    readonly line: CommandLineOf<'rendering-audit-settled-report'>;
    readonly resolveRuns: () => Promise<string>;
  },
): Promise<void> {
  /**
   What the command line named, with a valueless flag refused rather than
   read as absent: `--run` written last used to report the newest run and
   `--against` written last used to print no across-run band, in silence.
   */
  const asked = readReportArguments({ line, },);

  /**
   Run named with `--run`, absent when the newest kept run is meant.
   */
  const [named,] = asked.run;

  /**
   File this report reads, which is the newest kept when none was named.
   */
  const path = named ?? await newestRun({ runsDir: await resolveRuns(), },);

  /**
   Earlier run to pair against, empty when none was named.
   */
  const against = asked.against[0] ?? '';

  /**
   Rows that run bought, and where it said it read them from.
   */
  const {
    rows,
    archiveDir,
    roster,
  } = await readRunRows({ path, },);
  console.log(`${path}\n${String(rows.length,)} ${
    wordForCount({
      count: rows.length,
      one: 'subject',
      many: 'subjects',
    },)
  }\n`,);

  console.log('THE TWO HALVES, READ APART',);
  printSplit({
    split: splitFor({
      rows,
      audits: 'archive',
    },),
  },);
  printSplit({
    split: splitFor({
      rows,
      audits: 'fresh',
    },),
  },);

  printRelations({ tallies: relationTallyOf({ rows, },), },);
  printVoices({
    rates: rateByVoice({
      rows,
      roster,
    },),
  },);
  printRelocations({ pairs: auditRelocationPairs({ rows, },), },);
  printBand({
    band: repeatBandOf({ pairs: auditRepeatsWithin({ rows, },), },),
    over: 'texts this run audited twice',
  },);
  if (against !== '')
    await printAcross({
      rows,
      against,
    },);

  // Said every time, because the numbers this report prints are the ones most
  // likely to be quoted without it.
  console.log(
    `\nTWO ENTRIES. Nothing here settles anything about a particular entry, and nothing here may`
      + ` gate what ships: the instrument's own error rate is unmeasured.`
      + `\nArchive that run read: ${archiveDir}`,
  );
}

//endregion Settled audit report run
