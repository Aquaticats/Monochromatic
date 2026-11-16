/**
 The three moves a coordinator makes on a queue folder: one entry put in at a position, one taken out by name, and one moved to a position.
 Each is made of one atomic step on one entry's file, and nothing here waits on another writer.

 A put writes one file and rewrites no entry, because an entry's place in the queue is a key inside it (`order-of-an-entry-in-a-queue.ts`)
 and the key between any two keys is found digit by digit. A take renames the entry aside before it reads it, so the entry read is the one
 that was there and never one a person queued under that name while the read ran. A move rewrites the one entry whose place changes.

 @module
 */

import {
  rename,
  rm,
} from 'node:fs/promises';
import { errorCodeOf, } from './error-code-of.ts';
import {
  QUEUE_ID_FIELD,
  ORDER_FIELD,
  NO_QUEUE_ID,
  type QueueRecord,
  QueueFileError,
  aFreshQueueId,
  entryFileOf,
  queueIdOf,
} from './queue-of-runs.ts';
import {
  aNameTakenAside,
  aPlaceFor,
  readQueueOfRuns,
  readTheEntryAt,
  writeAnEntryOfAQueue,
} from './reading-and-writing-the-folder-a-queue-is.ts';

/**
 The code the operating system gives a path that is not there.
 */
const MISSING_PATH_CODE = 'ENOENT';

/**
 Reads the entry one put writes: the entry as it was given, with the place it is queued at and an identity no other entry of the queue
 carries.

 @param entry - the entry as the caller wrote it

 @param order - the place the queue writes it at

 @param entries - the entries the queue holds

 @returns the entry as it is written

 @example
 ```ts
 const written = anEntryQueuedAt({ entry, order: 'i', entries: [] });
 ```
 */
function anEntryQueuedAt({
  entry,
  order,
  entries,
}: {
  readonly entry: QueueRecord;
  readonly order: string;
  readonly entries: readonly QueueRecord[];
},): QueueRecord {
  /**
   The identity the entry already carries, when it carries one.
   */
  const held = queueIdOf({ record: entry, },);
  /**
   Whether another entry of this queue already carries that identity, which a copied entry does.
   */
  const taken = (held !== NO_QUEUE_ID) && entries.some(function isAnotherWithIt(other,) {
    return queueIdOf({ record: other, },) === held;
  },);
  return {
    ...entry,
    [ORDER_FIELD]: order,
    [QUEUE_ID_FIELD]: ((held === NO_QUEUE_ID) || taken) ? aFreshQueueId() : held,
  };
}

/**
 Puts one entry into the queue at a position counted from the head.

 An entry of that name is refused rather than queued twice: two runs of one name collide in every folder, unit and worktree the name gives
 them, so the second would replace the first rather than run beside it. The refusal is the link that writes the entry, so two puts of one
 name at once cannot both succeed.

 @param queueFolder - the queue's folder

 @param entry - the entry to queue

 @param position - where it goes, counted from the head, zero being the head

 @returns where it landed, and how many entries the queue now holds

 @throws QueueFileError when the queue cannot be read or written, or already holds an entry of that name

 @example
 ```ts
 const { position, count } = await putAQueueEntryIn({ queueFolder: '/runs/q', entry: { name: 'a' }, position: 0 });
 ```
 */
export async function putAQueueEntryIn({
  queueFolder,
  entry,
  position,
}: {
  readonly queueFolder: string;
  readonly entry: QueueRecord;
  readonly position: number;
},): Promise<{
  readonly position: number;
  readonly count: number;
}> {
  /**
   The queue as it stands.
   */
  const entries = await readQueueOfRuns({ queueFolder, },);
  /**
   Where the entry goes and the key it is written at.
   */
  const place = aPlaceFor({
    entries,
    position,
  },);
  /**
   Whether the entry was written, or the queue already held that name.
   */
  const written = await writeAnEntryOfAQueue({
    queueFolder,
    entry: anEntryQueuedAt({
      entry,
      order: place.order,
      entries,
    },),
    replace: false,
  },);
  if (!written) {
    throw new QueueFileError(`an entry named ${entry.name} is already in the queue at ${queueFolder}`,);
  }
  return {
    position: place.at,
    count: entries.length + 1,
  };
}

/**
 Takes the entry of one name out of the queue, and answers it.

 The entry's file is renamed aside first and read from there, so the entry answered is the one the rename moved, and an entry a person
 queued under that name a moment later is left where it is.

 @param queueFolder - the queue's folder

 @param name - the entry's name

 @returns the entries taken out, and the entries the queue holds afterwards

 @throws QueueFileError when the queue cannot be read, or the entry taken aside holds no entry, which leaves it there for a person

 @example
 ```ts
 const { taken, entries } = await takeAQueueEntryOut({ queueFolder: '/runs/q', name: 'a' });
 ```
 */
export async function takeAQueueEntryOut({
  queueFolder,
  name,
}: {
  readonly queueFolder: string;
  readonly name: string;
},): Promise<{
  readonly taken: readonly QueueRecord[];
  readonly entries: readonly QueueRecord[];
}> {
  /**
   The entry's file.
   */
  const file = entryFileOf({
    queueFolder,
    name,
  },);
  /**
   Where the entry is renamed to while it is read.
   */
  const aside = aNameTakenAside({ path: file, },);
  try {
    await rename(
      file,
      aside,
    );
  }
  catch (error) {
    // A name the queue does not hold is nothing to take, which is what a filling that started the entry leaves.
    if (errorCodeOf(error,) === MISSING_PATH_CODE) {
      return {
        taken: [],
        entries: await readQueueOfRuns({ queueFolder, },),
      };
    }
    throw new QueueFileError(`the entry at ${file} cannot be taken out of the queue: ${String(error,)}`,);
  }
  /**
   The entry the rename moved, which is the one that was there.
   */
  const entry = await readTheEntryAt({ path: aside, },);
  await rm(
    aside,
    { force: true, },
  );
  return {
    taken: [entry,],
    entries: await readQueueOfRuns({ queueFolder, },),
  };
}

/**
 Moves the entry of one name to a position counted from the head.

 @param queueFolder - the queue's folder

 @param name - the entry's name

 @param position - where it goes, counted from the head, zero being the head

 @returns where it was and where it landed, or that the queue holds no entry of that name

 @throws QueueFileError when the queue cannot be read or written

 @example
 ```ts
 const moved = await moveAQueueEntryTo({ queueFolder: '/runs/q', name: 'a', position: 0 });
 ```
 */
export async function moveAQueueEntryTo({
  queueFolder,
  name,
  position,
}: {
  readonly queueFolder: string;
  readonly name: string;
  readonly position: number;
},): Promise<{
  readonly from?: number;
  readonly to?: number;
  readonly count: number;
}> {
  /**
   The queue as it stands.
   */
  const entries = await readQueueOfRuns({ queueFolder, },);
  /**
   Where the entry stands, or minus one when the queue holds none of that name.
   */
  const from = entries.findIndex(function isNamed(entry,) {
    return entry.name === name;
  },);
  if (from === (-1)) {
    return { count: entries.length, };
  }
  /**
   The entry itself.
   */
  const entry = entries[from] as QueueRecord;
  /**
   The entries without it, which is the queue its new place is counted in.
   */
  const rest = entries.filter(function isAnother(
    _,
    place,
  ) {
    return place !== from;
  },);
  /**
   Where it lands and the key it is written at.
   */
  const place = aPlaceFor({
    entries: rest,
    position,
  },);
  await writeAnEntryOfAQueue({
    queueFolder,
    entry: {
      ...entry,
      [ORDER_FIELD]: place.order,
    },
    replace: true,
  },);
  return {
    from,
    to: place.at,
    count: rest.length + 1,
  };
}
