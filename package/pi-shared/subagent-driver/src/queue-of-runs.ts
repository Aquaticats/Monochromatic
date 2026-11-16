/**
 One entry of the queue of Pi runs waiting for a free place, and the names a queue folder gives it: the folder is one file per entry, named
 after the entry, and a claim on one entry is one file beside the entries, named after the entry and holding the attempt's own token.

 A name is not an identity. An entry's name is refused while the folder holds an entry of that name, which is what makes two runs of one
 name impossible; but a name may be queued again once its earlier entry has gone, so the entry a filling starts, sets aside or gives back
 is told apart from a namesake by the `queueId` its own write minted.

 The fields a queue's bookkeeping writes into an entry are named here, and no entry shape reads them: `order`, which is the entry's place
 in the queue, and `queueId`, which is its identity.

 @module
 */

import { randomUUID, } from 'node:crypto';
import { join, } from 'node:path';
import {
  anOrderBetween,
  anOrderIn,
  type NOT_AN_ORDER,
} from './order-of-an-entry-in-a-queue.ts';

/**
 One name in a folder, as reading the folder reports it: the name, and whether it is a file.
 */
export type ANameInAFolder = {
  /**
   The name inside the folder.
   */
  readonly name: string;

  /**
   Says whether the name is a file, which a folder and a socket are not.
   
   @returns whether it is a file
   */
  readonly isFile: () => boolean;
};

/**
 One entry of a queue, as its file holds it: a JSON object named by `name`.

 The fields a run or a review needs are read by `entry-of-a-queue.ts`, which refuses an entry it cannot take rather than dropping a
 field; this module only knows that every entry is an object with a name.
 */
export type QueueRecord = Readonly<Record<string, unknown> & { readonly name: string }>;

/**
 A queue folder, or one entry in it, that cannot be read as a queue.
 */
export class QueueFileError extends Error {
  /**
   The name printed with the message.
   */
  override readonly name = 'QueueFileError';
}

/**
 The field holding an entry's place in the queue, which the put that writes the entry decides and which no entry shape reads.
 */
export const ORDER_FIELD = 'order';

/**
 The field holding an entry's own identity, which the write that puts the entry into a queue mints.
 */
export const QUEUE_ID_FIELD = 'queueId';

/**
 The fields a queue's own bookkeeping writes into an entry, and which no entry shape reads.
 */
export const QUEUE_BOOKKEEPING_FIELDS: readonly string[] = [
  ORDER_FIELD,
  QUEUE_ID_FIELD,
];

/**
 The folder inside a queue folder that holds one claim for each entry a filling is starting.
 */
export const CLAIMED_FOLDER = 'claimed';

/**
 The ending of the file one entry is written to, and of the file one claim is written to.
 */
const ENTRY_FILE_ENDING = '.json';

/**
 The ending of the file one claim is written to, after the entry's own name.
 */
const CLAIM_FILE_ENDING = '.claim.json';

/**
 The start of the name of a file written beside another and renamed or linked into place, which is never read as anything, and which
 every reader of a queue folder therefore skips.
 */
export const BEING_WRITTEN_START = '.being-written-';

/**
 The start of the name an entry is renamed to while a command takes it out of the queue, so the entry a reader asked for is the one the
 rename moved and not whatever was put under that name while the read ran.
 */
export const BEING_TAKEN_START = '.being-taken-';

/**
 The claim one filling holds on one entry while it starts it.

 The token names the start attempt, and the batch list of that attempt, the unit it starts in and the unit's output file are all named from
 it (`namesOfAnAttemptIn` in `claiming-an-entry-of-a-queue.ts`), so a claim names its own evidence without naming a path.
 */
export type QueueClaim = {
  /**
   The token naming this one start attempt, minted when the claim was written.
   */
  readonly token: string;

  /**
   The process number of the filling that holds the claim.
   */
  readonly pid: number;

  /**
   When the claim was taken, as ISO 8601.
   */
  readonly at: string;
};

/**
 The claim one filling holds on one entry, as the claim's file holds it: which entry it is for, which entry of that name it is, and the
 attempt's own facts.
 */
export type AClaimOnAnEntry = {
  /**
   The entry's name.
   */
  readonly name: string;

  /**
   The identity of the entry the claim was taken on, which tells it from a namesake queued later.
   */
  readonly queueId: string;

  /**
   The attempt the claim names.
   */
  readonly claim: QueueClaim;
};

/**
 Says whether a value is one entry of a queue: a JSON object named by a non-empty `name`.

 @param value - the value, of any shape

 @returns whether it is one entry of a queue

 @example
 ```ts
 const isOne = isQueueRecord({ name: 'a' });
 ```
 */
export function isQueueRecord(value: unknown,): value is QueueRecord {
  return ((typeof value) === 'object')
    && (value !== null)
    && ('name' in value)
    && ((typeof value.name) === 'string')
    && (value.name !== '');
}

/**
 Mints an identity for one entry.

 @returns a fresh identity

 @example
 ```ts
 const queueId = aFreshQueueId();
 ```
 */
export function aFreshQueueId(): string {
  return randomUUID();
}

/**
 No identity, as a symbol so no identity can be mistaken for one: an entry is written by no put until it is queued.
 */
export const NO_QUEUE_ID: unique symbol = Symbol('this entry has been written into a queue by no put',);

/**
 Reads one entry's own identity.

 @param record - the entry

 @returns the identity the queue wrote, or that no write has put the entry into a queue

 @example
 ```ts
 const id = queueIdOf({ record: { name: 'a', queueId: 'q1' } });
 ```
 */
export function queueIdOf({ record, }: { readonly record: QueueRecord },): string | typeof NO_QUEUE_ID {
  /**
   What the identity field holds.
   */
  const written: unknown = record[QUEUE_ID_FIELD];
  return ((typeof written) === 'string') && (written !== '') ? written : NO_QUEUE_ID;
}

/**
 Reads one entry's place in the queue.

 @param record - the entry

 @returns the order key, or that the entry carries none this module wrote

 @example
 ```ts
 const order = orderOf({ record: { name: 'a', order: 'i' } });
 ```
 */
export function orderOf({ record, }: { readonly record: QueueRecord },): string | typeof NOT_AN_ORDER {
  return anOrderIn({ order: record[ORDER_FIELD], },);
}

/**
 Reads the name a file gives one entry, which is the entry's name written so that it is one file's name and no other entry's.

 A name is spelled with percent escapes for every character that is not safe in a file name, which keeps two names two file names and
 keeps a name from reaching outside the folder. The name itself is inside the file, so nothing has to be decoded to read an entry.

 @param name - the entry's name

 @returns the file's name, with its ending

 @example
 ```ts
 const file = anEntryFileName({ name: 'review-a' });
 ```
 */
export function anEntryFileName({ name, }: { readonly name: string },): string {
  /**
   The name as a file name, where only the characters a file name may carry unescaped stay as they are.
   */
  const escaped = encodeURIComponent(name,);
  // A name that begins with a dot would be a hidden file, and `..` would escape the folder, so the first dot is escaped.
  return `${escaped.startsWith('.' as string,) ? `%2E${escaped.slice(1,)}` : escaped}${ENTRY_FILE_ENDING}`;
}

/**
 Reads the path of the file one entry is written to.

 @param queueFolder - the queue's folder

 @param name - the entry's name

 @returns the path

 @example
 ```ts
 const file = entryFileOf({ queueFolder: '/runs/q', name: 'a' });
 ```
 */
export function entryFileOf({
  queueFolder,
  name,
}: {
  readonly queueFolder: string;
  readonly name: string;
},): string {
  return join(
    queueFolder,
    anEntryFileName({ name, },),
  );
}

/**
 Reads the folder a queue's claims are written in.

 @param queueFolder - the queue's folder

 @returns the path of the folder

 @example
 ```ts
 const claimed = claimedFolderOf({ queueFolder: '/runs/q' });
 ```
 */
export function claimedFolderOf({ queueFolder, }: { readonly queueFolder: string },): string {
  return join(
    queueFolder,
    CLAIMED_FOLDER,
  );
}

/**
 Reads the path of the file one entry's claim is written to.

 @param queueFolder - the queue's folder

 @param name - the entry's name

 @returns the path

 @example
 ```ts
 const file = claimFileOf({ queueFolder: '/runs/q', name: 'a' });
 ```
 */
export function claimFileOf({
  queueFolder,
  name,
}: {
  readonly queueFolder: string;
  readonly name: string;
},): string {
  return join(
    claimedFolderOf({ queueFolder, },),
    `${anEntryFileName({ name, },)
      .slice(
        0,
        -ENTRY_FILE_ENDING.length,
      )}${CLAIM_FILE_ENDING}`,
  );
}
