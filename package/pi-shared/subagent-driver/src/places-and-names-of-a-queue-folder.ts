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
   The key of the entry before the place, when one stands there.
   */
  /**
   The entry before the place, when one stands there.
   */
  const before = (at > 0) ? entries[at - 1] : undefined;
  /**
   The key it holds, which the new entry must come after.
   */
  const lower = (before === undefined) ? NO_ENTRY_AFTER : orderKeyOf({ entry: before, },);
  /**
   The key the new entry must come before: the first key strictly greater than the lower one, which skips entries that hold the very
   key the new entry is placed after.

   Two entries can hold one key only in a queue written before puts kept their keys apart, and a new entry placed between them must land
   after that whole run of them rather than between two of them, which the key cannot express.
   */
  const upper = theFirstKeyAfter({
    entries,
    at,
    lower,
  },);
  /**
   The key between the two, which is the one this entry is written at.
   */
  const order = anOrderForAPlace({
    ...(lower === NO_ENTRY_AFTER ? {} : { lower, }),
    ...(upper === NO_ENTRY_AFTER ? {} : { upper, }),
  },);
  // A key another entry already holds is one a later put could not be placed between, so the key is parted once more until it is the
  // queue's own: the entry lands after the run of entries that hold the key it was given.
  return {
    at,
    order: anUnheldKey({
      order,
      entries,
    },),
  };
}

/**
 Reads the first key at or after one place that is greater than a key, which is the entry a put must come before.

 @param entries - the queue

 @param at - the place the put is made at

 @param lower - the key the put comes after, or nothing when it goes at the head

 @returns the key to come before, or nothing when the put goes at the tail

 @example
 ```ts
 const upper = theFirstKeyAfter({ entries, at: 0, lower: 'i' });
 ```
 */
function theFirstKeyAfter({
  entries,
  at,
  lower,
}: {
  readonly entries: readonly QueueRecord[];
  readonly at: number;
  readonly lower?: string | typeof NO_ENTRY_AFTER;
}): string | typeof NO_ENTRY_AFTER {
  for (const entry of entries.slice(at,)) {
    /**
     The key of this entry.
     */
    const key = orderKeyOf({ entry, },);
    if ((lower === undefined) || (lower === NO_ENTRY_AFTER)
      || (key > lower)) {
      return key;
    }
  }
  return NO_ENTRY_AFTER;
}

/**
 There is no entry after that place, so the put goes at the queue's tail.

 */
export const NO_ENTRY_AFTER: unique symbol = Symbol('that place is the queue at its tail',);

/**
 Reads a key no entry of the queue holds, by parting the key given once more for every entry that holds it.

 A key two entries hold is one a later put cannot be placed inside, because the key is the place: parting it puts the new entry after
 every entry that holds the key it started from, which is the place the caller asked for.

 @param order - the key the queue would write

 @param entries - the entries the queue holds

 @returns a key no entry holds, which sorts after or at the same place as the key given

 @example
 ```ts
 const order = anUnheldKey({ order: 'ii', entries });
 ```
 */
function anUnheldKey({
  order,
  entries,
}: {
  readonly order: string;
  readonly entries: readonly QueueRecord[];
},): string {
  /**
   The keys the queue holds.
   */
  const held = new Set(entries.map(function keyOf(entry,) {
    return orderKeyOf({ entry, },);
  },),);
  /**
   The key to write, parted for as long as another entry holds it.
   */
  let order2 = order;
  while (held.has(order2,)) {
    order2 = anOrderForAPlace({ lower: order2, },);
  }
  return order2;
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
