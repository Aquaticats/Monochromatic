import { readFile, } from 'node:fs/promises';
import {
  basename,
  join,
} from 'node:path';

import {
  logger,
  tagged,
} from '@monochromatic-dev/logger-core/ts';

import { isJsonRecord, } from '@monochromatic-dev/translation-repair-document-structure/ts';
import {
  compareCodePoints,
  CORROBORATION_TRIGRAM_SHARE,
  errorName,
  trigramOverlap,
} from '@monochromatic-dev/translation-repair-markup/ts';
import { filesUnder, } from './reader-measure-files.ts';
import type {
  TrustedReading,
  TrustedRecord,
} from './reader-measure-score.ts';

//region Reader measure records
// THE READINGS THE PIPELINE ALREADY STORED, as the yardstick a candidate reader
// is held to. A picture's reading is cached per entry as
// `slice-cache/<entry>/picture.<key>.json` holding `{cacheKey, record}`; a
// `corroborated` record and a stable `unavailable` record (the readers
// disagreed) both keep every reading, labelled by model.
//
// THE RECORDS DO NOT NAME THEIR PICTURE. The cache key is a digest of the
// picture's bytes together with the roster that read it, so no picture can be
// looked up from a key. A picture is matched to the record whose readings its
// deterministic reading overlaps the most, at the corroboration share or more,
// as the 2026-09-08 instrument matched them.

/**
 A record with the text of its readings, which tells two records apart.
 */
type KeyedRecord = {
  readonly key: string;
  readonly record: TrustedRecord;
};

/**
 A record with how far its readings overlap a deterministic reading.
 */
type ScoredRecord = {
  readonly record: TrustedRecord;
  readonly overlap: number;
};

/**
 Whether a value is one model's reading.

 @param value - element of a stored list of readings

 @returns Whether it names a model and carries text

 @example
 ```ts
 const kept = readings.filter(isTrustedReading,);
 ```
 */
function isTrustedReading(value: unknown,): value is TrustedReading {
  return isJsonRecord(value,) && ((typeof value.modelId) === 'string')
    && ((typeof value.text) === 'string');
}

/**
 The readings a stored file keeps, when it keeps at least two.

 @param content - parsed content of one cache file

 @returns Readings, or none when the file holds another kind of record or
 fewer than two readings

 @example
 ```ts
 const readings = readingsKeptBy({ content, },);
 ```
 */
function readingsKeptBy({ content, }: { readonly content: unknown; },): TrustedRecord {
  if ((!isJsonRecord(content,)) || (!isJsonRecord(content.record,)))
    return [];

  /**
   The stored record.
   */
  const { record, } = content;
  if ((record.kind !== 'corroborated') && (record.kind !== 'unavailable'))
    return [];
  if (!Array.isArray(record.readings,))
    return [];

  /**
   Readings that name their model and carry text.
   */
  const readings = record.readings
    .filter(isTrustedReading,);
  return (readings.length >= 2) ? readings : [];
}

/**
 Every picture-reading file under a directory, at any depth.

 @param dir - directory to search

 @returns Absolute paths of the files named `picture.<key>.json`

 @example
 ```ts
 const files = await pictureFilesUnder({ dir, },);
 ```
 */
async function pictureFilesUnder({ dir, }: { readonly dir: string; },): Promise<readonly string[]> {
  return (await filesUnder({ dir, },))
    .filter(function isPictureFile(path,): boolean {
      /**
       File name without its directories.
       */
      const name = basename(path,);
      return name.startsWith('picture.',) && name.endsWith('.json',);
    },)
    .map(function absolute(path,): string {
      return join(
        dir,
        path,
      );
    },);
}

/**
 Reads the trusted records under some directories.

 @param dirs - directories holding runs directories, searched at any depth

 @returns One record per distinct set of readings, in a stable order; a file
 that is no JSON or holds no usable record is skipped and logged

 @example
 ```ts
 const records = await readTrustedRecords({ dirs: ['/runs',], },);
 ```
 */
export async function readTrustedRecords({ dirs, }: { readonly dirs: readonly string[]; },): Promise<readonly TrustedRecord[]> {
  /**
   Logger tagged for this reading.
   */
  const l = tagged({
    tag: readTrustedRecords.name,
    l: logger,
  },);

  /**
   Records by the readings that identify them, so a picture stored under more than one key counts once.
   */
  const byReadings = new Map<string, TrustedRecord>();
  for (const dir of dirs) {
    /* oxlint-disable no-await-in-loop -- directories are read one after another so the log reads in order */
    for (const file of await pictureFilesUnder({ dir, },)) {
      try {
        /**
         Readings this file keeps.
         */
        const readings = readingsKeptBy({
          content: JSON.parse(await readFile(
            file,
            'utf8',
          ),) as unknown,
        },);
        /**
         The readings' own text, which tells one record from another.
         */
        const identity = JSON.stringify(readings,);
        if ((readings.length > 0) && (!byReadings.has(identity,))) {
          byReadings.set(
            identity,
            readings,
          );
        }
      }
      catch (error) {
        l.warn(`skipped ${file}: not readable as a stored picture reading (${errorName({ error, },)})`,);
      }
    }
    /* oxlint-enable no-await-in-loop */
  }

  /**
   Records with the readings that identify them, to be put in a stable order.
   */
  const keyed: KeyedRecord[] = [];
  for (const [key, record,] of byReadings)
    keyed.push({
      key,
      record,
    },);

  return keyed
    .toSorted(function byKey(
      left: KeyedRecord,
      right: KeyedRecord,
    ): number {
      return compareCodePoints({
        left: left.key,
        right: right.key,
      },);
    },)
    .map(function recordOf({ record, }: KeyedRecord,): TrustedRecord {
      return record;
    },);
}

/**
 Which record a picture belongs to.

 @example
 ```ts
 const matched: RecordMatch = { kind: 'unmatched', };
 ```
 */
export type RecordMatch = {
  readonly kind: 'matched';
  readonly record: TrustedRecord;
} | {
  readonly kind: 'unmatched';
};

/**
 Matches a picture to the record its deterministic reading overlaps most.

 @param ocrText - what the deterministic reader found in the picture

 @param records - trusted records

 @returns Record with the highest mean trigram overlap, when that is at
 the corroboration share or more; otherwise unmatched

 @example
 ```ts
 const match = matchRecord({ ocrText, records, },);
 ```
 */
export function matchRecord(
  {
    ocrText,
    records,
  }: {
    readonly ocrText: string;
    readonly records: readonly TrustedRecord[];
  },
): RecordMatch {
  /**
   Each record with how far its readings overlap the deterministic reading, on average.
   */
  const scored = records.map(function scoreOf(record,): ScoredRecord {
    return {
      record,
      overlap: record.reduce(
        function total(
          sum,
          reading,
        ): number {
          return sum + trigramOverlap({
            left: ocrText,
            right: reading.text,
          },);
        },
        0,
      ) / record.length,
    };
  },);

  /**
   The best-overlapping record.
   */
  const [best,] = scored.toSorted(function byOverlap(
    left: ScoredRecord,
    right: ScoredRecord,
  ): number {
    return right.overlap - left.overlap;
  },);
  if ((best === undefined) || (best.overlap < CORROBORATION_TRIGRAM_SHARE))
    return { kind: 'unmatched', };
  return {
    kind: 'matched',
    record: best.record,
  };
}

//endregion Reader measure records
