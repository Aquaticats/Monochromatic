/**
 The three moves a coordinator makes on a queue folder: one entry put in at a position, one taken out by name, and one moved to a position.
 Each is made of one atomic step on one entry's file, and nothing here waits on another writer.

 A put writes one file and rewrites no entry, because an entry's place in the queue is a key inside it (`order-of-an-entry-in-a-queue.ts`)
 and the key between any two keys is found digit by digit. A take renames the entry aside before it reads it, so the entry read is the one
 that was there and never one a person queued under that name while the read ran. A move rewrites the one entry whose place changes.

 @module
 */

import {
  aPlaceFor,
  aNameTakenAside,
} from './places-and-names-of-a-queue-folder.ts';
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
  NO_CLAIM_ON_THAT_ENTRY,
  claimAnEntryOfAQueue,
  giveBackTheClaim,
  theClaimOn,
} from './claiming-an-entry-of-a-queue.ts';
import { readTheQueueThatMayNotBeThere, } from './reading-a-queue-a-kill-left.ts';
import {
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
   The queue as it stands, which a queue folder that is not there yet holds nothing.
   */
  const entries = await readTheQueueThatMayNotBeThere({ queueFolder, },);
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
 queued under that name a moment later is left where it is. It is removed only after the caller has taken it (see
 {@link discardTheEntryTaken}), so a command whose reader went away leaves the entry where a person can find it.

 An entry a filling is starting is refused rather than taken: the person would otherwise hold a copy of an entry already running, and could
 queue it a second time. `--force` takes it anyway, which a person needs when a filling died holding a claim.

 @param queueFolder - the queue's folder

 @param name - the entry's name

 @param force - whether an entry a filling holds a claim on is taken anyway

 @returns the entries taken out, where each was left aside, and the entries the queue holds afterwards

 @throws QueueFileError when the queue cannot be read, or the entry taken aside holds no entry, which leaves it there for a person

 @throws AnotherFillingHoldsTheEntryError when a filling holds a claim on the entry and `force` is not given

 @example
 ```ts
 const { taken, asides, entries } = await takeAQueueEntryOut({ queueFolder: '/runs/q', name: 'a' });
 ```
 */
export async function takeAQueueEntryOut({
  queueFolder,
  name,
  force = false,
}: {
  readonly queueFolder: string;
  readonly name: string;
  readonly force?: boolean;
},): Promise<{
  readonly taken: readonly QueueRecord[];
  readonly asides: readonly string[];
  readonly entries: readonly QueueRecord[];
}> {
  /**
   The entry's file.
   */
  const file = entryFileOf({
    queueFolder,
    name,
  },);
  if (!force) {
    /**
     The claim a filling holds on the entry, or that it holds none.
     */
    const claimed = await theClaimOn({
      queueFolder,
      name,
    },);
    if (claimed !== NO_CLAIM_ON_THAT_ENTRY) {
      throw new AnotherFillingHoldsTheEntryError(
        `${name} is being started by the filling of process ${String(claimed.claim
          .pid,)} since ${claimed.claim
            .at}, so it is not taken: `
          + 'the person would hold a copy of an entry that is already running. Take it with --force when that filling is gone, and the '
          + 'entry it is starting is left to the settlement.',
      );
    }
  }
  /**
   Where the entry is renamed to while it is read, and stays until the caller takes it.
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
        asides: [],
        entries: await readQueueOfRuns({ queueFolder, },),
      };
    }
    throw new QueueFileError(`the entry at ${file} cannot be taken out of the queue: ${String(error,)}`,);
  }
  /**
   The entry the rename moved, which is the one that was there.
   */
  const entry = await readTheEntryAt({ path: aside, },);
  return {
    taken: [entry,],
    asides: [aside,],
    entries: await readQueueOfRuns({ queueFolder, },),
  };
}

/**
 Discards one entry a take left aside, once the caller has taken it.

 A command that could not show the entry it took keeps the file, so the entry is still somewhere a person can read: nothing is taken out of
 a queue before the person has seen it.

 @param aside - the path a take left its entry at

 @throws QueueFileError when the file cannot be removed

 @example
 ```ts
 await discardTheEntryTaken({ aside: '/runs/q/.being-taken-a.json-1-2' });
 ```
 */
export async function discardTheEntryTaken({ aside, }: { readonly aside: string },): Promise<void> {
  try {
    await rm(
      aside,
      { force: true, },
    );
  }
  catch (error) {
    throw new QueueFileError(`the entry taken aside at ${aside} cannot be removed: ${String(error,)}`,);
  }
}

/**
 A filling holds a claim on the entry a person asked to take, so taking it would hand the person an entry that is already running.
 */
export class AnotherFillingHoldsTheEntryError extends QueueFileError {
  /**
   The name printed with the message.
   */
  override readonly name: string = 'AnotherFillingHoldsTheEntryError';
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
   The entry itself, which the queue holds because the index above found it.
   */
  const entry = entries[from];
  if (entry === undefined) {
    return { count: entries.length, };
  }
  // The entry is claimed while its place changes, which is the one thing that keeps a filling from starting it meanwhile: a filling's
  // claim and this one are the same link, so one of the two wins. A move that renamed a snapshot back over an entry a filling had already
  // started put that entry back into the queue and started it a second time.
  /**
   What taking a claim on the entry answered.
   */
  const claimed = await claimAnEntryOfAQueue({
    queueFolder,
    record: entry,
    now: Date.now,
  },);
  if (claimed.kind !== 'claimed') {
    return { count: entries.length, };
  }
  /**
   What moving the entry answered, or what ended the move before it answered.
   */
  const outcome = await theEntryMovedWithTheClaimHeld({
    queueFolder,
    name,
    position,
    claimed,
  },);
  // The claim goes whatever the move answered: a claim left by a live process keeps an entry where no filling starts it.
  await giveBackTheClaim({
    queueFolder,
    name,
    token: claimed.claim
      .token,
  },);
  if ('problem' in outcome) {
    throw outcome.problem;
  }
  return outcome.moved;
}

/**
 Moves one entry whose claim this caller holds, and answers where it landed.

 @param queueFolder - the queue's folder

 @param name - the entry's name

 @param position - where it goes, counted from the head

 @param claimed - the claim this caller holds on it

 @returns where it landed, or what ended the move before it answered

 @example
 ```ts
 const outcome = await theEntryMovedWithTheClaimHeld({ queueFolder, name, position, claimed });
 ```
 */
async function theEntryMovedWithTheClaimHeld({
  queueFolder,
  name,
  position,
  claimed,
}: {
  readonly queueFolder: string;
  readonly name: string;
  readonly position: number;
  readonly claimed: { readonly queueId: string };
},): Promise<
  | { readonly moved: {
    readonly from: number;
    readonly to: number;
    readonly count: number
  } }
    | { readonly problem: unknown }
  > {
  try {
    /**
     The queue as it stands once this move holds the entry, which is the queue the new place is counted in.
     */
    const now = await readQueueOfRuns({ queueFolder, },);
    /**
     Where the entry stands now, or minus one when it left the queue while the claim was being taken.
     */
    const at = now.findIndex(function isThisOne(other,) {
      return (other.name === name) && (queueIdOf({ record: other, },) === claimed.queueId);
    },);
    if (at === (-1)) {
      return { moved: {
        from: 0,
        to: 0,
        count: now.length
      }, };
    }
    /**
     The entries without it, which is the queue its new place is counted in.
     */
    const rest = now.filter(function isAnother(
      _,
      place,
    ) {
      return place !== at;
    },);
    /**
     Where it lands and the key it is written at.
     */
    const place = aPlaceFor({
      entries: rest,
      position,
    },);
    /**
     The entry as the queue holds it now, which is the one this move holds a claim on.
     */
    const held = now[at];
    if (held === undefined) {
      return { moved: {
        from: 0,
        to: 0,
        count: now.length
      }, };
    }
    /**
     The entry as it is written, which keeps the identity its claim was taken on.
     */
    const written: QueueRecord = {
      ...held,
      [ORDER_FIELD]: place.order,
    };
    await writeAnEntryOfAQueue({
      queueFolder,
      entry: written,
      replace: true,
    },);
    return {
      moved: {
        from: at,
        to: place.at,
        count: rest.length + 1,
      },
    };
  }
  catch (problem) {
    return { problem, };
  }
}
