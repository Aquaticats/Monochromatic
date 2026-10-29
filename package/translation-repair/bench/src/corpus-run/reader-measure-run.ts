import {
  type Logger,
  tagged,
} from '@monochromatic-dev/logger-core/ts';

import type { ModelCaller, } from '@monochromatic-dev/translation-repair-provider/ts';
import type { RosterModelId, } from '@monochromatic-dev/translation-repair-shared-contract/ts';
import {
  type ImageReading,
  type OcrReader,
  readPastRefusal,
} from '@monochromatic-dev/translation-repair-slice-reading/ts';
import {
  matchRecord,
} from './reader-measure-records.ts';
import type { TrustedRecord, } from './reader-measure-score.ts';

//region Reader measure run
// THE LOOP OF `reader-measure`: each picture is screened by the deterministic
// reader, matched to a trusted record, and only a matched picture is put to the
// candidate, up to the number asked for. A picture with no text, or one that
// matches nothing, costs no call.
//
// THE CANDIDATE IS ASKED THROUGH THE PRODUCTION READER STAGE (`readPastRefusal`),
// so a refusal is re-asked and a reply the length limit cut is asked once more,
// exactly as in a pass; what the candidate would do in a pass is what is measured.

/**
 A picture on disk.
 */
export type MeasuredPicture = {
  readonly assetName: string;
  readonly bytes: Uint8Array;
};

/**
 What the candidate gave for one picture.
 */
export type CandidateOutcome = {
  readonly kind: 'read';
  readonly text: string;
} | {
  readonly kind: 'nothing';

  /**
   Why the stage kept no reading.
   */
  readonly reason: string;
};

/**
 One matched picture and what the candidate gave for it.
 */
export type MeasuredRow = {
  readonly assetName: string;
  readonly trusted: TrustedRecord;
  readonly candidate: CandidateOutcome;
};

/**
 Reduces a stage reading to what the measurement keeps.

 @param reading - what the reader stage returned

 @returns Text when the stage kept a reading, else the reason it did not

 @example
 ```ts
 const outcome = outcomeOf({ reading, },);
 ```
 */
function outcomeOf({ reading, }: { readonly reading: ImageReading; },): CandidateOutcome {
  if (reading.kind === 'read') {
    return {
      kind: 'read',
      text: reading.text,
    };
  }
  return {
    kind: 'nothing',
    reason: (reading.kind === 'short') ? 'too-short' : reading.reason,
  };
}

/**
 Puts the matched pictures to the candidate.

 @param client - client the candidate is asked through

 @param readOcr - deterministic reader that screens and matches each picture

 @param modelId - candidate reader

 @param records - trusted records pictures are matched to

 @param pictures - pictures to try, in order

 @param limit - how many matched pictures to ask the candidate about

 @param signal - abort for the whole measurement

 @param perCallTimeoutMs - how long one ask may run

 @param l - logger

 @returns One row per matched picture asked, in order

 @example
 ```ts
 const rows = await measureReader({ client, readOcr, modelId, records, pictures, limit: 9, signal, perCallTimeoutMs, l, },);
 ```
 */
export async function measureReader(
  {
    client,
    readOcr,
    modelId,
    records,
    pictures,
    limit,
    signal,
    perCallTimeoutMs,
    l: parent,
  }: {
    readonly client: ModelCaller;
    readonly readOcr: OcrReader;
    readonly modelId: RosterModelId;
    readonly records: readonly TrustedRecord[];
    readonly pictures: readonly MeasuredPicture[];
    readonly limit: number;
    readonly signal: AbortSignal;
    readonly perCallTimeoutMs: number;
    readonly l: Logger;
  },
): Promise<readonly MeasuredRow[]> {
  /**
   Logger tagged for this loop.
   */
  const l = tagged({
    tag: measureReader.name,
    l: parent,
  },);

  /**
   Rows so far.
   */
  const rows: MeasuredRow[] = [];
  for (const picture of pictures) {
    if (rows.length >= limit)
      break;

    /* oxlint-disable no-await-in-loop -- one picture at a time on purpose: a measurement, and concurrent calls would let a rate limit read as a reader fault */
    /**
     What the deterministic reader found in the picture.
     */
    const ocr = await readOcr({
      bytes: picture.bytes,
      assetName: picture.assetName,
      l,
    },);
    if (ocr.kind !== 'read') {
      l.info(`${picture.assetName}: no deterministic text, not asked`,);
      continue;
    }

    /**
     Record the deterministic text matches.
     */
    const match = matchRecord({
      ocrText: ocr.text,
      records,
    },);
    if (match.kind === 'unmatched') {
      l.info(`${picture.assetName}: matches no trusted record, not asked`,);
      continue;
    }

    /**
     What the production reader stage keeps of the candidate's answer.
     */
    const reading = await readPastRefusal({
      client,
      modelId,
      bytes: picture.bytes,
      assetName: picture.assetName,
      signal,
      perCallTimeoutMs,
      l,
    },);
    rows.push({
      assetName: picture.assetName,
      trusted: match.record,
      candidate: outcomeOf({ reading, },),
    },);
    /* oxlint-enable no-await-in-loop */
  }
  return rows;
}

//endregion Reader measure run
