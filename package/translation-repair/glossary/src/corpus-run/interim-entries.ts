import { createHash, } from 'node:crypto';
import {
  mkdir,
  readFile,
  rm,
} from 'node:fs/promises';
import { join, } from 'node:path';

import { tagged, } from '@monochromatic-dev/logger-core/ts';

import {
  allInInputOrder,
  isJsonRecord,
} from '@monochromatic-dev/translation-repair-document-structure/ts';
import {
  contextRoot,
  rethrowUnlessMissingPath,
} from '@monochromatic-dev/translation-repair-typography/ts';
import { textsInCodePointOrder, } from '@monochromatic-dev/translation-repair-markup/ts';
import { writeFileAtomic, } from './atomic-write.ts';
import { presentNamesOfKind, } from './directory-listing.ts';

//region Interim entries
// WHAT A PASS LEAVES BEHIND FOR AN ENTRY IT SHIPPED A PAGE FOR AND KEPT OPEN. The owner's answer of 2026-10-29 ("Ship
// interim, resumable"): when every provider goes dry in the middle of a page, the run ships the settled slices, the
// archive text and the original under its note, and the entry stays unfinished so a later run replaces the page.
//
// BESIDE THE ARTIFACTS, NOT AMONG THEM, for the reason a decline record sits beside them (`declined-entries.ts`): the
// artifact is the sentinel that an entry is done, every reader of `artifacts/` treats a file there as a settled
// two-lane artifact, and the skip set, the republish and the census all read it so. A record in its own directory
// leaves every one of those readers as it was, and the entry stays pending by construction. The page itself is in the
// published tree, where a later run overwrites it; this record says that page is interim, what it is made of, and which
// bytes it was written as.

/**
 Directory under a runs dir holding one record per entry whose page is interim.
 */
export const INTERIM_DIR = 'interim';

/**
 File suffix of an interim record.
 */
const RECORD_SUFFIX = '.json';

/**
 Why a page was shipped before its entry settled; one reason exists today.
 */
export type InterimReason = 'dry-stop';

/**
 What an interim record carries.

 @example
 ```ts
 const record: InterimEntryRecord = {
   id: 'XingZ6020',
   tip: 'abc123',
   pipelineDigest: 'sha256-tree-v1:...',
   corpusSha: 'a41fc60',
   timestamp: '2026-10-29T21:00:00.000Z',
   reason: 'dry-stop',
   settledSlices: 6,
   archiveSlices: 3,
   sourceSlices: 1,
   pageSha256: 'e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855',
 };
 ```
 */
export type InterimEntryRecord = {
  /**
   Entry whose page is interim.
   */
  readonly id: string;

  /**
   Repository head the pass ran at.
   */
  readonly tip: string;

  /**
   Built pipeline that shipped the page.
   */
  readonly pipelineDigest: string;

  /**
   Corpus commit the entry was read at.
   */
  readonly corpusSha: string;

  /**
   When the page was shipped.
   */
  readonly timestamp: string;

  /**
   Why the page is interim.
   */
  readonly reason: InterimReason;

  /**
   Slices of the page that carry a wording a consolidation record settled.
   */
  readonly settledSlices: number;

  /**
   Slices of the page that carry the archive's own text because nothing settled them.
   */
  readonly archiveSlices: number;

  /**
   Slices of the page that carry the original under the note that it has not been translated yet.
   */
  readonly sourceSlices: number;

  /**
   SHA-256 of the page as written, in lowercase hexadecimal, so a reader can tell that page from any other that later
   stands at the same path.
   */
  readonly pageSha256: string;
};

/**
 Logger root for the interim records.
 */
const root = contextRoot({ tag: 'translation-repair', },);

/**
 Digest an interim record holds of its page.

 @param pageText - the page as written

 @returns SHA-256 of its UTF-8 bytes, in lowercase hexadecimal

 @example
 ```ts
 const pageSha256 = interimPageDigest({ pageText, },);
 ```
 */
export function interimPageDigest({ pageText, }: { readonly pageText: string; },): string {
  return createHash('sha256',)
    .update(pageText,)
    .digest('hex',);
}

/**
 Path of an entry's interim record.

 @param interimDir - directory of interim records

 @param entryId - entry the record is for

 @returns Absolute path of the record

 @example
 ```ts
 const path = interimRecordPath({ interimDir, entryId: 'Kitten', },);
 ```
 */
function interimRecordPath(
  {
    interimDir,
    entryId,
  }: {
    readonly interimDir: string;
    readonly entryId: string;
  },
): string {
  return join(
    interimDir,
    `${entryId}${RECORD_SUFFIX}`,
  );
}

/**
 Writes one interim record where the next pass and `verify-published` will find it.

 WRITTEN AFTER THE PAGE it describes, so a record never names a page that was not written; a crash between the two
 leaves a page with no record, which `verify-published` reports as a page with no artifact.

 @param interimDir - directory of interim records, created if absent

 @param record - what to record

 @example
 ```ts
 await writeInterimEntry({ interimDir, record, },);
 ```
 */
export async function writeInterimEntry(
  {
    interimDir,
    record,
  }: {
    readonly interimDir: string;
    readonly record: InterimEntryRecord;
  },
): Promise<void> {
  // CREATED ON THE FIRST INTERIM SHIP rather than at startup: a runs dir whose pass stopped nowhere carries no such
  // directory, and the reader treats its absence as no interim pages.
  await mkdir(
    interimDir,
    { recursive: true, },
  );
  await writeFileAtomic({
    path: interimRecordPath({
      interimDir,
      entryId: record.id,
    },),
    text: `${JSON.stringify(
      record,
      undefined,
      2,
    )}\n`,
  },);
}

/**
 Removes an entry's interim record, once the entry has settled and its artifact stands.

 @param interimDir - directory of interim records

 @param entryId - entry whose record goes

 @example
 ```ts
 await removeInterimEntry({ interimDir, entryId: 'Kitten', },);
 ```
 */
export async function removeInterimEntry(
  {
    interimDir,
    entryId,
  }: {
    readonly interimDir: string;
    readonly entryId: string;
  },
): Promise<void> {
  // `force` reads an absent record as removed, which is the common case: most entries never had one.
  await rm(
    interimRecordPath({
      interimDir,
      entryId,
    },),
    { force: true, },
  );
}

/**
 Whether a parsed value is a whole number of at least zero.

 @param value - what the record holds for a count

 @returns Whether it is a safe integer from zero

 @example
 ```ts
 const usable = isCount(1.5,); // => false
 ```
 */
function isCount(value: unknown,): value is number {
  if ((typeof value) !== 'number')
    return false;
  return Number.isSafeInteger(value,) && (value >= 0);
}

/**
 Whether a parsed object is an interim record, every field present and of its kind.

 @param fields - the parsed object

 @returns Whether it is a record

 @example
 ```ts
 if (isInterimRecord(fields,)) use(fields.settledSlices,);
 ```
 */
function isInterimRecord(fields: Readonly<Record<string, unknown>>,): fields is InterimEntryRecord {
  return ((typeof fields.id) === 'string')
    && ((typeof fields.tip) === 'string')
    && ((typeof fields.pipelineDigest) === 'string')
    && ((typeof fields.corpusSha) === 'string')
    && ((typeof fields.timestamp) === 'string')
    && (fields.reason === 'dry-stop')
    && isCount(fields.settledSlices,)
    && isCount(fields.archiveSlices,)
    && isCount(fields.sourceSlices,)
    && ((typeof fields.pageSha256) === 'string');
}

/**
 What reading one record file came to.

 @example
 ```ts
 const read: RecordRead = { kind: 'gone', };
 ```
 */
type RecordRead =
  | {
    /**
     The file held the record of its entry.
     */
    readonly kind: 'record';

    /**
     The record.
     */
    readonly record: InterimEntryRecord;
  }
    | {
    /**
     The file is gone, or holds no record of its entry, and is left out.
     */
      readonly kind: 'none';
    };

/**
 What reading the text of one record file came to.

 @example
 ```ts
 const read: RecordText = { kind: 'gone', };
 ```
 */
type RecordText =
  | {
    /**
     The file was read.
     */
    readonly kind: 'text';

    /**
     Its text.
     */
    readonly text: string;
  }
    | {
    /**
     The file is gone.
     */
      readonly kind: 'gone';
    };

/**
 Reads the text of one record file.

 @param interimDir - directory of interim records

 @param entryId - entry whose record to read

 @returns The text, or that the file is gone

 @throws What the read raises for any failure but an absent file

 @example
 ```ts
 const read = await readRecordText({ interimDir, entryId: 'Kitten', },);
 ```
 */
async function readRecordText(
  {
    interimDir,
    entryId,
  }: {
    readonly interimDir: string;
    readonly entryId: string;
  },
): Promise<RecordText> {
  try {
    return {
      kind: 'text',
      text: await readFile(
        interimRecordPath({
          interimDir,
          entryId,
        },),
        'utf8',
      ),
    };
  }
  catch (error) {
    rethrowUnlessMissingPath({ error, },);
    return { kind: 'gone', };
  }
}

/**
 Parses a record file's text, naming the entry in the log where it is no JSON.

 @param text - the file's text

 @param entryId - entry the file is named for

 @returns The parsed value, which is not an object where the text does not parse

 @example
 ```ts
 const parsed = parsedRecordText({ text, entryId: 'Kitten', },);
 ```
 */
function parsedRecordText(
  {
    text,
    entryId,
  }: {
    readonly text: string;
    readonly entryId: string;
  },
): unknown {
  try {
    return JSON.parse(text,);
  }
  catch (error) {
    tagged({
      tag: parsedRecordText.name,
      l: root,
    },)
      .warn(`interim record of ${entryId} is not JSON, so it is read as none (${String(error,)})`,);
    return false;
  }
}

/**
 Reads one record file, naming the entry in the log where the file is no record.

 @param interimDir - directory of interim records

 @param entryId - entry whose record to read

 @returns The record, or that there is none to read

 @throws What the read raises for any failure but an absent file

 @example
 ```ts
 const read = await readRecordFile({ interimDir, entryId: 'Kitten', },);
 ```
 */
async function readRecordFile(
  {
    interimDir,
    entryId,
  }: {
    readonly interimDir: string;
    readonly entryId: string;
  },
): Promise<RecordRead> {
  /**
   The file's text, absent when it went between the listing and the read.
   */
  const read = await readRecordText({
    interimDir,
    entryId,
  },);
  if (read.kind === 'gone')
    return { kind: 'none', };
  /**
   The file's text as a value.
   */
  const parsed = parsedRecordText({
    text: read.text,
    entryId,
  },);
  if (!isJsonRecord(parsed,)) {
    tagged({
      tag: readRecordFile.name,
      l: root,
    },)
      .warn(`interim record of ${entryId} is no object, so it is read as none`,);
    return { kind: 'none', };
  }
  if ((!isInterimRecord(parsed,)) || (parsed.id !== entryId)) {
    tagged({
      tag: readRecordFile.name,
      l: root,
    },)
      .warn(`interim record of ${entryId} is not the record of that entry, so it is read as none`,);
    return { kind: 'none', };
  }
  return {
    kind: 'record',
    record: parsed,
  };
}

/**
 Reads every interim record a runs dir carries, in entry-id order.

 An absent directory is no interim pages. A file that is not the record its name says is left out and named in the log,
 so one damaged file costs the report of its own page and no other.

 @param interimDir - directory of interim records

 @returns Records by entry id, in code-point order of the ids

 @throws What listing the directory raises for any failure but an absent directory

 @example
 ```ts
 const interim = await readInterimEntries({ interimDir, },);
 ```
 */
export async function readInterimEntries(
  { interimDir, }: { readonly interimDir: string; },
): Promise<ReadonlyMap<string, InterimEntryRecord>> {
  /**
   Ids the directory holds a file for.
   */
  const ids = textsInCodePointOrder({ texts: (await presentNamesOfKind({
    dir: interimDir,
    kind: 'file',
  },))
    .filter(function isRecord(name,): boolean {
      return name.endsWith(RECORD_SUFFIX,);
    },)
    .map(function toId(name,): string {
      return name.slice(
        0,
        -RECORD_SUFFIX.length,
      );
    },), },);
  /**
   What reading each id's file came to.
   */
  const reads = await allInInputOrder({
    members: ids.map(function readOne(entryId,): Promise<RecordRead> {
      return readRecordFile({
        interimDir,
        entryId,
      },);
    },),
  },);
  return new Map(reads.flatMap(function entryOf(read,): readonly (readonly [
    string,
    InterimEntryRecord,
  ])[] {
    if (read.kind === 'none')
      return [];
    /**
     The record read, which names the entry its file is named for.
     */
    const { record, } = read;
    return [
      [
        record.id,
        record,
      ],
    ];
  },),);
}

//endregion Interim entries
