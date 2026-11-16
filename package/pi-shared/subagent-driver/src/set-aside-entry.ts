/**
 Where an entry whose start failed is taken out of the way: one JSON file per entry beside the runs, holding the entry itself, the
 reason its start failed, and when, so a person and a session both find it where every other record of a run is.

 The record is written before the entry is taken out of the queue, both inside one held step of the queue's lock (`withTheQueueLock`),
 so no other command can see the record of an entry that is still queued unless a kill left it there. A kill between the two leaves
 the entry in both places, which loses nothing: the filling reads a record whose name the queue still holds as an unfinished
 set-aside, drops the record inside the same held step, and starts the entry again. The other order would leave a kill with the entry
 out of the queue and nowhere else.

 Two commands must therefore never touch a record outside that step: a put-back that finds the entry still queued leaves the record
 alone (removing it there, as it did before the second review, is how an entry in flight could be lost), and the filling's sweep
 makes its decision on the queue as it is inside the step, never on a snapshot.

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
  entryIsTheAttempt,
  isQueueRecord,
  type QueueClaim,
  type QueueRecord,
  withTheQueueLock,
  withoutTheClaim,
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

 The caller holds the queue's lock: a record is state of its entry, and writing one outside a held step is how a put-back could
 delete the record of a set-aside a filling was still committing, losing the entry. Every command of this package writes a record
 through `setAsideAnEntryOfAQueue`, which holds the step.

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

 The caller holds the queue's lock and decides on the queue inside that step: a removal made on a snapshot of the queue is how a
 second filling deleted the record of a set-aside a first was still committing.

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

 Both happen inside one held step of the queue's lock, the record first: a kill between the two leaves the entry claimed with its
 record, and the next pass's settlement finishes the set-aside by taking the entry out. The entry is taken out by its identity and
 the claim's token, so an entry queued under the same name meanwhile, while this start was in flight, is left where it is.

 @param queueFile - the queue's path

 @param runsFolder - folder the driver writes its runs into

 @param record - the entry, as the queue held it

 @param claim - the claim the filling holds on it, and the entry's identity

 @param reason - why its start failed

 @param keptOutputAt - where the whole of what the failed start printed was kept, when it was kept

 @param now - the reading the record's time is written from

 @param waitMilliseconds - how long to keep asking for the queue's lock before refusing

 @returns the record's path, and how many entries the queue holds afterwards

 @throws SetAsideRecordError when the record cannot be written, which leaves the entry queued

 @example
 ```ts
 const { file, left } = await setAsideAnEntryOfAQueue({ queueFile: '/runs/q.json', runsFolder: '/runs', record, claim, reason: 'x', now: Date.now });
 ```
 */
export async function setAsideAnEntryOfAQueue({
  queueFile,
  runsFolder,
  record,
  claim,
  reason,
  keptOutputAt,
  now,
  waitMilliseconds,
}: {
  readonly queueFile: string;
  readonly runsFolder: string;
  readonly record: QueueRecord;
  readonly claim: {
    readonly queueId: string;
    readonly claim: QueueClaim;
  };
  readonly reason: string;
  readonly keptOutputAt?: string;
  readonly now: () => number;
  readonly waitMilliseconds?: number;
},): Promise<{
  readonly file: string;
  readonly left: number;
}> {
  return await withTheQueueLock({
    queueFile,
    ...(waitMilliseconds === undefined ? {} : { waitMilliseconds, }),
    work: async function setItAside({
      entries,
      write,
    },) {
      if (((typeof claim) !== 'object') || (claim === null)
        || ((typeof claim.queueId) !== 'string')
        || (claim.queueId === '')) {
        // Only an entry this filling claimed can be set aside: the queue's copy goes only when it is that very attempt, so a caller
        // with no claim would write a record and leave the entry where it stands, which loses nothing but decides nothing.
        throw new SetAsideRecordError(
          `${record.name} is not set aside: only an entry a filling has claimed can be, because the record is that attempt's copy and `
            + 'the queue holds one entry of that name, which is taken out by the identity and the token of the claim that took it. Nothing changed.',
        );
      }
      // The record is written first, inside this step: a kill before the queue write leaves the entry claimed with its record, which
      // the next pass's settlement finishes, so the entry is set aside and nothing is lost.
      /**
       The entry as the record holds it: whole, with its identity, and without the claim, which belongs to the queue's copy.
       */
      const setAside = withoutTheClaim({
        record: {
          ...record,
          queueId: claim.queueId,
        },
      },);
      /**
       The record's path.
       */
      const file = await writeSetAsideRecord({
        runsFolder,
        record: {
          name: record.name,
          entry: setAside,
          reason,
          setAsideAt: new Date(now(),).toISOString(),
          ...((keptOutputAt === undefined) ? {} : { keptOutputAt, }),
        },
      },);
      /**
       The queue without this attempt, matched on the entry's identity and the claim's token.
       */
      const kept = entries.filter(function isNoLongerTheAttempt(entry,) {
        return !entryIsTheAttempt({
          record: entry,
          queueId: claim.queueId,
          token: claim.claim
            .token,
        },);
      },);
      await write(kept,);
      return {
        file,
        left: kept.length,
      };
    },
  },);
}
