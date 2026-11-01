import { wordForCount, } from '@monochromatic-dev/translation-repair-typography/ts';
import type { CommandLineOf, } from './command-lines.ts';
import {
  linesOfLog,
  readLogTexts,
  refuseRepeatedLogs,
} from './report-log-read.ts';
import { priceTally, } from './spend-cost.ts';
import { printCost, } from './spend-report-print.ts';
import { tallySpend, } from './spend-read.ts';

//region Spend report run
// The spend report's whole procedure: read every named log, say how many seats
// and unreadable lines it held, and print what the run they describe cost.
//
// SPLIT OUT OF `spend-report.ts` so the entry holds only the wiring.

/**
 Reads named logs and reports what the run they describe cost.

 Returns nothing: the report on stdout IS the output.

 @param line - the report's command line, read whole by `reportingRefusals`,
 which refuses it when no log is named: any log a pass, probe or calibration
 wrote will do, and passing several totals them as one run

 @example
 ```ts
 await reportSpendCost({ line, },);
 ```
 */
export async function reportSpendCost({ line, }: { readonly line: CommandLineOf<'spend-report'>; },): Promise<void> {
  /**
   Logs to read, named on the command line.
   */
  const paths = line.positionals;

  // BEFORE ANY FILE IS READ: a log named twice would be totalled twice.
  refuseRepeatedLogs({ paths, },);

  /**
   Every line of every named log, in one list.
   */
  const lines = (await readLogTexts({ paths, },))
    .flatMap(function linesOf(text,): readonly string[] {
      return linesOfLog({ text, },);
    },);

  /**
   Per-seat totals over every record those lines held.
   */
  const tally = tallySpend({ lines, },);

  /**
   Distinct provider and model pairs those records named.
   */
  const seatCount = tally
    .seats
    .length;

  console.log(
    `spend-report: ${String(paths.length,)} ${
      wordForCount({
        count: paths.length,
        one: 'log',
        many: 'logs',
      },)
    }, ${String(lines.length,)} ${
      wordForCount({
        count: lines.length,
        one: 'line',
        many: 'lines',
      },)
    }, `
      + `${String(seatCount,)} ${
        wordForCount({
          count: seatCount,
          one: 'seat',
          many: 'seats',
        },)
      }`,
  );

  if (seatCount === 0) {
    console.log(
      'NOTHING RECORDED. These logs carry no SPEND line, which is not the same as a run that spent '
        + 'nothing: every log written before spend-line.ts landed carries none. Check the run date '
        + 'against that landing before reading this as a free run.',
    );
  }

  if (tally.unreadableLines > 0) {
    console.log(
      `${String(tally.unreadableLines,)} ${
        wordForCount({
          count: tally.unreadableLines,
          one: 'line',
          many: 'lines',
        },)
      } carried the marker and would not parse, so this report's totals `
        + 'are short by whatever those calls cost',
    );
  }

  // NO TOTAL OF NOTHING: a log with no record is not a run that spent
  // nothing, and a total of zero printed beside that sentence is the figure it
  // warns against reading.
  if (seatCount === 0)
    return;

  printCost({ cost: priceTally({ tally, },), },);

  /**
   Calls across every seat whose counts and cost were reckoned.
   */
  const reckonedCalls = tally.seats
    .reduce(
    function sum(
      total,
      seat,
    ): number {
      return total + seat.reckonedCalls;
    },
    0,
    );

  if (reckonedCalls > 0) {
    console.log(
      `RECKONED, NOT REPORTED: ${String(reckonedCalls,)} ${
        wordForCount({
          count: reckonedCalls,
          one: 'call',
          many: 'calls',
        },)
      } ${
        wordForCount({
          count: reckonedCalls,
          one: 'was',
          many: 'were',
        },)
      } written as reckonings, an attempt `
        + 'abandoned before it finished or a Bedrock attempt at its bound, so the tokens and cost this report '
        + 'shows for such calls are estimates or bounds rather than what the wire said (ledger P14)',
    );
  }
}

//endregion Spend report run
