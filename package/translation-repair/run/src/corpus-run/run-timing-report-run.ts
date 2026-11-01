import { wordForCount, } from '@monochromatic-dev/translation-repair-typography/ts';
import type { CommandLineOf, } from './command-lines.ts';
import {
  linesOfLog,
  readLogTexts,
  refuseRepeatedLogs,
} from './report-log-read.ts';
import {
  printInFlight,
  printRounds,
} from './run-timing-report-print.ts';
import {
  measureInFlight,
  NothingInFlightError,
  readRunTiming,
} from './run-timing-read.ts';

//region Run timing report run
// The run timing report's whole procedure: read every named log, say what the
// rounds and the calls in flight came to, and say what the logs could not
// answer.
//
// SPLIT OUT OF `run-timing-report.ts` so the entry holds only the wiring.

/**
 Reads every named log and reports where its hours went.

 Returns nothing: the report on stdout IS the output.

 @param line - the report's command line, read whole by `reportingRefusals`,
 which refuses it when no log is named: any log a pass, probe or calibration
 wrote will do, and passing several reads them as one run

 @example
 ```ts
 await reportRunTiming({ line, },);
 ```
 */
export async function reportRunTiming({ line, }: { readonly line: CommandLineOf<'run-timing-report'>; },): Promise<void> {
  /**
   Logs to read, named on the command line.
   */
  const paths = line.positionals;

  // BEFORE ANY FILE IS READ: a log named twice would be counted twice.
  refuseRepeatedLogs({ paths, },);

  /**
   Every line of every named log, in one list.
   */
  const lines = (await readLogTexts({ paths, },))
    .flatMap(function linesOf(text,): readonly string[] {
      return linesOfLog({ text, },);
    },);

  /**
   Every timing line those logs held.
   */
  const reading = readRunTiming({ lines, },);

  console.log(
    `run-timing-report: ${String(paths.length,)} ${
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
    }`,
  );
  printRounds({ reading, },);

  if (reading.callsWithoutDuration > 0) {
    console.log(
      `${String(reading.callsWithoutDuration,)} completion ${
        wordForCount({
          count: reading.callsWithoutDuration,
          one: 'line',
          many: 'lines',
        },)
      } ${
        wordForCount({
          count: reading.callsWithoutDuration,
          one: 'carries',
          many: 'carry',
        },)
      } no elapsed field, predating call durations and leaving no interval to report. Any concurrency `
        + 'this report prints describes only the calls that could be timed.',
    );
  }

  if (reading.callsWithoutStamp > 0) {
    console.log(
      `Completion lines whose stamp the logger did not write: ${String(reading.callsWithoutStamp,)}. No `
        + 'instant places their calls, so any concurrency this report prints leaves them out.',
    );
  }

  try {
    /**
     What the sweep counted over the timed calls.
     */
    const flight = measureInFlight({ calls: reading.calls, },);
    printInFlight({ flight, },);
  } catch (error) {
    if (!(error instanceof NothingInFlightError))
      throw new Error(
        'unreachable: measureInFlight raised something other than its refusal for an empty span, the only throw a call list it checked is non-empty can reach',
        { cause: error, },
      );

    // NO SPAN TO COUNT OVER, whether no call carried a duration or every timed
    // call took no time: the refusal says which, and neither is a run that
    // made one call at a time.
    // A COLON, not a period: the refusal's sentence opens in lower case, as
    // every error message here does, so it continues the heading.
    console.log(`NOTHING IN FLIGHT: ${error.message}, which is not the same as a run that made one call at a time.`,);
  }
}

//endregion Run timing report run
