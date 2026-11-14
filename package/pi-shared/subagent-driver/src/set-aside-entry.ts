/**
 Where an entry whose start failed is taken out of the way: one JSON file per entry beside the runs, holding the entry itself, the
 reason its start failed, and when, so a person and a session both find it where every other record of a run is.

 The record is written before the entry is taken out of the queue. A kill between the two therefore leaves the entry in both places,
 which loses nothing: the filling reads a record whose name the queue still holds as an unfinished set-aside, drops the record, and
 starts the entry again. The other order would leave a kill with the entry out of the queue and nowhere else.

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
import { errorCodeOf, } from './error-code-of.ts';
import {
  isQueueRecord,
  putAQueueEntryIn,
  type QueueRecord,
  readQueueOfRuns,
  takeAQueueEntryOut,
} from './queue-of-runs.ts';

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
    `${name}${SET_ASIDE_FILE_SUFFIX}`,
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
 Writes one entry's set-aside record, through a file beside it that is renamed over it.

 @param runsFolder - folder the driver writes its runs into

 @param record - what the record says

 @returns the record's path

 @throws SetAsideRecordError when the record cannot be written

 @example
 ```ts
 await writeSetAsideRecord({ runsFolder: '/runs', record });
 ```
 */
export async function writeSetAsideRecord({
  runsFolder,
  record,
}: {
  readonly runsFolder: string;
  readonly record: SetAsideRecord;
},): Promise<string> {
  /**
   The record's path.
   */
  const file = setAsideFileOf({
    runsFolder,
    name: record.name,
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
    return file;
  }
  catch (error) {
    throw new SetAsideRecordError(`the set-aside record of ${record.name} cannot be written at ${file}: ${String(error,)}`,);
  }
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
     The record of the file just listed.
     */
    // oxlint-disable-next-line eslint/no-await-in-loop -- one record after another, each with its own error naming its own file
    const record = await setAsideRecordOf({
      runsFolder,
      name: file.slice(
        0,
        -SET_ASIDE_FILE_SUFFIX.length,
      ),
    },);
    if (record !== NO_SET_ASIDE_RECORD) {
      records.push(record,);
    }
  }
  return records;
}

/**
 Deletes one entry's set-aside record.

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
 Sets one entry aside: writes its record, then takes it out of the queue.

 @param queueFile - the queue's path

 @param runsFolder - folder the driver writes its runs into

 @param record - the entry, as the queue held it

 @param reason - why its start failed

 @param keptOutputAt - where the whole of what the failed start printed was kept, when it was kept

 @param now - the reading the record's time is written from

 @returns the record's path, and how many entries the queue holds afterwards

 @throws SetAsideRecordError when the record cannot be written, which leaves the entry queued

 @example
 ```ts
 const { file, left } = await setAsideAnEntryOfAQueue({ queueFile: '/runs/q.json', runsFolder: '/runs', record, reason: 'x', now: Date.now });
 ```
 */
export async function setAsideAnEntryOfAQueue({
  queueFile,
  runsFolder,
  record,
  reason,
  keptOutputAt,
  now,
}: {
  readonly queueFile: string;
  readonly runsFolder: string;
  readonly record: QueueRecord;
  readonly reason: string;
  readonly keptOutputAt?: string;
  readonly now: () => number;
},): Promise<{
  readonly file: string;
  readonly left: number;
}> {
  /**
   The record's path, written before the entry leaves the queue.
   */
  const file = await writeSetAsideRecord({
    runsFolder,
    record: {
      name: record.name,
      entry: record,
      reason,
      setAsideAt: new Date(now(),).toISOString(),
      ...((keptOutputAt === undefined) ? {} : { keptOutputAt, }),
    },
  },);
  /**
   The queue without the entry.
   */
  const { entries } = await takeAQueueEntryOut({
    queueFile,
    name: record.name,
  },);
  return {
    file,
    left: entries.length,
  };
}

/**
 Puts a set-aside entry back into the queue, at a position counted from the head.

 The entry goes into the queue before its record goes away. A kill between the two leaves both, which the filling reads as an
 unfinished set-aside and starts the entry, so nothing is lost either way. An entry the queue already holds is only its record
 going away, which is what an interrupted set-aside left.

 @param queueFile - the queue's path

 @param runsFolder - folder the driver writes its runs into

 @param name - the entry's name

 @param position - where it goes, counted from the head, `0` being the head

 @returns the line to print

 @throws SetAsideRecordError when the record cannot be read or deleted, or the queue cannot be written

 @example
 ```ts
 const line = await putASetAsideEntryBack({ queueFile: '/runs/q.json', runsFolder: '/runs', name: 'a', position: 0 });
 ```
 */
export async function putASetAsideEntryBack({
  queueFile,
  runsFolder,
  name,
  position,
}: {
  readonly queueFile: string;
  readonly runsFolder: string;
  readonly name: string;
  readonly position: number;
},): Promise<{ readonly line: string }> {
  /**
   The record the entry is put back from.
   */
  const record = await setAsideRecordOf({
    runsFolder,
    name,
  },);
  if (record === NO_SET_ASIDE_RECORD) {
    return { line: `no entry named ${name} is set aside beside the runs in ${runsFolder}; nothing changed`, };
  }
  /**
   Whether the queue holds the entry already, which an interrupted set-aside leaves.
   */
  const queued = (await readQueueOfRuns({ queueFile, },)).some(function isNamed(entry,) {
    return entry.name === name;
  },);
  /**
   Where it landed, when this put it in.
   */
  const at = queued
    ? undefined
    : await putAQueueEntryIn({
      queueFile,
      entry: record.entry,
      position,
    },);
  await removeSetAsideRecord({
    runsFolder,
    name,
  },);
  return {
    line: (at === undefined)
      ? `${name} was still in the queue, so only its set-aside record was removed: ${record.reason}`
      : `put ${name} back at position ${String(at.position,)} of ${String(at.count,)}; it was set aside at ${record.setAsideAt} for: ${record.reason}`,
  };
}
