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
  appendFile,
  readFile,
  rename,
  rm,
} from 'node:fs/promises';
import { errorCodeOf, } from './error-code-of.ts';
import { isThere, } from './newest-finished-run-of-a-name.ts';
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
 The code the operating system gives a path that is not there.
 */
const MISSING_PATH_CODE = 'ENOENT';

/**
 The ending the array file is renamed to when the changeover is done, so what became the folder is still there to read.
 */
export const CONVERTED_QUEUE_ENDING = '.converted';

/**
 The ending the array is held under while it is being carried, which a run that was killed leaves for the next run to finish.
 */
export const BEING_CARRIED_ENDING = '.converting';

/**
 The ending of the file that records the names a changeover has already written into the folder.
 */
export const CARRIED_ENDING = '.converting-carried';

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
   What the file holds, read as one value so no declaration is left unset.
   */
  const parsed = await whatOneQueueFileHolds({ queueFile, },);
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
        `${queueFile}: ${entry.name} is no entry this package reads, so it is not carried into the folder: ${shape.reason}`,
      );
    }
    return entry;
  },);
}

/**
 Reads what one queue array file holds, as anything.

 @param queueFile - the file's path

 @returns what its text parsed to

 @throws QueueFileError when the file cannot be read or holds no JSON

 @example
 ```ts
 const parsed = await whatOneQueueFileHolds({ queueFile: '/runs/queue.json' });
 ```
 */
async function whatOneQueueFileHolds({ queueFile, }: { readonly queueFile: string },): Promise<unknown> {
  try {
    return JSON.parse(await readFile(
      queueFile,
      'utf8',
    ),);
  }
  catch (error) {
    throw new QueueFileError(`the queue at ${queueFile} cannot be read as JSON: ${String(error,)}`,);
  }
}

/**
 Carries one queue array file into a queue folder, and answers what it carried.

 The array is renamed to `<file>.converting` **first**, and the folder is built from that file. A queue that another command adds to has
 therefore either been renamed already and its last entry carried, or not renamed and not carried at all: nothing written into the array
 between a read and a rename can be lost.

 Every entry is recorded by name in `<file>.converting-carried` as it is written, and a run after a kill passes over the names already
 recorded. That is what keeps a re-run from queueing again an entry a filling started while the changeover was unfinished.

The array is renamed to `<file>.converted-<stamp>` at the end, a name no earlier changeover holds, so a changeover never writes over the
copy an earlier one kept.

 @param queueFile - the array file's path

 @param queueFolder - the folder to carry it into

 @param now - the clock the stamp of the kept array is read from

 @returns what the changeover did

 @throws QueueFileError when the array cannot be read, an entry of it is one this package does not read, the folder cannot be written, or
 the folder holds an entry of that name with other fields

 @example
 ```ts
 const done = await convertAQueueFileIntoAFolder({ queueFile: '/runs/queue.json', queueFolder: '/runs/queue', now: Date.now });
 ```
 */
export async function convertAQueueFileIntoAFolder({
  queueFile,
  queueFolder,
  now = Date.now,
}: {
  readonly queueFile: string;
  readonly queueFolder: string;
  readonly now?: () => number;
},): Promise<ConvertedAQueue> {
  /**
   Where the array is while it is carried, which a run that was killed left behind.
   */
  const beingCarried = `${queueFile}${BEING_CARRIED_ENDING}`;
  /**
   Where the names already carried are recorded, one per line.
   */
  const carried = `${queueFile}${CARRIED_ENDING}`;
  if (!(await isThere(beingCarried,))) {
    // The rename comes before anything is read, so the array a changeover carries is the array as it stands at that one step.
    try {
      await rename(
        queueFile,
        beingCarried,
      );
    }
    catch (error) {
      if (errorCodeOf(error,) !== MISSING_PATH_CODE) {
        throw new QueueFileError(`the queue at ${queueFile} cannot be taken for the changeover: ${String(error,)}`,);
      }
      // Nothing to carry, and nothing of an earlier changeover left: there is no array to change over.
      if (!(await isThere(carried,))) {
        throw new QueueFileError(
          `the queue file ${queueFile} is not there, and neither is a changeover of it left unfinished, so there is nothing to carry into ${
             queueFolder}`,
        );
      }
    }
  }
  /**
   The entries the array holds, in its order.
   */
  const entries = await entriesOfAQueueFile({ queueFile: beingCarried, },);
  /**
   The names an earlier run of this changeover already wrote into the folder.
   */
  /**
   The names an earlier run of this changeover already wrote, read one line each.
   */
  const recorded = new Set<string>();
  if (await isThere(carried,)) {
    /**
     What the file recording the names holds.
     */
    const lines = await readFile(
      carried,
      'utf8',
    );
    for (const line of lines.split('\n',)) {
      if (line !== '') {
        recorded.add(line,);
      }
    }
  }
  /**
   The places a queue of this length is written at, which is the array's own order.
   */
  const places = ordersOfAPlace({ count: entries.length, },);
  /**
   The entries the folder already held with the same fields, one name each.
   */
  const alreadyThere: string[] = [];
  /* oxlint-disable eslint/no-await-in-loop -- one entry at a time, and the folder is read for each of them */
  for (const [place, entry,] of entries.entries()) {
    if (recorded.has(entry.name,)) {
      // An earlier run of this changeover wrote this entry, and a filling may have started it since: it is never queued a second time.
      alreadyThere.push(entry.name,);
      continue;
    }
    /**
     The entry the folder holds under that name, or that it holds none.
     */
    const held = await readTheEntryNamed({
      queueFolder,
      name: entry.name,
    },);
    if (held !== NO_ENTRY_OF_THAT_NAME) {
      if (theFieldsOfAnEntry({ entry: held, },) === theFieldsOfAnEntry({ entry, },)) {
        alreadyThere.push(entry.name,);
        await appendFile(
          carried,
          `${entry.name}\n`,
        );
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
          + 'the array is still held as being carried, and running this again passes over what is already there',
      );
    }
    // The name is recorded once the entry is in the folder, so a run after a kill at any step passes over exactly what it wrote.
    await appendFile(
      carried,
      `${entry.name}\n`,
    );
  }
  /* oxlint-enable eslint/no-await-in-loop */
  /**
   Where the array is kept once every entry of it is in the folder, which no earlier changeover holds.
   */
  const arrayKeptAt = `${queueFile}${CONVERTED_QUEUE_ENDING}-${new Date(now(),).toISOString()
    .replaceAll(
      ':',
      '',
    )}`;
  await rename(
    beingCarried,
    arrayKeptAt,
  );
  await rm(
    carried,
    { force: true, },
  );
  return {
    entries: entries.length - alreadyThere.length,
    alreadyThere: alreadyThere.length,
    arrayKeptAt,
  };
}
