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
} from './queue-of-runs.ts';
import {
  anOrderBetween,
  ordersOfAPlace,
} from './order-of-an-entry-in-a-queue.ts';

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
  let held;
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
async function anEntryAt({
  path,
  missingIsNone,
}: {
  readonly path: string;
  readonly missingIsNone: boolean;
},): Promise<QueueRecord | typeof NO_ENTRY_OF_THAT_NAME> {
  /**
   What the file holds.
   */
  let shown: string;
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
  let parsed: unknown;
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
      missingIsNone: false,
    },);
    if (read === NO_ENTRY_OF_THAT_NAME) {
      throw new QueueFileError(`${join(
        queueFolder,
        file,
      )} is not there, though the folder listed it`,);
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
    return one.entry
      .name
      .localeCompare(other.entry
        .name,);
  },)
    .map(function theEntryOf(one,) {
      return one.entry;
    },);
}

/**
 Reads what one claim's file holds as a claim, or refuses it by name.

 @param parsed - what the file's text parsed to

 @param path - the file's path, for the message

 @returns the claim

 @throws QueueFileError when it holds no name, identity, token, process number and time

 @example
 ```ts
 const claim = aClaimIn({ parsed, path: '/runs/q/claimed/a.claim.json' });
 ```
 */
export function aClaimIn({
  parsed,
  path,
}: {
  readonly parsed: unknown;
  readonly path: string;
},): AClaimOnAnEntry {
  if (((typeof parsed) !== 'object') || (parsed === null)) {
    throw new QueueFileError(`${path} holds no claim, so which filling started that entry is unknown`,);
  }
  /**
   The fields the claim should hold.
   */
  const name: unknown = Reflect.get(
    parsed,
    'name',
  );
  /**
   The identity of the entry the claim was taken on.
   */
  const queueId: unknown = Reflect.get(
    parsed,
    'queueId',
  );
  /**
   The attempt's own facts, as the file holds them.
   */
  const claim: unknown = Reflect.get(
    parsed,
    'claim',
  );
  if (
    ((typeof name) !== 'string')
      || (name === '')
      || ((typeof queueId) !== 'string')
      || (queueId === '')
      || ((typeof claim) !== 'object')
      || (claim === null)
  ) {
    throw new QueueFileError(`${path} holds no entry's name and identity with the claim on it, so which filling started that entry is unknown`,);
  }
  /**
   The token naming the attempt.
   */
  const token: unknown = Reflect.get(
    claim,
    'token',
  );
  /**
   The filling's process number.
   */
  const pid: unknown = Reflect.get(
    claim,
    'pid',
  );
  /**
   When the claim was taken.
   */
  const at: unknown = Reflect.get(
    claim,
    'at',
  );
  if (
    ((typeof token) !== 'string')
      || (token === '')
      || ((typeof pid) !== 'number')
      || ((typeof at) !== 'string')
  ) {
    throw new QueueFileError(`${path} names no token, process number and time for the attempt, so which filling started that entry is unknown`,);
  }
  return {
    name,
    queueId,
    claim: {
      token,
      pid,
      at,
    },
  };
}

/**
 Reads every claim a queue folder holds: the entries fillings are starting now, or were starting when their filling was killed.

 @param queueFolder - the queue's folder

 @returns one claim per entry a filling has taken, in the order the folder lists them

 @throws QueueFileError when the claims folder or one of its files cannot be read, or holds no claim

 @example
 ```ts
 const claims = await readTheClaimsOnAQueue({ queueFolder: '/runs/q' });
 ```
 */
export async function readTheClaimsOnAQueue({
  queueFolder,
}: {
  readonly queueFolder: string;
},): Promise<readonly AClaimOnAnEntry[]> {
  /**
   The folder the claims live in.
   */
  const claimedFolder = claimedFolderOf({ queueFolder, },);
  /**
   What that folder holds.
   */
  let held;
  try {
    held = await readdir(
      claimedFolder,
      { withFileTypes: true, },
    );
  }
  catch (error) {
    // A queue nothing has claimed an entry of holds no claims folder, which is a fact about the queue and not a failure to read it.
    if (errorCodeOf(error,) === MISSING_PATH_CODE) {
      return [];
    }
    throw new QueueFileError(`the claims beside the queue at ${queueFolder} cannot be read: ${String(error,)}`,);
  }
  /**
   The claims read so far.
   */
  const claims: AClaimOnAnEntry[] = [];
  for (const file of held) {
    if ((!file.isFile())
      || (!file.name
        .endsWith(ENTRY_FILE_ENDING,))
      || file.name
        .startsWith('.',)) {
      if ((!file.isFile()) && (!file.name
        .startsWith('.',))) {
        throw new QueueFileError(
          `${join(
            claimedFolder,
            file.name,
          )} is neither a claim nor a name a killed command left, so the claims beside the queue at ${queueFolder} cannot be read`,
        );
      }
      continue;
    }
    /**
     The claim's path, for the message.
     */
    const path = join(
      claimedFolder,
      file.name,
    );
    /**
     What the claim's file holds, as anything.
     */
    let parsed: unknown;
    try {
      parsed = JSON.parse(await readFile(
        path,
        'utf8',
      ),);
    }
    catch (error) {
      throw new QueueFileError(`the claim at ${path} cannot be read as one: ${String(error,)}`,);
    }
    claims.push(aClaimIn({
      parsed,
      path,
    },),);
  }
  return claims;
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
  try {
    await writeFile(
      beside,
      text,
    );
    if (replace) {
      await rename(
        beside,
        path,
      );
      return true;
    }
    await link(
      beside,
      path,
    );
    return true;
  }
  catch (error) {
    if (errorCodeOf(error,) === EXISTS_CODE) {
      return false;
    }
    throw new QueueFileError(`the queue file at ${path} cannot be written: ${String(error,)}`,);
  }
  finally {
    // The file beside the path is never an entry, and it is removed whatever the step answered, so a refused write leaves nothing.
    await rm(
      beside,
      { force: true, },
    );
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

/**
 Reads a key that stands between two entries of a queue, which is what a put at a position writes.

 @param lower - the key the new entry must come after, or nothing when it goes at the head

 @param upper - the key the new entry must come before, or nothing when it goes at the tail

 @returns the key

 @example
 ```ts
 const order = anOrderForAPlace({ lower: 'i', upper: 'j' });
 ```
 */
export function anOrderForAPlace({
  lower,
  upper,
}: {
  readonly lower?: string;
  readonly upper?: string;
},): string {
  return anOrderBetween({
    ...(lower === undefined ? {} : { lower, }),
    ...(upper === undefined ? {} : { upper, }),
  },);
}

/**
 Reads the place one entry goes to, counted from the head, which is the head of a queue shorter than the position and the tail of a
 position past its end.

 @param entries - the queue without the entry being placed

 @param position - where it goes, counted from the head, zero being the head

 @returns where it lands, and the key it is written at

 @throws QueueFileError when an entry of the queue carries no order key

 @example
 ```ts
 const place = aPlaceFor({ entries, position: 0 });
 ```
 */
export function aPlaceFor({
  entries,
  position,
}: {
  readonly entries: readonly QueueRecord[];
  readonly position: number;
},): {
  readonly at: number;
  readonly order: string
} {
  /**
   Where the entry lands.
   */
  const at = Math.min(
    position,
    entries.length,
  );
  /**
   The order key of one entry of the queue.

   @param entry - the entry

   @returns the key

   @throws QueueFileError when the entry carries none
   */
  function keyOf(entry: QueueRecord,): string {
    /**
     What the entry says its order is.
     */
    const order: unknown = entry[ORDER_FIELD];
    if ((typeof order) !== 'string') {
      throw new QueueFileError(`${entry.name} carries no order, so where it stands in the queue is unknown`,);
    }
    return order;
  }
  /**
   The key of the entry before it, when one stands there.
   */
  const lower = (at > 0) ? keyOf(entries[at - 1] as QueueRecord) : undefined;
  /**
   The key of the entry after it, when one stands there.
   */
  const upper = (at < entries.length) ? keyOf(entries[at] as QueueRecord) : undefined;
  return {
    at,
    order: anOrderForAPlace({
      ...(lower === undefined ? {} : { lower, }),
      ...(upper === undefined ? {} : { upper, }),
    },),
  };
}

/**
 Reads a name inside a queue folder that no entry and no claim carries, which a command renames or writes a file to while it works.

 @param path - the path the command is working on

 @returns a path beside it that is no entry's name and no claim's

 @example
 ```ts
 const aside = aNameTakenAside({ path: '/runs/q/a.json' });
 ```
 */
export function aNameTakenAside({ path, }: { readonly path: string },): string {
  // The name begins with a dot, so no reader of the folder takes the file a command is still working on for an entry.
  return join(
    dirname(path,),
    `${BEING_TAKEN_START}${basename(path,)}-${String(process.pid,)}-${randomUUID()}`,
  );
}

/**
 Reads the claim a filling is about to write: the facts of one attempt on one entry of one name.

 @param claim - the attempt's token, process number and time

 @param name - the entry's name

 @param queueId - the identity of the entry the claim is taken on

 @returns the claim as its file holds it

 @example
 ```ts
 const held = aClaimOf({ claim, name: 'a', queueId: 'q1' });
 ```
 */
export function aClaimOf({
  claim,
  name,
  queueId,
}: {
  readonly claim: QueueClaim;
  readonly name: string;
  readonly queueId: string;
},): AClaimOnAnEntry {
  return {
    name,
    queueId,
    claim,
  };
}
