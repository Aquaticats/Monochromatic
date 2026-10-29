import {
  logger,
  tagged,
} from '@monochromatic-dev/logger-core/ts';

import { StatedRefusalError, } from '@monochromatic-dev/translation-repair-glossary/ts';
import { wordForCount, } from '@monochromatic-dev/translation-repair-typography/ts';
import type { ModelCaller, } from '@monochromatic-dev/translation-repair-provider/ts';
import type { OcrReader, } from '@monochromatic-dev/translation-repair-slice-reading/ts';
import type { MeasureAsk, } from './reader-measure-ask.ts';
import { loadPictures, } from './reader-measure-pictures.ts';
import { renderMeasureReport, } from './reader-measure-report.ts';
import { readTrustedRecords, } from './reader-measure-records.ts';
import {
  measureReader,
  type MeasuredRow,
} from './reader-measure-run.ts';
import {
  seatedBandOf,
  candidateRatesOf,
  type TrustedRecord,
  verdictOf,
} from './reader-measure-score.ts';

//region Reader measure command
// THE WHOLE MEASUREMENT, with its client, deterministic reader and output
// handed in, so a test runs it over scripted pieces and the entry
// (`reader-measure.ts`) hands in the real ones: the OpenRouter client built
// DIRECTLY (not through the router, whose picture route stays shut by the card's
// hold until this measurement passes), the installed OCR and the console.

/**
 The line that says what the candidate gave for one picture.

 @param row - matched picture and candidate's outcome

 @returns Picture's name, then the length of the reading or why there is none

 @example
 ```ts
 const text = rowText({ row, },);
 ```
 */
function rowText({ row, }: { readonly row: MeasuredRow; },): string {
  if (row.candidate
    .kind
    === 'nothing')
    return `${row.assetName}: nothing (${row.candidate
      .reason})`;

  /**
   Length of the reading.
   */
  const characters = row.candidate
    .text
    .length;
  return `${row.assetName}: ${String(characters,)} ${
    wordForCount({
      count: characters,
      one: 'character',
      many: 'characters',
    },)
  }`;
}

/**
 Runs the measurement and prints what it found.

 @param ask - model, directories, sample and key

 @param client - client the candidate is asked through

 @param readOcr - deterministic reader that screens each picture

 @param signal - abort for the whole measurement

 @param perCallTimeoutMs - how long one ask may run

 @param print - where each line of the report goes

 @throws {@link StatedRefusalError} When no trusted reading or no picture is found
 under the directories named

 @example
 ```ts
 await runReaderMeasure({ ask, client, readOcr, signal, perCallTimeoutMs: 360_000, print, },);
 ```
 */
export async function runReaderMeasure(
  {
    ask,
    client,
    readOcr,
    signal,
    perCallTimeoutMs,
    print,
  }: {
    readonly ask: MeasureAsk;
    readonly client: ModelCaller;
    readonly readOcr: OcrReader;
    readonly signal: AbortSignal;
    readonly perCallTimeoutMs: number;
    readonly print: (line: string,) => void;
  },
): Promise<void> {
  /**
   Logger tagged for this measurement.
   */
  const l = tagged({
    tag: runReaderMeasure.name,
    l: logger,
  },);

  /**
   Readings the pipeline already trusts.
   */
  const records = await readTrustedRecords({ dirs: ask.trustedDirs, },);
  if (records.length === 0) {
    throw new StatedRefusalError({
      says: `no stored picture reading with two or more readings was found under ${ask.trustedDirs
        .join(', ',)}`,
    },);
  }

  /**
   Pictures to try.
   */
  const pictures = await loadPictures({ dir: ask.picturesDir, },);
  if (pictures.length === 0) {
    throw new StatedRefusalError({
      says: `no picture file was found under ${ask.picturesDir}`,
    },);
  }
  l.info(`${String(records.length,)} trusted ${
    wordForCount({
      count: records.length,
      one: 'record',
      many: 'records',
    },)
  }, ${String(pictures.length,)} ${
    wordForCount({
      count: pictures.length,
      one: 'picture',
      many: 'pictures',
    },)
  }`,);

  /**
   One row per matched picture asked.
   */
  const rows = await measureReader({
    client,
    readOcr,
    modelId: ask.modelId,
    records,
    pictures,
    limit: ask.sample,
    signal,
    perCallTimeoutMs,
    l,
  },);
  for (const row of rows) {
    print(rowText({ row, },),);
  }

  /**
   Trusted records of the pictures asked, each once.
   */
  const matchedRecords: TrustedRecord[] = [...new Set(rows.map(function recordOf(row,): TrustedRecord {
    return row.trusted;
  },),),];

  /**
   How the seated readers corroborate one another over those pictures.
   */
  const band = seatedBandOf({ records: matchedRecords, },);

  /**
   How the candidate's produced readings fared.
   */
  const candidate = candidateRatesOf({
    rows: rows.flatMap(function producedOf(row,) {
      return (row.candidate
        .kind
        === 'read')
        ? [{
          candidate: row.candidate
            .text,
          trusted: row.trusted,
        },]
        : [];
    },),
  },);
  for (const line of renderMeasureReport({
    modelId: ask.modelId,
    matchedAssets: rows.map(function nameOf(row,): string {
      return row.assetName;
    },),
    sample: ask.sample,
    trustedRecords: records.length,
    pictures: pictures.length,
    band,
    candidate,
    verdict: verdictOf({
      band,
      candidate,
    },),
  },)) {
    print(line,);
  }
}

//endregion Reader measure command
