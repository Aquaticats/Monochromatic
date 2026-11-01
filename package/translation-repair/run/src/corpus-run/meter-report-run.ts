import { PROVIDER_ORDER, } from '@monochromatic-dev/translation-repair-shared-contract/ts';
import type { CommandLineOf, } from './command-lines.ts';
import { reportProvider, } from './meter-report-provider.ts';
import { readLogTexts, } from './report-log-read.ts';
import {
  spanText,
  stampText,
} from './meter-report-text.ts';
import {
  type MeterLogReading,
  type MeterSample,
  readMeterLog,
} from './meter-sample-read.ts';

//region Meter report run
// The meter report's whole procedure: read every named log, merge their
// readings, say what window they cover and report each provider in turn.
//
// SPLIT OUT OF `meter-report.ts` so the entry holds only the wiring.

/**
 Exit code returned when the logs held no reading at all.
 */
const NOTHING_RECORDED = 1;

/**
 Exit code returned when the logs held a reading and every provider was
 reported over it.
 */
const REPORTED = 0;

/**
 Merges what every log held into one list of readings in time order.

 @param readings - what each named log yielded

 @returns Every reading once, earliest first

 @example
 ```ts
 const samples = mergeSamples({ readings, },);
 ```
 */
export function mergeSamples({ readings, }: { readonly readings: readonly MeterLogReading[]; },): readonly MeterSample[] {
  return [
    ...new Map(readings
      .flatMap(function toSamples(reading,): readonly MeterSample[] {
        return reading.samples;
      },)
      .map(function keyed(sample,): readonly [
        string,
        MeterSample,
      ] {
        return [
          // EVERY FIELD OF THE READING, as a list the text keeps apart: a
          // key of the stamp and the first two states alone took two readings
          // that differ in the third or fourth provider's state, or in a
          // number, for one reading, and a key that joined the numbers took
          // one field for two.
          JSON.stringify([
            sample.at,
            sample.neuralwatt,
            sample.synthetic,
            sample.hyper,
            sample.openrouter,
            sample.bedrock,
            sample.deepseek,
            sample.xiaomi,
            sample.anthropic,
            sample.levels,
          ],),
          sample,
        ];
      },),).values(),
  ].toSorted(function byTime(
    left,
    right,
  ): number {
    return left.at - right.at;
  },);
}

/**
 Reads every named log and reports both providers.

 The report on stdout and the returned exit code ARE the output; the entry
 file sets the code on the process, so a case reads it without starting one.

 @param line - the report's command line, read whole by `reportingRefusals`,
 which refuses it when no log is named: any log a pass, probe or sample wrote
 will do, and passing several merges them into one record

 @returns The exit code the command leaves: 1 when the logs held no reading,
 0 once every provider was reported

 @example
 ```ts
 process.exitCode = await reportMeters({ line, },);
 ```
 */
export async function reportMeters({ line, }: { readonly line: CommandLineOf<'meter-report'>; },): Promise<number> {
  /**
   Logs to read, named on the command line.
   */
  const paths = line.positionals;

  /**
   Everything every named log held.
   */
  const readings = (await readLogTexts({ paths, },))
    .map(function read(text,): MeterLogReading {
      return readMeterLog({ text, },);
    },);

  /**
   Every reading from every log, with exact repeats collapsed so passing one
   log twice cannot double its weight.
   */
  const samples = mergeSamples({ readings, },);

  /**
   Marked lines no log would yield a reading from.
   */
  const skipped = readings.reduce(
    function addSkipped(
      total,
      reading,
    ): number {
      return total + reading.skippedLines;
    },
    0,
  );

  console.log(
    `meter-report: logs=${String(paths.length,)} readings=${String(samples.length,)} `
      + `unread=${String(skipped,)}`,
  );

  /**
   Earliest reading, which opens the window every figure of this report sits in.
   */
  const first = samples.at(0,);

  /**
   Latest reading, which closes it.
   */
  const last = samples.at(-1,);

  if ((first === undefined) || (last === undefined)) {
    console.log(
      '  NOTHING RECORDED. These logs carry no availability reading. Runs written before the '
        + 'reading was promoted out of debug level have none, so read a log from a pass or a '
        + '`budget-sample` taken after that landed.',
    );
    return NOTHING_RECORDED;
  }

  console.log(
    `  window ${stampText({ at: first.at, },)} .. ${stampText({ at: last.at, },)} `
      + `(${spanText({ ms: last.at - first.at, },)})`,
  );
  console.log(
    '  READINGS HAPPEN WHEN A RUN ASKS FOR ONE, so this window is dense while work ran and '
      + 'empty otherwise. Every figure in this report is availability WHEN WE WERE ASKING.',
  );

  // EVERY PROVIDER, in the order the record names them; one absent from a
  // sample contributes no reading to its series.
  for (const provider of PROVIDER_ORDER) {
    reportProvider({
      samples,
      provider,
    },);
  }
  return REPORTED;
}

//endregion Meter report run
