import { join, } from 'node:path';

import { LEDGER_DIR, } from '@monochromatic-dev/translation-repair-house-rule/ts';
import { wordForCount, } from '@monochromatic-dev/translation-repair-typography/ts';
import type { CommandLineOf, } from './command-lines.ts';
import { readLedgerDirectory, } from './ledger-directory.ts';
import {
  printRefusals,
  printSeat,
  printSummary,
} from './ledger-report-print.ts';

//region Ledger report run
// The ledger report's whole procedure: read a run's ledger directory, say what
// it held and what would not read, and print either the summary or one seat's
// work.
//
// SPLIT OUT OF `ledger-report.ts` so the entry holds only what a real process
// supplies (the command line and the runs directory the environment names).
// This takes both, so a case reads a ledger it wrote and says nothing about the
// operator's runs.

/**
 Exit code returned when there is no ledger to read.
 */
const NOTHING_TO_READ = 1;

/**
 Exit code returned when the ledger was read but not all of it.

 SEPARATE FROM AN ABSENT LEDGER, on the same grounds `verify-published.ts`
 separates its two: a run that recorded nothing and a run whose record is
 part unreadable answer a roster question differently, and a gate treating
 them alike either trusts a partial standing or discards a whole one.
 */
const LEDGER_INCOMPLETE = 2;

/**
 Exit code returned when every ledger file read.
 */
const LEDGER_READ = 0;

/**
 Reads a run's ledger and reports what it holds.

 The report on stdout and the returned exit code ARE the output; the entry
 file sets the code on the process, so a case reads it without starting one.

 @param line - the report's command line, read whole by `reportingRefusals`

 @param runsDir - run directory to read, which the entry takes from the
 environment or the house default so this never reads either

 @returns The exit code the command leaves: 1 for a run that wrote no ledger,
 2 when a ledger file would not read, whatever else did, and 0 otherwise

 @example
 ```ts
 process.exitCode = await reportLedger({ line, runsDir, },);
 ```
 */
export async function reportLedger(
  {
    line,
    runsDir,
  }: {
    readonly line: CommandLineOf<'ledger-report'>;
    readonly runsDir: string;
  },
): Promise<number> {
  // A FLAG WITH NOTHING AFTER IT IS REFUSED rather than ignored, before this
  // runs. Falling through to the summary would answer a question nobody asked,
  // and the summary looks exactly like a successful run to anything reading the
  // exit code. So is a seat flag followed by the next flag, which this once
  // read as the seat to report (ledger B75).
  /**
   Seat to read in full, unwritten when the whole ledger was asked for.
   */
  const seat = line.flag('model',);

  /**
   Every contest the ledger holds, beside the files that would not read.
   */
  const reading = await readLedgerDirectory({
    dir: join(
      runsDir,
      LEDGER_DIR,
    ),
  },);

  /**
   Both halves of the reading, named so no member chain runs two steps deep.
   */
  const {
    refused,
    rounds,
  } = reading;

  console.log(`ledger-report: ${String(rounds.length,)} ${
    wordForCount({
      count: rounds.length,
      one: 'contest',
      many: 'contests',
    },)
  } under ${runsDir}`,);
  printRefusals({ reading, },);

  /**
   Code of a run whose ledger was read: incomplete when a file refused.
   */
  const readCode = (refused.length > 0) ? LEDGER_INCOMPLETE : LEDGER_READ;

  if (rounds.length === 0) {
    if (refused.length === 0) {
      console.log(
        'NOTHING RECORDED. This run wrote no ledger, which is not the same as a run whose models '
          + 'wrote nothing: every run started before candidate-ledger.ts landed has none, and so does '
          + 'any run launched without TRANSLATION_REPAIR_RUNS_DIR set.',
      );
      return NOTHING_TO_READ;
    }
    console.log(
      'NOTHING COUNTED. Every ledger file this run wrote refused to read, so this is a run whose '
        + 'record was lost rather than a run that recorded nothing.',
    );
    return readCode;
  }

  if (seat.kind === 'written') {
    printSeat({
      reading,
      wanted: seat.value,
    },);
    return readCode;
  }

  printSummary({ reading, },);
  return readCode;
}

//endregion Ledger report run
