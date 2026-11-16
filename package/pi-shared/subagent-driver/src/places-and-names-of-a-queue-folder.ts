/**
 Where one entry stands in a queue folder and what a command calls the files it works on: the order keys a put and a move write, the key
 between two places, the name a file takes while a command takes it out, and the claim one attempt writes into the claims folder.

 @module
 */

import { randomUUID, } from 'node:crypto';
import {
  basename,
  dirname,
  join,
} from 'node:path';
import {
  type AClaimOnAnEntry,
  BEING_TAKEN_START,
  ORDER_FIELD,
  type QueueClaim,
  type QueueRecord,
  QueueFileError,
} from './queue-of-runs.ts';
import { anOrderBetween, } from './order-of-an-entry-in-a-queue.ts';

/**
 Reads the order key of one entry of the queue.

 @param entry - the entry

 @returns the key

 @throws QueueFileError when the entry carries none

 @example
 ```ts
 const key = orderKeyOf({ entry });
 ```
 */
function orderKeyOf({ entry, }: { readonly entry: QueueRecord },): string {
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
  readonly order: string;
} {
  /**
   Where the entry lands.
   */
  const at = Math.min(
    position,
    entries.length,
  );
  /**
   The key of the entry before it, when one stands there.
   */
  /**
   The entry before the place, when one stands there.
   */
  const before = (at > 0) ? entries[at - 1] : undefined;
  /**
   The key of the entry before the place, when one stands there.
   */
  const lower = (before === undefined) ? undefined : orderKeyOf({ entry: before, },);
  /**
   The entry after the place, when one stands there.
   */
  const after = (at < entries.length) ? entries[at] : undefined;
  /**
   The key of the entry after the place, when one stands there.
   */
  const upper = (after === undefined) ? undefined : orderKeyOf({ entry: after, },);
  return {
    at,
    order: anOrderForAPlace({
      ...(lower === undefined ? {} : { lower, }),
      ...(upper === undefined ? {} : { upper, }),
    },),
  };
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
