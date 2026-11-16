/**
 Reads what a queue folder holds, refusing a folder that cannot be read at all.

 @param queueFolder - the queue's folder

 @returns the names inside it

 @throws QueueFileError when the folder cannot be read

 @example
 ```ts
 const held = await whatTheQueueFolderHolds({ queueFolder: '/runs/q' });
 ```
 */
/**
 The two readers a filling's pass uses where the plain reader would stop at one file: one that reports every name it could not take and
 goes on, and one that answers an empty queue for a folder the first put has not made yet.

 The plain reader in `reading-and-writing-the-folder-a-queue-is.ts` refuses anything it cannot read, which is what a command that changes the
 queue needs. A pass that starts entries needs the other answer: one file a person left, or one file a crash left half written, must not stop
 every entry behind it, while a folder that cannot be read at all is still refused.

 @module
 */

import {
  mkdir,
  readdir,
} from 'node:fs/promises';
import { join, } from 'node:path';
import {
  type ANameInAFolder,
  CLAIMED_FOLDER,
  ORDER_FIELD,
  QueueFileError,
  type QueueRecord,
} from './queue-of-runs.ts';
import {
  NO_ENTRY_OF_THAT_NAME,
  anEntryAt,
  readQueueOfRuns,
} from './reading-and-writing-the-folder-a-queue-is.ts';

/**
 The ending every entry's file carries.
 */
const ENTRY_FILE_ENDING = '.json';

/**
 Reads what a queue folder holds, refusing a folder that cannot be read at all.

 @param queueFolder - the queue's folder

 @returns the names inside it

 @throws QueueFileError when the folder cannot be read

 @example
 ```ts
 const held = await whatTheQueueFolderHolds({ queueFolder: '/runs/q' });
 ```
 */
async function whatTheQueueFolderHolds({ queueFolder, }: { readonly queueFolder: string },): Promise<readonly ANameInAFolder[]> {
  try {
    return await readdir(
      queueFolder,
      { withFileTypes: true, },
    );
  }
  catch (error) {
    throw new QueueFileError(`the queue at ${queueFolder} cannot be read: ${String(error,)}`,);
  }
}



/**
 Reads a queue folder's entries in the queue's order, and reports every file it could not take rather than stopping at it.

 This is the reader a filling uses. A queue is a folder a person and a crash both write into, so one file that is empty, holds no entry, or
 names no place must not stop every entry behind it: each is reported by name, with what it is and what to do about it, and left where it is
 for a person. A folder that cannot be read at all is still refused, because “could not tell” is never “none”.

 @param queueFolder - the queue's folder

 @param report - told about each name this reader passed over, one line each

 @returns the entries it could read, in the order they are to start

 @throws QueueFileError when the folder itself cannot be read

 @example
 ```ts
 const entries = await readTheQueueReportingProblems({ queueFolder: '/runs/q', report: (line) => console.error(line) });
 ```
 */
export async function readTheQueueReportingProblems({
  queueFolder,
  report,
}: {
  readonly queueFolder: string;
  readonly report: (line: string) => void;
},): Promise<readonly QueueRecord[]> {
  /**
   What the folder holds, which a folder that cannot be read at all refuses.
   */
  const held = await whatTheQueueFolderHolds({ queueFolder, },);
  /**
   The entries read so far, each with the place it was written at.
   */
  const placed: {
    readonly order: string;
    readonly entry: QueueRecord
  }[] = [];
  /* oxlint-disable eslint/no-await-in-loop -- one file after another, each with its own line */
  for (const name of held) {
    /**
     The file's path, for the line.
     */
    const path = join(
      queueFolder,
      name.name,
    );
    if ((name.name === CLAIMED_FOLDER)
      || name.name
        .startsWith('.',)) {
      // The claims folder is the queue's own, and a dot-file is what a command writes on its way to a name.
      continue;
    }
    if ((!name.isFile()) || (!name.name
      .endsWith(ENTRY_FILE_ENDING,))) {
      report(`${path} is neither an entry of the queue nor the folder its claims live in, so it is left where it is; move it aside and the pass uses the rest of the queue\n`,);
      continue;
    }
    /**
     The entry the file holds, or nothing when it was taken out between the listing and this read.
     */
    const read = await readOneFileOfTheQueue({
      path,
      report,
    },);
    if (read === NO_ENTRY_OF_THAT_NAME) {
      continue;
    }
    if (read === NO_READABLE_ENTRY) {
      continue;
    }
    /**
     The entry the file holds.
     */
    const entry = read;
    /**
     The entry's own place in the queue.
     */
    const order: unknown = entry[ORDER_FIELD];
    if ((typeof order) !== 'string') {
      report(`${path} names no place in the queue and is left where it is, so a person can see it; the pass uses the rest of the queue: take it out, or give it an order, which an entry of this package's own put always has\n`,);
      continue;
    }
    placed.push({
      order,
      entry,
    },);
  }
  /* oxlint-enable eslint/no-await-in-loop */
  return placed.toSorted(function inTheQueuesOrder(
    one,
    other,
  ) {
    if (one.order !== other.order) {
      return one.order < other.order ? (-1) : 1;
    }
    return one.entry
      .name
      < other.entry
        .name ? (-1) : 1;
  },)
    .map(function theEntryOf(one,) {
      return one.entry;
    },);
}

/**
 This file holds no entry a pass may start, as a symbol so no entry can be mistaken for none.

 */
export const NO_READABLE_ENTRY: unique symbol = Symbol('this file holds an entry a pass cannot start',);

/**
 Reads one file of a queue, reporting what kept it from being read rather than stopping at it.

 @param path - the file's path

 @param report - where the line about a file that could not be read goes

 @returns the entry, that no entry of that name is there now, or that the file cannot be read

 @example
 ```ts
 const read = await readOneFileOfTheQueue({ path: '/runs/q/a.json', report: () => {} });
 ```
 */
async function readOneFileOfTheQueue({
  path,
  report,
}: {
  readonly path: string;
  readonly report: (line: string) => void;
},): Promise<QueueRecord | typeof NO_ENTRY_OF_THAT_NAME | typeof NO_READABLE_ENTRY> {
  try {
    return await anEntryAt({
      path,
      missingIsNone: true,
    },);
  }
  catch (error) {
    report(`${path} cannot be read as an entry and is left where it is, so a person can see it; the pass uses the rest of the queue: ${String(error,)}\n`,);
    return NO_READABLE_ENTRY;
  }
}

/**
 Reads a queue folder that may not be there yet, for the first put into a new queue.

 A folder that is not there is a queue that holds nothing, which is one definite state and not a failure to read: the put that asks makes
 the folder. A folder that is there and cannot be read is still refused.

 @param queueFolder - the queue's folder

 @returns the entries, in the order they are to start, and nothing for a folder that is not there

 @throws QueueFileError when the folder is there and cannot be read

 @example
 ```ts
 const entries = await readTheQueueThatMayNotBeThere({ queueFolder: '/runs/q' });
 ```
 */
export async function readTheQueueThatMayNotBeThere({
  queueFolder,
}: {
  readonly queueFolder: string;
},): Promise<readonly QueueRecord[]> {
  try {
    return await readQueueOfRuns({ queueFolder, },);
  }
  catch (error) {
    // Only a folder that is missing is a queue of nothing: a message naming anything else is carried out as the failure it is.
    if (String(error,)
      .includes('ENOENT',)) {
      await mkdir(
        queueFolder,
        { recursive: true, },
      );
      return [];
    }
    throw error;
  }
}
