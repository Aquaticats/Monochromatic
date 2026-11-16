/**
 Changing a machine over from the queue this package kept as one JSON array to the queue folder it keeps now, without losing an entry at any
 instruction it can be killed at.

 The array is read first and every entry of it is placed in the folder: one file per entry, each written by the folder's own step, with the
 place and the identity the folder writes. Only when every entry is in the folder is the array renamed aside, so a command killed at any
 instruction leaves either the array still there, which is a queue this command reads again, or the array aside, which says the changeover
 is done.

 Running it twice is safe: an entry the folder already holds with the same fields is passed over, and one it holds with other fields is
 refused by name, because a changeover never overwrites a queue somebody has been editing since.

 @module
 */

import {
  readFile,
  rename,
} from 'node:fs/promises';
import { shapeOfAQueueEntry, } from './entry-of-a-queue.ts';
import {
  aFreshQueueId,
  isQueueRecord,
  ORDER_FIELD,
  QUEUE_ID_FIELD,
  type QueueRecord,
  QueueFileError,
} from './queue-of-runs.ts';
import {
  NO_ENTRY_OF_THAT_NAME,
  readTheEntryNamed,
  writeAnEntryOfAQueue,
} from './reading-and-writing-the-folder-a-queue-is.ts';
import { ordersOfAPlace, } from './order-of-an-entry-in-a-queue.ts';

/**
 The ending the array file is renamed to when the changeover is done, so what became the folder is still there to read.
 */
export const CONVERTED_QUEUE_ENDING = '.converted';

/**
 What one changeover did.
 */
export type ConvertedAQueue = {
  /**
   How many entries the folder holds from the array.
   */
  readonly entries: number;

  /**
   How many entries the folder already held with the same fields, which the changeover passed over.
   */
  readonly alreadyThere: number;

  /**
   Where the array was renamed to.
   */
  readonly arrayKeptAt: string;
};

/**
 Reads one entry without the fields the queue folder's own bookkeeping writes, which is what says whether two entries are the same entry.

 @param entry - the entry

 @returns the entry's own fields, without its place and its identity

 @example
 ```ts
 const own = theFieldsOfAnEntry({ entry });
 ```
 */
function theFieldsOfAnEntry({ entry, }: { readonly entry: QueueRecord },): string {
  /**
   The entry's fields, less the two the folder writes.
   */
  const {
    [ORDER_FIELD]: _place,
    [QUEUE_ID_FIELD]: _identity,
    ...own
  } = entry;
  return JSON.stringify(own,);
}

/**
 Reads the entries one queue array file holds, refusing an entry this package does not know rather than carrying it into the folder.

 @param queueFile - the array file's path

 @returns the entries, in the order the array holds them

 @throws QueueFileError when the file cannot be read, holds no JSON array, holds no entry, or holds an entry no shape of this package reads

 @example
 ```ts
 const entries = await entriesOfAQueueFile({ queueFile: '/runs/queue.json' });
 ```
 */
export async function entriesOfAQueueFile({ queueFile, }: { readonly queueFile: string },): Promise<readonly QueueRecord[]> {
  /**
   What the file holds.
   */
  let shown: string;
  try {
    shown = await readFile(
      queueFile,
      'utf8',
    );
  }
  catch (error) {
    throw new QueueFileError(`the queue at ${queueFile} cannot be read: ${String(error,)}`,);
  }
  /**
   What the text parsed to.
   */
  let parsed: unknown;
  try {
    parsed = JSON.parse(shown,);
  }
  catch (error) {
    throw new QueueFileError(`the queue at ${queueFile} holds no JSON: ${String(error,)}`,);
  }
  if (!Array.isArray(parsed,)) {
    throw new QueueFileError(`${queueFile} holds no JSON array of entries, so nothing can be carried into a queue folder`,);
  }
  return parsed.map(function asAnEntry(
    entry: unknown,
    index: number,
  ) {
    if (!isQueueRecord(entry,)) {
      throw new QueueFileError(`${queueFile}: entry ${String(index + 1,)} is no JSON object naming a run, so it is not carried into the folder`,);
    }
    /**
     What this package reads the entry as.
     */
    const shape = shapeOfAQueueEntry({ record: entry, },);
    if (shape.kind === 'unreadable') {
      throw new QueueFileError(
        `${queueFile}: extra is no entry this package reads, so it is not carried into the folder: ${entry.name} holds ${shape.reason}`,
      );
    }
    return entry;
  },);
}

/**
 Carries one queue array file into a queue folder, and renames the array aside when every entry of it is in the folder.

 @param queueFile - the array file's path

 @param queueFolder - the folder to carry it into

 @returns what the changeover did

 @throws QueueFileError when the array cannot be read, an entry of it is one this package does not read, the folder cannot be written, or
 the folder holds an entry of that name with other fields

 @example
 ```ts
 const done = await convertAQueueFileIntoAFolder({ queueFile: '/runs/queue.json', queueFolder: '/runs/queue' });
 ```
 */
export async function convertAQueueFileIntoAFolder({
  queueFile,
  queueFolder,
}: {
  readonly queueFile: string;
  readonly queueFolder: string;
},): Promise<ConvertedAQueue> {
  /**
   The entries the array holds, in its order.
   */
  const entries = await entriesOfAQueueFile({ queueFile, },);
  /**
   The places a queue of this length is written at, which is the array's own order.
   */
  const places = ordersOfAPlace({ count: entries.length, },);
  /**
   How many entries the folder already held with the same fields.
   */
  let alreadyThere = 0;
  /* oxlint-disable eslint/no-await-in-loop -- one entry at a time, and the folder is read for each of them */
  for (const [place, entry,] of entries.entries()) {
    /**
     The entry the folder holds under that name, or that it holds none.
     */
    const held = await readTheEntryNamed({
      queueFolder,
      name: entry.name,
    },);
    if (held !== NO_ENTRY_OF_THAT_NAME) {
      if (theFieldsOfAnEntry({ entry: held, },) === theFieldsOfAnEntry({ entry, },)) {
        alreadyThere += 1;
        continue;
      }
      throw new QueueFileError(
        `the queue folder ${queueFolder} already holds an entry named ${entry.name} with other fields than the array holds, so nothing `
          + 'more was carried over: a changeover never writes over an entry somebody has been editing since',
      );
    }
    /**
     Whether the entry was written, or a name was queued under it while the changeover ran.
     */
    const written = await writeAnEntryOfAQueue({
      queueFolder,
      entry: {
        ...entry,
        [ORDER_FIELD]: places[place] ?? '',
        [QUEUE_ID_FIELD]: aFreshQueueId(),
      },
      replace: false,
    },);
    if (!written) {
      throw new QueueFileError(
        `an entry named ${entry.name} was queued in ${queueFolder} while the changeover ran, so nothing more was carried over: `
          + 'the array is still where it was, and running this again passes over what is already there',
      );
    }
  }
  /* oxlint-enable eslint/no-await-in-loop */
  // Every entry of the array is in the folder, so the array is renamed aside: a kill before this leaves the array, which this command
  // reads again and passes over, and a kill after it leaves the folder.
  /**
   Where the array is renamed to.
   */
  const arrayKeptAt = `${queueFile}${CONVERTED_QUEUE_ENDING}`;
  await rename(
    queueFile,
    arrayKeptAt,
  );
  return {
    entries: entries.length - alreadyThere,
    alreadyThere,
    arrayKeptAt,
  };
}
