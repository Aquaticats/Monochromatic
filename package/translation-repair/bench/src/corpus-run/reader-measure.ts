import {
  fetchTransport,
} from '@monochromatic-dev/translation-repair-provider-stream/ts';
import { createOpenRouterClient, } from '@monochromatic-dev/translation-repair-provider/ts';
import {
  type CommandLineOf,
  reportingRefusals,
  RUN_PER_CALL_TIMEOUT_MS,
} from '@monochromatic-dev/translation-repair-run/ts';
import {
  ocrReaderOver,
  runInstalledProgram,
} from '@monochromatic-dev/translation-repair-slice-reading/ts';
import { readMeasureAsk, } from './reader-measure-ask.ts';
import { runReaderMeasure, } from './reader-measure-command.ts';

//region Reader measure
// MEASURES A CANDIDATE PICTURE READER THROUGH OPENROUTER against the readings
// the pipeline already stored, by the two-clause rule of 2026-09-08. This is
// the step that lifts a card's `openrouter-pictures-unmeasured` hold:
// `reader-measure-ask.ts` reads the command line, `reader-measure-records.ts`
// the trusted readings, `reader-measure-run.ts` the loop,
// `reader-measure-score.ts` the clauses and `reader-measure-report.ts` what is
// printed.
//
// SPENDS A KEY: it asks the candidate once per matched picture (nine by
// default) over OpenRouter with zero data retention, through a client built here
// and not through the router, because the router keeps pictures off this route
// until the hold is lifted.
//
// usage: mise run //package/translation-repair/bench:reader-measure -- --trusted <runs dir> --pictures <people dir>

/**
 How long the whole measurement may run before it is abandoned: a quarter of an
 hour for each of the default nine pictures.
 */
const MEASURE_TIMEOUT_MS = 8_100_000;

/**
 Prints one line of the report. Raw output, deliberately: a report a person
 reads, not pipeline logging.

 @param text - the line

 @example
 ```ts
 printLine('MEASURED: gemma-4-26b-a4b-it',);
 ```
 */
function printLine(text: string,): void {
  console.log(text,);
}

/**
 Measures the candidate over the process's own environment.

 @param line - the command line, read whole by `reportingRefusals`

 @example
 ```ts
 await measure({ line, },);
 ```
 */
async function measure({ line, }: { readonly line: CommandLineOf<'reader-measure'>; },): Promise<void> {
  /**
   What to measure, with what.
   */
  const ask = readMeasureAsk({
    line,
    env: process.env,
  },);
  await runReaderMeasure({
    ask,
    client: createOpenRouterClient({
      apiKey: ask.apiKey,
      transport: fetchTransport,
    },),
    readOcr: ocrReaderOver({ runProgram: runInstalledProgram, },),
    signal: AbortSignal.timeout(MEASURE_TIMEOUT_MS,),
    perCallTimeoutMs: RUN_PER_CALL_TIMEOUT_MS,
    print: printLine,
  },);
}

if (import.meta.main)
  await reportingRefusals({
    what: 'reader-measure',
    argv: process.argv,
    env: process.env,
    run: measure,
  },);

//endregion Reader measure
