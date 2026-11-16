/**
 Where an entry whose start failed is taken out of the way: one JSON file per entry beside the runs, holding the entry itself, the reason
 its start failed, and when, so a person and a session both find it where every other record of a run is.

 The record is written before the entry leaves the queue, and each of the two is one atomic step on one file: the record is a file renamed
 over its name, and the entry's own file is unlinked after it. A kill between the two leaves the entry in both places, which loses nothing:
 the next pass's settlement reads a record whose identity the queue still holds under that name as an unfinished set-aside and finishes it.
 The other order would leave a kill with the entry out of the queue and nowhere else.

 A put-back that finds the entry still queued leaves the record alone, and the settlement decides it, because removing the record there is
 how an entry in flight was lost.

 @module
 */

import {
  access,
  readdir,
  readFile,
  rename,
  rm,
  writeFile,
} from 'node:fs/promises';
import { join, } from 'node:path';
import { completeTheClaim, } from './claiming-an-entry-of-a-queue.ts';
import { errorCodeOf, } from './error-code-of.ts';
import {
  isQueueRecord,
  anEntryNameInAFileName,
  QUEUE_ID_FIELD,
  type QueueRecord,
  queueIdOf,
} from './queue-of-runs.ts';
import {
  NO_NAME_IN_AN_ESCAPED_FORM,
  anEscapedName,
} from './names-a-queue-gives-its-things.ts';
import { readQueueOfRuns, } from './reading-and-writing-the-folder-a-queue-is.ts';

/**
 The code the operating system gives a path that is not there.
 */
const MISSING_PATH_CODE = 'ENOENT';

/**
 The end of the file one entry's set-aside record is written to, which sits beside the runs under the entry's name.
 */
export const SET_ASIDE_FILE_SUFFIX = '-set-aside.json';

/**
 The end of the file a record is written through, which is renamed over the record.
 */
const BEING_WRITTEN_SUFFIX = '.being-written';

/**
 No set-aside record for that entry, as a symbol so no record can be mistaken for one.
 */
export const NO_SET_ASIDE_RECORD: unique symbol = Symbol('runs folder holds no set-aside record for that entry',);

/**
 What one entry's set-aside record says.
 */
export type SetAsideRecord = {
  /**
   The entry's name.
   */
  readonly name: string;

  /**
   The entry as the queue held it, so putting it back needs nothing else.
   */
  readonly entry: QueueRecord;

  /**
   Why its start failed, as the start said it.
   */
  readonly reason: string;

  /**
   When it was set aside, as ISO 8601 with its offset.
   */
  readonly setAsideAt: string;

  /**
   Where the whole of what the failed start printed was kept, when it was kept.
   */
  readonly keptOutputAt?: string;
};

/**
 A set-aside record that cannot be read as one.
 */
export class SetAsideRecordError extends Error {
  /**
   The name printed with the message.
   */
  override readonly name = 'SetAsideRecordError';
}

/**
 Reads the file one entry's set-aside record is written to.

 @param runsFolder - folder the driver writes its runs into

 @param name - the entry's name

 @returns the record's path

 @example
 ```ts
 const file = setAsideFileOf({ runsFolder: '/runs', name: 'a' });
 ```
 */
export function setAsideFileOf({
  runsFolder,
  name,
}: {
  readonly runsFolder: string;
  readonly name: string;
},): string {
  return join(
    runsFolder,
    `${anEscapedName({ name, },)}${SET_ASIDE_FILE_SUFFIX}`,
  );
}

/**
 Says whether a file name is one entry's set-aside record.

 @param file - the file's name

 @returns whether the file holds a set-aside record

 @example
 ```ts
 const isOne = isASetAsideRecordFile({ file: 'a-set-aside.json' });
 ```
 */
export function isASetAsideRecordFile({ file, }: { readonly file: string },): boolean {
  return file.endsWith(SET_ASIDE_FILE_SUFFIX,);
}

/**
 Reads one set-aside record's text.

 @param shown - the file's text

 @param file - the file's path, for the message

 @returns the record

 @throws SetAsideRecordError when the text holds no entry, reason and time

 @example
 ```ts
 const record = setAsideRecordIn({ shown: '{"name":"a","entry":{"name":"a"},"reason":"x","setAsideAt":"2026-11-14T00:00:00Z"}', file: 'a-set-aside.json' });
 ```
 */
export function setAsideRecordIn({
  shown,
  file,
}: {
  readonly shown: string;
  readonly file: string;
},): SetAsideRecord {
  /**
   The record, of any shape.
   */
  const parsed: unknown = JSON.parse(shown,);
  if (
    ((typeof parsed) !== 'object')
      || (parsed === null)
      || (!('name' in parsed))
      || (!('reason' in parsed))
      || (!('setAsideAt' in parsed))
      || (!('entry' in parsed))
      || ((typeof parsed.name) !== 'string')
      || ((typeof parsed.reason) !== 'string')
      || ((typeof parsed.setAsideAt) !== 'string')
      || (!isQueueRecord(parsed.entry,))
  ) {
    throw new SetAsideRecordError(
      `${file} names no entry, reason and time, so the entry it set aside cannot be read or put back`,
    );
  }
  return {
    name: parsed.name,
    entry: parsed.entry,
    reason: parsed.reason,
    setAsideAt: parsed.setAsideAt,
    ...((('keptOutputAt' in parsed) && ((typeof parsed.keptOutputAt) === 'string'))
      ? { keptOutputAt: parsed.keptOutputAt, }
      : {}),
  };
}

/**
 Writes one entry's set-aside record, through a file beside it that is renamed over it, and never over another entry's only copy.

 A name may be queued again once its earlier entry has gone, and the earlier entry's record is then the only copy of what that entry was
 asked to do. Writing the new entry's record straight over it would lose that copy, so a record holding another entry is renamed to a name
 of its own first, which a person and this package's own reader both still find, and the new record takes the plain name.

 @param runsFolder - folder the driver writes its runs into

 @param record - what the record says

 @returns the record's path, and where a record of another entry was kept, when one was there

 @throws SetAsideRecordError when the record cannot be written

 @example
 ```ts
 const { file } = await writeSetAsideRecord({ runsFolder: '/runs', record });
 ```
 */
export async function writeSetAsideRecord({
  runsFolder,
  record,
}: {
  readonly runsFolder: string;
  readonly record: SetAsideRecord;
},): Promise<{
  readonly file: string;
  readonly keptTheEarlierRecordAt?: string;
}> {
  /**
   The record's path.
   */
  const file = setAsideFileOf({
    runsFolder,
    name: record.name,
  },);
  /**
   Where a record of another entry of that name was kept, or that the name held none.
   */
  const kept = await anotherEntrysRecordKept({
    runsFolder,
    name: record.name,
    record,
    file,
  },);
  /**
   The file written beside it and renamed over it.
   */
  const beside = `${file}${BEING_WRITTEN_SUFFIX}-${String(process.pid,)}`;
  try {
    await writeFile(
      beside,
      `${JSON.stringify(
        record,
        undefined,
        2,
      )}\n`,
    );
    await rename(
      beside,
      file,
    );
    return {
      file,
      ...(('keptAt' in kept) ? { keptTheEarlierRecordAt: kept.keptAt, } : {}),
    };
  }
  catch (error) {
    throw new SetAsideRecordError(`the set-aside record of ${record.name} cannot be written at ${file}: ${String(error,)}`,);
  }
}

/**
 Keeps a record of another entry that stands under the same name, so that writing this entry's record never destroys it.

 A name may be queued again once its earlier entry has gone, and that entry's record is then its only copy: it is renamed to a name of its
 own, which a person still finds, rather than written over.

 @param runsFolder - folder the driver writes its runs into

 @param name - the entry's name

 @param record - the record about to be written

 @param file - the path that record takes

 @returns where the earlier record was kept, or that this name held no other entry's record

 @throws SetAsideRecordError when the record there cannot be read or renamed

 @example
 ```ts
 const kept = await anotherEntrysRecordKept({ runsFolder: '/runs', name: 'a', record, file: '/runs/a-set-aside.json' });
 ```
 */
async function anotherEntrysRecordKept({
  runsFolder,
  name,
  record,
  file,
}: {
  readonly runsFolder: string;
  readonly name: string;
  readonly record: SetAsideRecord;
  readonly file: string;
},): Promise<{ readonly keptAt: string } | { readonly none: true }> {
  /**
   The record already beside the runs under that name, when there is one.
   */
  const earlier = await setAsideRecordOf({
    runsFolder,
    name,
  },);
  if (
    (earlier === NO_SET_ASIDE_RECORD)
      || (queueIdOf({ record: earlier.entry, },) === queueIdOf({ record: record.entry, },))
  ) {
    return { none: true, };
  }
  /**
   The identity the earlier record's entry carries, which names the file it is kept in.
   */
  const earlierId = queueIdOf({ record: earlier.entry, },);
  /**
   Where the earlier entry's only copy is kept.
   */
  const keptAt = `${file.slice(
    0,
    -SET_ASIDE_FILE_SUFFIX.length,
  )}-set-aside-${((typeof earlierId) === 'string') ? earlierId : 'no-identity'}${SET_ASIDE_FILE_SUFFIX}`;
  await rename(
    file,
    keptAt,
  );
  return { keptAt, };
}

/**
 Reads one entry's set-aside record.

 @param runsFolder - folder the driver writes its runs into

 @param name - the entry's name

 @returns the record, or nothing when the entry is not set aside

 @throws SetAsideRecordError when the record is there and cannot be read as one

 @example
 ```ts
 const record = await setAsideRecordOf({ runsFolder: '/runs', name: 'a' });
 ```
 */
export async function setAsideRecordOf({
  runsFolder,
  name,
}: {
  readonly runsFolder: string;
  readonly name: string;
},): Promise<SetAsideRecord | typeof NO_SET_ASIDE_RECORD> {
  /**
   The record's path.
   */
  const file = setAsideFileOf({
    runsFolder,
    name,
  },);
  try {
    return setAsideRecordIn({
      shown: await readFile(
        file,
        'utf8',
      ),
      file,
    },);
  }
  catch (error) {
    // A record that is not there is exactly what this asks about; one that is there and cannot be read is carried out, because a
    // reader who cannot put the entry back must be told.
    if ((error instanceof SetAsideRecordError) || (errorCodeOf(error,) !== MISSING_PATH_CODE)) {
      throw error;
    }
    return NO_SET_ASIDE_RECORD;
  }
}

/**
 Reads every set-aside record beside the runs.

 @param runsFolder - folder the driver writes its runs into

 @returns one record per entry set aside, in the order the folder lists them

 @throws SetAsideRecordError when one record cannot be read as one

 @example
 ```ts
 const records = await setAsideRecordsIn({ runsFolder: '/runs' });
 ```
 */
export async function setAsideRecordsIn({ runsFolder, }: { readonly runsFolder: string },): Promise<readonly SetAsideRecord[]> {
  /**
   The folder's own files and folders.
   */
  const files = await readdir(runsFolder,);
  /**
   The records read so far.
   */
  const records: SetAsideRecord[] = [];
  for (const file of files.filter(function isARecord(name,) {
    return isASetAsideRecordFile({ file: name, },);
  },)) {
    /**
     The entry's name the file's name was written from.
     */
    const name = anEntryNameInAFileName({
      fileName: `${file.slice(
        0,
        -SET_ASIDE_FILE_SUFFIX.length,
      )}.json`,
    },);
    if (name === NO_NAME_IN_AN_ESCAPED_FORM) {
      // A file named with a suffix this package writes but holding no name of its own is a person's file: it is left alone.
      continue;
    }
    /**
     The record of the file just listed.
     */
    // oxlint-disable-next-line eslint/no-await-in-loop -- one record after another, each with its own error naming its own file
    const record = await setAsideRecordOf({
      runsFolder,
      name,
    },);
    if (record !== NO_SET_ASIDE_RECORD) {
      records.push(record,);
    }
  }
  return records;
}

/**
 Deletes one entry's set-aside record.

 A caller decides on the queue's own state just before it calls this, and the record belongs to the entry that decision was about: the
 settlement removes one only after it has taken the entry out of the queue, and a put-back removes one only after the entry is queued
 again.

 @param runsFolder - folder the driver writes its runs into

 @param name - the entry's name

 @returns whether there was a record to delete

 A record that is not there is nothing to delete, and anything else is a failure a caller must see: an entry whose record could
 not be deleted cannot be put back without being set aside again.

 @example
 ```ts
 const deleted = await removeSetAsideRecord({ runsFolder: '/runs', name: 'a' });
 ```
 */
export async function removeSetAsideRecord({
  runsFolder,
  name,
}: {
  readonly runsFolder: string;
  readonly name: string;
},): Promise<boolean> {
  try {
    await rm(
      setAsideFileOf({
        runsFolder,
        name,
      },),
    );
    return true;
  }
  catch (error) {
    if (errorCodeOf(error,) === MISSING_PATH_CODE) {
      return false;
    }
    throw error;
  }
}

/**
 Says whether one entry is set aside, without reading what its record says.

 @param runsFolder - folder the driver writes its runs into

 @param name - the entry's name

 @returns whether the entry's record is there

 @example
 ```ts
 const setAside = await setAsideRecordIsThere({ runsFolder: '/runs', name: 'a' });
 ```
 */
export async function setAsideRecordIsThere({
  runsFolder,
  name,
}: {
  readonly runsFolder: string;
  readonly name: string;
},): Promise<boolean> {
  try {
    await access(setAsideFileOf({
      runsFolder,
      name,
    },),);
    return true;
  }
  catch (error) {
    // A record that is not there is the answer this asks for.
    if (errorCodeOf(error,) === MISSING_PATH_CODE) {
      return false;
    }
    throw error;
  }
}

/**
 Sets one entry aside: writes its record, then completes the claim that took it out of the queue.

 The record is one step and the queue's copy leaving is another, in that order, so a kill between them leaves an entry in both places and
 the next pass's settlement finishes the set-aside rather than losing it. The entry is taken out by its identity, so an entry queued under
 the same name while this start was in flight is left where it is.

 @param queueFolder - the queue's folder

 @param runsFolder - folder the driver writes its runs into

 @param record - the entry, as the queue held it

 @param queueId - the identity the claim was taken on

 @param reason - why its start failed

 @param keptOutputAt - where the whole of what the failed start printed was kept, when it was kept

 @param now - the reading the record's time is written from

 @returns the record's path, and how many entries the queue holds afterwards

 @throws SetAsideRecordError when the record cannot be written, which leaves the entry and its claim where they are

 @example
 ```ts
 const { file, left } = await setAsideAnEntryOfAQueue({ queueFolder: '/runs/q', runsFolder: '/runs', record, queueId, reason: 'x', now: Date.now });
 ```
 */
export async function setAsideAnEntryOfAQueue({
  queueFolder,
  runsFolder,
  record,
  queueId,
  reason,
  keptOutputAt,
  now,
}: {
  readonly queueFolder: string;
  readonly runsFolder: string;
  readonly record: QueueRecord;
  readonly queueId: string;
  readonly reason: string;
  readonly keptOutputAt?: string;
  readonly now: () => number;
},): Promise<{
  readonly file: string;
  readonly left: number;
}> {
  if (((typeof queueId) !== 'string') || (queueId === '')) {
    // Only an entry a filling claimed has a record that says which entry of that name was set aside; without its identity the record
    // could not be told from a namesake's, and the settlement would refuse to act on it.
    throw new SetAsideRecordError(
      `${record.name} is not set aside: only an entry a filling has claimed has the identity its record must carry, and the queue's copy `
        + 'is taken out by that identity. Nothing changed.',
    );
  }
  // The record first: a kill before the entry leaves the queue leaves the entry queued with its record, which the settlement finishes.
  /**
   The record's path.
   */
  const written = await writeSetAsideRecord({
    runsFolder,
    record: {
      name: record.name,
      entry: {
        ...record,
        [QUEUE_ID_FIELD]: queueId,
      },
      reason,
      setAsideAt: new Date(now(),).toISOString(),
      ...((keptOutputAt === undefined) ? {} : { keptOutputAt, }),
    },
  },);
  /**
   The record's path.
   */
  const {file} = written;
  await completeTheClaim({
    queueFolder,
    name: record.name,
    queueId,
  },);
  return {
    file,
    left: (await readQueueOfRuns({ queueFolder, },)).length,
  };
}
