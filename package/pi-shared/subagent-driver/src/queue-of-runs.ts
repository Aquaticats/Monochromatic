/**
 The queue of Pi runs waiting for a free place: one JSON array of entries, in the order they are to start, and the four moves a
 coordinator makes on it.

 The file is read and written whole, and every write goes to a file beside the queue and is renamed over it, so a reader never
 meets half a queue. Every change reads the queue, changes it and writes it while the queue's lock is held (`queue-lock.ts`), so two
 writers can never both read before either writes and drop one another's entry: that lost update was reproduced with two processes
 against the queue of 2026-11-14, 47 rounds of 200 losing an entry while both commands exited 0. The lock is one file beside the
 queue, taken by linking a file that names its holder into the lock's place, and it is released whatever the change answered.

 @module
 */

import { randomUUID, } from 'node:crypto';
import {
  acquireTheQueueLock,
  assertTheLockIsStillOurs,
} from './queue-lock.ts';
import {
  type QueueRecord,
  QueueFileError,
  readQueueOfRuns,
  writeTheQueueWhole,
} from './reading-and-writing-the-queue-file.ts';

export {
  type QueueRecord,
  QueueFileError,
  isQueueRecord,
  readQueueOfRuns,
  writtenQueueOfRuns,
} from './reading-and-writing-the-queue-file.ts';

/**
 The field holding an entry's own identity, which the write that claims the entry mints.
 */
export const QUEUE_ID_FIELD = 'queueId';

/**
 The field holding the claim one filling has on an entry, which no other filling starts it under.
 */
export const QUEUE_CLAIM_FIELD = 'queueClaim';

/**
 No identity yet, as a symbol so no identity can be mistaken for one: an entry is written by no claim until a filling takes one.
 */
export const NO_QUEUE_ID: unique symbol = Symbol('this entry has been claimed by no write',);

/**
 No claim on that entry, as a symbol so no claim can be mistaken for one.
 */
export const NO_CLAIM_ON_AN_ENTRY: unique symbol = Symbol('this entry carries the claim of no filling',);

/**
 The fields a queue's own bookkeeping writes into an entry, and which no entry shape reads.

 A name is not an identity: a name may be queued again once its earlier entry has gone, so the state of one entry is known by
 its identity and by the token of the attempt holding it.
 */
export const QUEUE_BOOKKEEPING_FIELDS: readonly string[] = [
  QUEUE_ID_FIELD,
  QUEUE_CLAIM_FIELD,
];

/**
 The claim one filling holds on one entry while it starts it.

 The token names the start attempt, and the batch list of that attempt, the unit it starts in and that unit's output file are
 all named from it (`namesOfAnAttemptIn` in `claims-on-the-entries-of-a-queue.ts`), so a claim names its own evidence without
 naming a path.
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
 Reads the value of one field of an entry, of any shape.

 @param record - the entry

 @param field - the field's name

 @returns what it holds, or nothing when it names the field not at all

 @example
 ```ts
 const written = fieldOf({ record: { name: 'a', queueId: 'q1' }, field: 'queueId' });
 ```
 */
function fieldOf({
  record,
  field,
}: {
  readonly record: QueueRecord;
  readonly field: string;
},): unknown {
  return record[field];
}

/**
 Reads one entry's own identity.

 @param record - the entry

 @returns the identity the queue wrote, or nothing when the entry has none yet

 @example
 ```ts
 const id = queueIdOf({ record: { name: 'a', queueId: 'q1' } });
 ```
 */
export function queueIdOf({ record, }: { readonly record: QueueRecord },): string | typeof NO_QUEUE_ID {
  /**
   What the identity field holds.
   */
  const written = fieldOf({
    record,
    field: QUEUE_ID_FIELD,
  },);
  return ((typeof written) === 'string') && (written !== '') ? written : NO_QUEUE_ID;
}

/**
 Reads the claim a filling holds on one entry.

 @param record - the entry

 @returns the claim, or nothing when no filling holds one or the field holds no claim

 @example
 ```ts
 const claim = claimOnAnEntry({ record: { name: 'a', queueClaim: { token: 't', pid: 1, at: '2026-11-15T00:00:00.000Z' } } });
 ```
 */
export function claimOnAnEntry({ record, }: { readonly record: QueueRecord },): QueueClaim | typeof NO_CLAIM_ON_AN_ENTRY {
  /**
   What the claim field holds.
   */
  const written = fieldOf({
    record,
    field: QUEUE_CLAIM_FIELD,
  },);
  if (((typeof written) !== 'object') || (written === null)) {
    return NO_CLAIM_ON_AN_ENTRY;
  }
  /**
   What the field says a claim is.
   */
  const token: unknown = Reflect.get(
    written,
    'token',
  );
  /**
   The process number it names.
   */
  const pid: unknown = Reflect.get(
    written,
    'pid',
  );
  /**
   The time it names.
   */
  const at: unknown = Reflect.get(
    written,
    'at',
  );
  if (
    ((typeof token) !== 'string')
      || (token === '')
      || ((typeof pid) !== 'number')
      || ((typeof at) !== 'string')
  ) {
    return NO_CLAIM_ON_AN_ENTRY;
  }
  return {
    token,
    pid,
    at,
  };
}

/**
 Reads one entry with the claim a filling holds on it, and nothing else changed.

 @param record - the entry

 @param claim - the claim to write on it

 @returns the entry holding that claim

 @example
 ```ts
 const claimed = withTheClaim({ record: { name: 'a' }, claim });
 ```
 */
export function withTheClaim({
  record,
  claim,
}: {
  readonly record: QueueRecord;
  readonly claim: QueueClaim;
},): QueueRecord {
  return {
    ...record,
    [QUEUE_CLAIM_FIELD]: claim,
  };
}

/**
 Reads one entry with its claim gone, which is the entry a give-back leaves and the entry a set-aside record holds.

 @param record - the entry

 @returns the entry without a claim

 @example
 ```ts
 const queued = withoutTheClaim({ record: { name: 'a', queueClaim: claim } });
 ```
 */
export function withoutTheClaim({ record, }: { readonly record: QueueRecord },): QueueRecord {
  /**
   The entry's fields without the claim's.
   */
  const {
    [QUEUE_CLAIM_FIELD]: _claim,
    ...rest
  } = record;
  return rest;
}

/**
 Says whether one entry is the attempt named: its own identity and that attempt's token, both.

 A name is not an identity, so a removal or a give-back matched on a name alone would act on an entry queued under that name
 after the one read, which is how a completed start once deleted replacement user data.

 @param record - the entry

 @param queueId - the identity of the entry the attempt was claimed on

 @param token - the token naming the attempt

 @returns whether this entry is that attempt

 @example
 ```ts
 const mine = entryIsTheAttempt({ record, queueId: 'q1', token: 't1' });
 ```
 */
export function entryIsTheAttempt({
  record,
  queueId,
  token,
}: {
  readonly record: QueueRecord;
  readonly queueId: string;
  readonly token: string;
},): boolean {
  /**
   The claim the entry carries, when a filling holds one.
   */
  const claim = claimOnAnEntry({ record, },);
  return (queueIdOf({ record, },) === queueId)
    && (claim !== NO_CLAIM_ON_AN_ENTRY)
    && (claim.token === token);
}

/**
 Mints an identity for one entry, which the write that claims it writes.

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
 One change to make to a queue while it is held: the entries after the change, and what the caller is answered.
 */
export type ChangeOfAQueue<Answer> = (entries: readonly QueueRecord[]) => {
  /**
   The entries the queue holds after the change.
   */
  readonly entries: readonly QueueRecord[];

  /**
   What the caller is answered.
   */
  readonly answer: Answer;
};

/**
 The queue while one caller holds it: the entries it holds at that moment, and the one write that replaces them.

 A caller that decides anything about an entry's set-aside record must make that decision and write both files here, because the
 record and the queue are two files and a decision on one of them made outside this step is a decision on a state another writer may
 have changed: that is how a put-back deleted the record of a set-aside a filling was still committing, and how a second filling's
 sweep deleted a record a first had just written.
 */
export type HeldQueueOfRuns = {
  /**
   The entries the queue holds, read under the lock.
   */
  readonly entries: readonly QueueRecord[];

  /**
   Replaces the queue's entries with these, through a file beside the queue that is renamed over it.
   */
  readonly write: (entries: readonly QueueRecord[]) => Promise<void>;
};

/**
 Holds a queue's lock while one caller reads it, decides on it and writes it, and writes any set-aside record that decision governs.

 Every decision this package acts on is made here: which entry leaves the queue when it is set aside, which record goes when its
 entry is put back or when a killed set-aside is swept up, and where a put-back review goes. The lock is the one file beside the
 queue (`queue-lock.ts`), and it is given back when this block ends, answered or thrown.

 @param queueFile - the queue's path

 @param work - the decision and its writes, asked for the entries the queue holds under the lock

 @param now - the clock the lock's wait and age are read from

 @param wait - the pause between two attempts to take the lock

 @param waitMilliseconds - how long to keep asking for the lock before refusing

 @returns what the work answered

 @throws QueueFileError when the queue cannot be read or written, which leaves it as it was

 @throws QueueLockHeldError when another writer holds the queue for the lock's whole wait

 @example
 ```ts
 const answer = await withTheQueueLock({ queueFile: '/runs/queue.json', work: async ({ entries, write }) => { await write(entries); return entries.length; } });
 ```
 */
export async function withTheQueueLock<Answer>({
  queueFile,
  work,
  now,
  wait,
  waitMilliseconds,
}: {
  readonly queueFile: string;
  readonly work: (held: HeldQueueOfRuns) => Promise<Answer>;
  readonly now?: () => number;
  readonly wait?: (milliseconds: number) => Promise<unknown>;
  readonly waitMilliseconds?: number;
},): Promise<Answer> {
  /**
   The queue's lock, given back when this block ends whether the work answered or threw.
   */
  await using lock = await acquireTheQueueLock({
    queueFile,
    ...(now === undefined ? {} : { now, }),
    ...(wait === undefined ? {} : { wait, }),
    ...(waitMilliseconds === undefined ? {} : { waitMilliseconds, }),
  },);
  /**
   The queue as it stands under the lock.
   */
  const entries = await readQueueOfRuns({ queueFile, },);
  return await work({
    entries,
    async write(theQueue,) {
      // The token is checked before every write: a lock that was broken and taken by another caller while this one held the
      // queue would otherwise let this caller write the list it read over the taker's change. See `queue-lock.ts`.
      await assertTheLockIsStillOurs({
        lockFile: lock.lockFile,
        token: lock.token,
        queueFile,
      },);
      await writeTheQueueWhole({
        queueFile,
        entries: theQueue,
      },);
    },
  },);
}

/**
 Changes a queue while no other writer can: the read, the change and the write happen under the queue's lock.

 The change is asked for the entries the queue holds at that moment, so no entry another session queued meanwhile is written away.
 A change that answers the very array it was given writes nothing.

 @param queueFile - the queue's path

 @param change - the change to make, asked for the entries the queue holds under the lock

 @param now - the clock the lock's wait and age are read from

 @param wait - the pause between two attempts to take the lock

 @param waitMilliseconds - how long to keep asking for the lock before refusing

 @returns what the change answered

 @throws QueueFileError when the queue cannot be read or written, which leaves it as it was

 @throws QueueLockHeldError when another writer holds the queue for the lock's whole wait

 @example
 ```ts
 const answer = await changeTheQueueOfRuns({ queueFile: '/runs/queue.json', change: (entries) => ({ entries, answer: entries.length }) });
 ```
 */
export async function changeTheQueueOfRuns<Answer>({
  queueFile,
  change,
  now,
  wait,
  waitMilliseconds,
}: {
  readonly queueFile: string;
  readonly change: ChangeOfAQueue<Answer>;
  readonly now?: () => number;
  readonly wait?: (milliseconds: number) => Promise<unknown>;
  readonly waitMilliseconds?: number;
},): Promise<Answer> {
  return await withTheQueueLock({
    queueFile,
    ...(now === undefined ? {} : { now, }),
    ...(wait === undefined ? {} : { wait, }),
    ...(waitMilliseconds === undefined ? {} : { waitMilliseconds, }),
    work: async function changeIt({
      entries,
      write,
    },) {
      /**
       What the change made of the queue.
       */
      const changed = await Promise.resolve(change(entries,),);
      if (changed.entries !== entries) {
        await write(changed.entries,);
      }
      return changed.answer;
    },
  },);
}

/**
 Writes a queue file whole, holding the queue's lock, so no change of another writer is written away.

 @param queueFile - the queue's path

 @param entries - the entries, in the order they are to start

 @param now - the clock the lock's wait reads

 @param wait - the pause between attempts to take the lock

 @param waitMilliseconds - how long to keep asking for the lock before refusing

 @throws QueueFileError when the file cannot be written, which leaves the queue as it was

 @throws QueueLockHeldError when another writer holds the queue for the lock's whole wait

 @example
 ```ts
 await writeQueueOfRuns({ queueFile: '/runs/queue.json', entries: [] });
 ```
 */
export async function writeQueueOfRuns({
  queueFile,
  entries,
  now,
  wait,
  waitMilliseconds,
}: {
  readonly queueFile: string;
  readonly entries: readonly QueueRecord[];
  readonly now?: () => number;
  readonly wait?: (milliseconds: number) => Promise<unknown>;
  readonly waitMilliseconds?: number;
},): Promise<void> {
  // A whole-file write reads nothing, so it makes a queue file that is not there yet, which is how every caller's queue starts.
  /**
   The queue's lock, given back when this block ends whether the write answered or threw.
   */
  await using lock = await acquireTheQueueLock({
    queueFile,
    ...(now === undefined ? {} : { now, }),
    ...(wait === undefined ? {} : { wait, }),
    ...(waitMilliseconds === undefined ? {} : { waitMilliseconds, }),
  },);
  await assertTheLockIsStillOurs({
    lockFile: lock.lockFile,
    token: lock.token,
    queueFile,
  },);
  await writeTheQueueWhole({
    queueFile,
    entries,
  },);
}

/**
 Takes every entry of one name out of the queue, reading the queue again at the moment of writing.

 @param queueFile - the queue's path

 @param name - the entry's name

 @returns the entries taken out, and the entries the queue holds afterwards

 @throws QueueFileError when the queue cannot be read or written

 @example
 ```ts
 const { taken, entries } = await takeAQueueEntryOut({ queueFile: '/runs/queue.json', name: 'a' });
 ```
 */
export async function takeAQueueEntryOut({
  queueFile,
  name,
}: {
  readonly queueFile: string;
  readonly name: string;
},): Promise<{
  readonly taken: readonly QueueRecord[];
  readonly entries: readonly QueueRecord[];
}> {
  return await changeTheQueueOfRuns({
    queueFile,
    change: function takeThemOut(queue,) {
      /**
       The entries of that name.
       */
      const taken = queue.filter(function isNamed(entry,) {
        return entry.name === name;
      },);
      if (taken.length === 0) {
        return {
          entries: queue,
          answer: {
            taken,
            entries: queue,
          },
        };
      }
      /**
       The queue without them.
       */
      const kept = queue.filter(function isNotNamed(entry,) {
        return entry.name !== name;
      },);
      return {
        entries: kept,
        answer: {
          taken,
          entries: kept,
        },
      };
    },
  },);
}

/**
 Puts one entry into the queue at a position counted from the head.

 An entry of that name is refused rather than queued twice: two runs of one name collide in every folder, unit and worktree the
 name gives them, so the second would replace the first rather than run beside it.

 @param queueFile - the queue's path

 @param entry - the entry to queue

 @param position - where it goes, counted from the head, `0` being the head

 @returns where it landed, and how many entries the queue now holds

 @throws QueueFileError when the queue cannot be read or written, or already holds an entry of that name

 @example
 ```ts
 const { position, count } = await putAQueueEntryIn({ queueFile: '/runs/queue.json', entry: { name: 'a' }, position: 0 });
 ```
 */
export async function putAQueueEntryIn({
  queueFile,
  entry,
  position,
}: {
  readonly queueFile: string;
  readonly entry: QueueRecord;
  readonly position: number;
},): Promise<{
  readonly position: number;
  readonly count: number;
}> {
  return await changeTheQueueOfRuns({
    queueFile,
    change: function putTheEntryIn(queue,) {
      if (queue.some(function isNamed(queued,) {
        return queued.name === entry.name;
      },)) {
        throw new QueueFileError(`an entry named ${entry.name} is already in the queue at ${queueFile}`,);
      }
      /**
       Where the entry lands, which is the head of a queue shorter than the position.
       */
      const at = Math.min(
        position,
        queue.length,
      );
      /**
       The queue with it in.
       */
      const next = [
        ...queue.slice(
          0,
          at,
        ),
        entry,
        ...queue.slice(at,),
      ];
      return {
        entries: next,
        answer: {
          position: at,
          count: next.length,
        },
      };
    },
  },);
}

/**
 Moves the entry of one name to a position counted from the head.

 @param queueFile - the queue's path

 @param name - the entry's name

 @param position - where it goes, counted from the head, `0` being the head

 @returns where it was and where it landed, or that the queue holds no entry of that name

 @throws QueueFileError when the queue cannot be read or written

 @example
 ```ts
 const moved = await moveAQueueEntryTo({ queueFile: '/runs/queue.json', name: 'a', position: 0 });
 ```
 */
export async function moveAQueueEntryTo({
  queueFile,
  name,
  position,
}: {
  readonly queueFile: string;
  readonly name: string;
  readonly position: number;
},): Promise<{
  readonly from?: number;
  readonly to?: number;
  readonly count: number;
}> {
  return await changeTheQueueOfRuns<{
    readonly from?: number;
    readonly to?: number;
    readonly count: number;
  }>({
    queueFile,
    change: function moveTheEntry(queue,) {
      /**
       Where the entry stands, or minus one when the queue holds none of that name.
       */
      const from = queue.findIndex(function isNamed(entry,) {
        return entry.name === name;
      },);
      if (from === (-1)) {
        return {
          entries: queue,
          answer: { count: queue.length, },
        };
      }
      /**
       The entry itself, taken as the one-element list it is so that no absent index has to be read.
       */
      const moved = queue.slice(
        from,
        from + 1,
      );
      /**
       The entries without it.
       */
      const rest = queue.filter(function isAnother(entry,) {
        return entry.name !== name;
      },);
      /**
       Where it lands, which is the tail of a queue shorter than the position.
       */
      const to = Math.min(
        position,
        rest.length,
      );
      /**
       The queue with it moved.
       */
      const next = [
        ...rest.slice(
          0,
          to,
        ),
        ...moved,
        ...rest.slice(to,),
      ];
      return {
        entries: next,
        answer: {
          from,
          to,
          count: next.length,
        },
      };
    },
  },);
}
