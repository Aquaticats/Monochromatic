/**
 The folder a queue of runs is: one file per entry, named after the entry, and one file per claim on an entry, in a folder beside the
 entries. Every change is one atomic step on one entry, so two commands change two entries at once and neither loses the other's work.

 The steps this folder is built on:

 - an entry is written by a file written beside the folder and linked into the entry's name, which refuses the write when the name is
   taken: that link is what makes two entries of one name impossible without a lock;
 - an entry is replaced by a file written beside it and renamed over it, so a reader meets one entry or the other and never half of either;
 - a claim is a file linked into a name of its own, holding the attempt's token, the filling's process number and when it was taken, so
   exactly one filling takes it and what it holds is one small file to read;
 - an entry is taken out by a rename first and a read of what the rename moved, so the entry read is the one that was there and never one a
   person queued under that name a moment later.

 Nothing here holds a lock and nothing here waits: a step that cannot be taken answers what it found, and the caller asks about the next
 entry. Every file written on the way to a name begins with a dot, and every reader of a queue folder skips a name that begins with one, so
 a command killed in the middle of a step leaves a name no reader ever takes for an entry.

 @module
 */

import {
  link,
  mkdir,
  readdir,
  readFile,
  rename,
  rm,
  writeFile,
} from 'node:fs/promises';
import {
  basename,
  dirname,
  join,
} from 'node:path';
import { randomUUID, } from 'node:crypto';
import { errorCodeOf, } from './error-code-of.ts';
import {
  type AClaimOnAnEntry,
  BEING_TAKEN_START,
  BEING_WRITTEN_START,
  CLAIMED_FOLDER,
  claimedFolderOf,
  anEntryFileName,
  entryFileOf,
  isQueueRecord,
  NO_QUEUE_ID,
  ORDER_FIELD,
  QUEUE_ID_FIELD,
  type QueueClaim,
  type QueueRecord,
  QueueFileError,
  aFreshQueueId,
  queueIdOf,
  type ANameInAFolder,
} from './queue-of-runs.ts';
import { ordersOfAPlace, } from './order-of-an-entry-in-a-queue.ts';

/**
 The code the operating system gives a path that is not there.
 */
const MISSING_PATH_CODE = 'ENOENT';

/**
 The code the operating system gives a path that is there already, which is a name a queue already holds.
 */
const EXISTS_CODE = 'EEXIST';

/**
 The ending every entry's file and every claim's file carries.
 */
const ENTRY_FILE_ENDING = '.json';

/**
 Reads the queue folder's own entry files, and refuses anything in it that is neither an entry nor a name a command writes.

 @param queueFolder - the queue's folder

 @returns the entry files' names

 @throws QueueFileError when the folder cannot be read, or holds a name that is neither an entry, the claims folder, nor a dot-file

 @example
 ```ts
 const files = await entryFilesIn({ queueFolder: '/runs/q' });
 ```
 */
async function entryFilesIn({ queueFolder, }: { readonly queueFolder: string },): Promise<readonly string[]> {
  /**
   What the folder holds.
   */
  let held: readonly ANameInAFolder[] = [];
  try {
    held = await readdir(
      queueFolder,
      { withFileTypes: true, },
    );
  }
  catch (error) {
    throw new QueueFileError(
      `the queue at ${queueFolder} cannot be read: ${String(error,)}. A queue is a folder of one file per entry, which the first put makes.`,
    );
  }
  /**
   The entry files, and the names this module passes over.
   */
  const files: string[] = [];
  for (const name of held) {
    if (name.isFile()) {
      if (name.name
        .endsWith(ENTRY_FILE_ENDING,)
        && (!name.name
          .startsWith('.',))) {
        files.push(name.name,);
        continue;
      }
      if (name.name
        .startsWith('.',)) {
        continue;
      }
    }
    // The claims fold is the queue's own, and a dot-file is what a command writes on its way to a name. Any other name is not this
    // module's to read and not a caller's to lose: it is refused by name rather than taken for an empty queue.
    if ((name.name === CLAIMED_FOLDER)
      || name.name
        .startsWith('.',)) {
      continue;
    }
    throw new QueueFileError(
      `${join(
        queueFolder,
        name.name,
      )} is neither an entry of the queue nor the folder its claims live in, so the queue at ${queueFolder} cannot be read: move it aside, or name the folder that holds only entries`,
    );
  }
  return files;
}

/**
 No entry of that name, as a symbol so no entry can be mistaken for none: the queue holds no entry under that name now.
 */
export const NO_ENTRY_OF_THAT_NAME: unique symbol = Symbol('the queue holds no entry under that name',);

/**
 Reads one entry's file.

 @param path - the file's path, for the reading and the message

 @param missingIsNone - whether a file that is not there is the answer "no entry of that name" rather than a failure to read

 @returns the entry, or that no file is there when that is what was asked

 @throws QueueFileError when the file cannot be read, holds no JSON, or holds no entry, and a missing file was to be refused

 @example
 ```ts
 const entry = await anEntryAt({ path: '/runs/q/a.json', missingIsNone: false });
 ```
 */
export async function anEntryAt({
  path,
  missingIsNone,
}: {
  readonly path: string;
  readonly missingIsNone: boolean;
},): Promise<QueueRecord | typeof NO_ENTRY_OF_THAT_NAME> {
  /**
   What the file holds.
   */
  let shown = '';
  try {
    shown = await readFile(
      path,
      'utf8',
    );
  }
  catch (error) {
    // A file that is not there is a name this queue does not hold, which a take by a person and a completion both leave. A caller that
    // asked for a name to be there reads the refusal instead, because "could not tell" is never "none".
    if (missingIsNone && (errorCodeOf(error,) === MISSING_PATH_CODE)) {
      return NO_ENTRY_OF_THAT_NAME;
    }
    throw new QueueFileError(`the entry at ${path} cannot be read: ${String(error,)}`,);
  }
  /**
   What the text parsed to.
   */
  let parsed: unknown = undefined;
  try {
    parsed = JSON.parse(shown,);
  }
  catch (error) {
    throw new QueueFileError(`the entry at ${path} holds no JSON: ${String(error,)}`,);
  }
  if (!isQueueRecord(parsed,)) {
    throw new QueueFileError(`${path} holds no JSON object naming an entry, so it cannot be started`,);
  }
  return parsed;
}

/**
 Reads one entry's file at a path of its own, which is the entry a take renamed aside before it read it.

 @param path - the file's path

 @returns the entry

 @throws QueueFileError when the file cannot be read, holds no JSON, or holds no entry, and the file is left where it is

 @example
 ```ts
 const entry = await readTheEntryAt({ path: '/runs/q/.being-taken-1-2' });
 ```
 */
export async function readTheEntryAt({ path, }: { readonly path: string },): Promise<QueueRecord> {
  /**
   The entry the file holds, or that no file is there.
   */
  const entry = await anEntryAt({
    path,
    missingIsNone: false,
  },);
  if (entry === NO_ENTRY_OF_THAT_NAME) {
    // A file this caller renamed aside and then could not find is not a file it can read; the path names where the entry was left.
    throw new QueueFileError(`the entry at ${path} is not there`,);
  }
  return entry;
}

/**
 Reads the entry one name is queued under, which is the entry a filling claims, completes or gives back.

 @param queueFolder - the queue's folder

 @param name - the entry's name

 @returns the entry, or that the queue holds no entry of that name now

 @throws QueueFileError when the entry's file is there and cannot be read, or holds no entry

 @example
 ```ts
 const entry = await readTheEntryNamed({ queueFolder: '/runs/q', name: 'a' });
 ```
 */
export async function readTheEntryNamed({
  queueFolder,
  name,
}: {
  readonly queueFolder: string;
  readonly name: string;
},): Promise<QueueRecord | typeof NO_ENTRY_OF_THAT_NAME> {
  return await anEntryAt({
    path: entryFileOf({
      queueFolder,
      name,
    },),
    missingIsNone: true,
  },);
}

/**
 Reads a queue folder's entries in the queue's order.

 The order is the `order` key each entry carries, and two entries of one key are ordered by their names, which is a total order and the
 same one for every reader. An entry that carries no key cannot be placed, and is refused by name: a queue whose order is unknown is not a
 queue a filling may start.

 @param queueFolder - the queue's folder

 @returns the entries, in the order they are to start

 @throws QueueFileError when the folder or one of its entries cannot be read, or an entry carries no order key

 @example
 ```ts
 const entries = await readQueueOfRuns({ queueFolder: '/runs/q' });
 ```
 */
export async function readQueueOfRuns({ queueFolder, }: { readonly queueFolder: string },): Promise<readonly QueueRecord[]> {
  /**
   The entry files the folder holds.
   */
  const files = await entryFilesIn({ queueFolder, },);
  /**
   The entries read so far, each with the place it was written at.
   */
  const placed: {
    readonly order: string;
    readonly entry: QueueRecord
  }[] = [];
  for (const file of files) {
    /**
     The entry the file holds, which the read above refused to answer as none.
     */
    // oxlint-disable-next-line eslint/no-await-in-loop -- one entry after another, each with its own error naming its own file
    const read = await anEntryAt({
      path: join(
        queueFolder,
        file,
      ),
      // A file the folder listed and this read cannot find is one another command took out in between, which is a definite state and
      // not a failure to read: a whole-queue read must not refuse because a filling completed an entry while it ran.
      missingIsNone: true,
    },);
    if (read === NO_ENTRY_OF_THAT_NAME) {
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
      throw new QueueFileError(
        `${join(
          queueFolder,
          file,
        )} names no order, so where it stands in the queue is unknown: an entry is written by this package's own put, which writes its order beside it`,
      );
    }
    placed.push({
      order,
      entry,
    },);
  }
  return placed.toSorted(function inTheQueuesOrder(
    one,
    other,
  ) {
    if (one.order !== other.order) {
      return one.order < other.order ? (-1) : 1;
    }
    // Two names are compared by their own characters, which is one total order: two names that differ anywhere at all differ here, while
    // a locale's order can count two different names as one (`'é'` and `'e'` with a combining mark, for instance), leaving them in the
    // order the folder listed them.
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
 Writes a file beside its own path and puts it into place, which is one step and refuses a path already taken unless it is told to
 replace it.

 @param path - where the file goes

 @param text - what it holds

 @param replace - whether a file already at that path is replaced rather than refusing the write

 @returns whether the file was written, or a file was there already

 @throws QueueFileError when the file cannot be written for any other reason

 @example
 ```ts
 const written = await aFileLaidDown({ path: '/runs/q/a.json', text: '{}', replace: false });
 ```
 */
export async function aFileLaidDown({
  path,
  text,
  replace,
}: {
  readonly path: string;
  readonly text: string;
  readonly replace: boolean;
},): Promise<boolean> {
  /**
   The file written beside the path and put into place.
   */
  const beside = join(
    dirname(path,),
    `${BEING_WRITTEN_START}${basename(path,)}-${String(process.pid,)}-${randomUUID()}`,
  );
  /**
   Whether the file was put into place, or a name was there already.
   */
  await writeFile(
    beside,
    text,
  );
  /**
   Whether the file was put into place, or a name was there already.
   */
  const laidDown = await intoItsPlace({
    beside,
    path,
    replace,
  },);
  // The file beside the path is never an entry: it is removed whatever the step answered, so a refused write leaves nothing behind.
  await rm(
    beside,
    { force: true, },
  );
  return laidDown.placed;
}

/**
 What putting one written file into its place answered.
 */
type LaidDown =
  | { readonly placed: true }
    | { readonly placed: false };

/**
 Puts one written file into its place, which is a rename that replaces the name or a link that refuses it.

 @param beside - the file made already, beside the place it goes

 @param path - the name it takes

 @param replace - whether a file already at that name is replaced

 @returns whether it was put into place, or the name was taken

 @throws QueueFileError when the step failed for any other reason

 @example
 ```ts
 const laidDown = await intoItsPlace({ beside: '/runs/q/.being-written-a.json-1-2', path: '/runs/q/a.json', replace: false });
 ```
 */
async function intoItsPlace({
  beside,
  path,
  replace,
}: {
  readonly beside: string;
  readonly path: string;
  readonly replace: boolean;
},): Promise<LaidDown> {
  try {
    if (replace) {
      await rename(
        beside,
        path,
      );
      return { placed: true, };
    }
    await link(
      beside,
      path,
    );
    return { placed: true, };
  }
  catch (error) {
    if (errorCodeOf(error,) === EXISTS_CODE) {
      return { placed: false, };
    }
    throw new QueueFileError(`the queue file at ${path} cannot be written: ${String(error,)}`,);
  }
}

/**
 Writes one entry into its queue folder: a file linked into the entry's name, or renamed over it.

 @param queueFolder - the queue's folder

 @param entry - the entry, with its order and its identity

 @param replace - whether the entry's name is already this entry's, which a move and a put-back replace rather than refuse

 @returns whether the entry was written, or another entry holds that name

 @throws QueueFileError when the folder or the file cannot be written

 @example
 ```ts
 const written = await writeAnEntryOfAQueue({ queueFolder: '/runs/q', entry, replace: false });
 ```
 */
export async function writeAnEntryOfAQueue({
  queueFolder,
  entry,
  replace,
}: {
  readonly queueFolder: string;
  readonly entry: QueueRecord;
  readonly replace: boolean;
},): Promise<boolean> {
  try {
    await mkdir(
      queueFolder,
      { recursive: true, },
    );
  }
  catch (error) {
    throw new QueueFileError(`the queue folder ${queueFolder} cannot be made: ${String(error,)}`,);
  }
  return await aFileLaidDown({
    path: entryFileOf({
      queueFolder,
      name: entry.name,
    },),
    text: `${JSON.stringify(entry,)}\n`,
    replace,
  },);
}

/**
 Reads the entry one write of a whole queue should hold: the entry as it is, with a place of the queue's own and an identity no other
 entry of that queue carries.

 @param entry - the entry as it was given

 @param order - the place the queue writes it at

 @param entries - every entry of the queue, as they were given

 @returns the entry as it is written

 @example
 ```ts
 const written = anEntryOfAWholeQueue({ entry, order: 'i', entries: [entry] });
 ```
 */
function anEntryOfAWholeQueue({
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
   Whether another entry of this queue already carries that identity.
   */
  const taken = (held !== NO_QUEUE_ID) && entries.some(function isAnotherEntryWithIt(other,) {
    return (other !== entry) && (queueIdOf({ record: other, },) === held);
  },);
  return {
    ...entry,
    [ORDER_FIELD]: order,
    [QUEUE_ID_FIELD]: ((held === NO_QUEUE_ID) || taken) ? aFreshQueueId() : held,
  };
}

/**
 Writes a whole queue: every entry given, one file each, in the order given, with a place and an identity where an entry carries none.

 A queue written this way has evenly spaced places, so a put between any two of them is one file and rewrites no entry. Nothing already in
 the folder is removed and a name already there is refused, so this makes a queue and never quietly replaces one.

 @param queueFolder - the queue's folder

 @param entries - the entries, in the order they are to start

 @throws QueueFileError when the folder cannot be written, or already holds an entry of one of those names

 @example
 ```ts
 await writeTheQueueOfRuns({ queueFolder: '/runs/q', entries: [] });
 ```
 */
export async function writeTheQueueOfRuns({
  queueFolder,
  entries,
}: {
  readonly queueFolder: string;
  readonly entries: readonly QueueRecord[];
},): Promise<void> {
  /**
   The places a queue of this length is written at.
   */
  const orders = ordersOfAPlace({ count: entries.length, },);
  for (const [place, entry,] of entries.entries()) {
    // oxlint-disable-next-line eslint/no-await-in-loop -- one file per entry, and a name already taken ends the write
    if (!(await writeAnEntryOfAQueue({
      queueFolder,
      entry: anEntryOfAWholeQueue({
        entry,
        order: orders[place] ?? '',
        entries,
      },),
      replace: false,
    },))) {
      throw new QueueFileError(`an entry named ${entry.name} is already in the queue at ${queueFolder}`,);
    }
  }
}
