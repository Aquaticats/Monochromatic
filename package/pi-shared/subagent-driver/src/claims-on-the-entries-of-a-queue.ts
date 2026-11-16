/**
 The claim one filling holds on one entry of a queue, and the three steps that use it: taking it before anything is started, completing it
 when the entry has started, and giving it back when the machine refused the start.

 An entry used to be started first and taken out of the queue afterwards, by name, so two fillings started one entry, a filling killed
 between its start and its removal left the entry to be started again, and a completed start removed whatever entry bore that name when it
 finished, which could be a replacement queued meanwhile. A name is not an identity: the entry is known by the `queueId` this module mints
 when it is first claimed, and the attempt to start it by the token of its claim, and every step here matches on both.

 The claim is written under the queue's lock, so exactly one filling takes it; the start happens outside the lock, because a review's
 brief is minutes of git; and the batch list, its unit and that unit's output file are named from the claim's token, so the claim names the
 evidence a reader asks about after a kill without naming a path.

 What a kill leaves, and what the next pass does with it, is written in the README's "The states of an entry, and the one step between
 each two", and settled by `settling-the-queue-after-a-kill.ts`.

 @module
 */

import { randomUUID, } from 'node:crypto';
import { join, } from 'node:path';
import { isThere, } from './newest-finished-run-of-a-name.ts';
import {
  NO_CLAIM_ON_AN_ENTRY,
  NO_QUEUE_ID,
  type QueueClaim,
  type QueueRecord,
  aFreshQueueId,
  changeTheQueueOfRuns,
  claimOnAnEntry,
  queueIdOf,
  withTheClaim,
} from './queue-of-runs.ts';

/**
 What taking a claim on one entry answered: the claim this filling holds, or why there was nothing to claim.
 */
export type ClaimedAnEntry =
  | {
    readonly kind: 'claimed';

    /**
     The entry's own identity, minted by this claim when it had none.
     */
    readonly queueId: string;

    /**
     The claim this filling holds.
     */
    readonly claim: QueueClaim;
  }
    | {
      /**
       The entry left the queue before this filling claimed it, so nothing is started for it.
       */
      readonly kind: 'no-longer-queued';
    }
    | {
      /**
       Another filling holds a claim on the entry, so this one starts nothing for it and asks about the entry behind it.
       */
      readonly kind: 'claimed-by-another';
    };

/**
 Characters in the stamp a batch list's name begins with, up to the second: `2026-11-14T09:21:14`.
 */
const STAMP_CHARACTERS = 19;

/**
 How many characters of an entry's name go into the batch list's file name, which also becomes its unit's name.
 */
const NAME_CHARACTERS_IN_A_FILE = 60;

/**
 The extension of the batch list a start writes, which its unit's output file is named from.
 */
const LIST_EXTENSION = '.json';

/**
 The ending of the output file one attempt's unit writes both of its streams to.
 */
const OUTPUT_EXTENSION = '.out';

/**
 The batch list, the unit and the output file one start attempt is named with.

 The names come from the claim's token, so the filling that recovers a dead claim asks about the same file the dead attempt would have
 written, and a second attempt on the same entry never answers for the first.

 @param runsFolder - folder the driver writes its runs into

 @param name - the entry's name

 @param claim - the claim naming the attempt

 @returns the list's path and the path the attempt's unit writes its streams to

 @example
 ```ts
 const names = namesOfAnAttemptIn({ runsFolder: '/runs', name: 'a', claim });
 ```
 */
export function namesOfAnAttemptIn({
  runsFolder,
  name,
  claim,
}: {
  readonly runsFolder: string;
  readonly name: string;
  readonly claim: QueueClaim;
},): {
  readonly batchFile: string;
  readonly outputFile: string;
} {
  /**
   The stamp the list's name begins with, taken from the claim's own time so every reader of the attempt names it alike.
   */
  const stamp = claim.at
    .slice(
    0,
    STAMP_CHARACTERS,
    )
    .replaceAll(
    ':',
    '',
    );
  /**
   The list's path, beside the runs, named after the attempt and the entry.
   */
  const batchFile = join(
    runsFolder,
    `batch-${stamp}-${claim.token}-queue-entry-${name.slice(
      0,
      NAME_CHARACTERS_IN_A_FILE,
    )}${LIST_EXTENSION}`,
  );
  return {
    batchFile,
    outputFile: `${batchFile.slice(
      0,
      -LIST_EXTENSION.length,
    )}${OUTPUT_EXTENSION}`,
  };
}

/**
 Says whether one attempt's start took effect: whether its unit wrote the output file it was given.

 A unit's output file is opened by the manager when the unit starts, and it stays after the unit ends, so its presence is the evidence
 that the attempt was started. A claim whose holder is gone and whose output file is not there names an attempt that never reached its
 unit, which is the one an entry may be started for again.

 @param runsFolder - folder the driver writes its runs into

 @param name - the entry's name

 @param claim - the claim naming the attempt

 @returns whether the attempt's unit wrote its output

 @example
 ```ts
 const started = await startOfAnAttemptTookEffect({ runsFolder: '/runs', name: 'a', claim });
 ```
 */
export async function startOfAnAttemptTookEffect({
  runsFolder,
  name,
  claim,
}: {
  readonly runsFolder: string;
  readonly name: string;
  readonly claim: QueueClaim;
},): Promise<boolean> {
  /**
   Where the attempt's unit would write both of its streams.
   */
  const { outputFile, } = namesOfAnAttemptIn({
    runsFolder,
    name,
    claim,
  },);
  return await isThere(outputFile,);
}

/**
 Says whether the queue still holds the very entry this filling read.

 @param queued - the entry the queue holds

 @param read - the entry read before the claim was asked for

 @returns whether the queue still holds that entry

 @example
 ```ts
 const same = theEntryAsRead({ queued, read });
 ```
 */
function theEntryAsRead({
  queued,
  read,
}: {
  readonly queued: QueueRecord;
  readonly read: QueueRecord;
},): boolean {
  /**
   The fields the queue's entry has, which are the fields an entry read from it has: both are JSON objects read from the same file.
   */
  const fields = Object.keys(queued,);
  if (fields.length
    !== Object.keys(read,)
      .length) {
    return false;
  }
  return fields.every(function isTheSame(field,) {
    return JSON.stringify(queued[field],) === JSON.stringify(read[field],);
  },);
}

/**
 Mints an identity no entry of the queue holds yet.

 @param entries - the entries the queue holds

 @returns a fresh identity

 @example
 ```ts
 const queueId = anIdentityNoEntryHolds({ entries });
 ```
 */
function anIdentityNoEntryHolds({ entries }: { readonly entries: readonly QueueRecord[] },): string {
  for (;;) {
    /**
     The identity just minted, which an entry already queued may already carry.
     */
    const minted = aFreshQueueId();
    if (!entries.some(function holdsIt(entry,) {
      return queueIdOf({ record: entry, },) === minted;
    },)) {
      return minted;
    }
  }
}

/**
 Claims one entry of the queue before anything is started for it, so exactly one filling starts it.

 The claim is one write under the queue's lock, matched on the entry as it was read and on its holding no claim: another filling that read
 the same entry writes second and is answered that the entry is claimed.

 @param queueFile - the queue's path

 @param record - the entry as this filling read it

 @param now - the clock the claim's time is read from

 @param waitMilliseconds - how long to keep asking for the queue's lock before refusing

 @returns the claim this filling holds and the entry's identity, or why there was nothing to claim

 @throws QueueFileError when the queue cannot be read or written

 @throws QueueLockHeldError when another writer holds the queue for the lock's whole wait

 @example
 ```ts
 const taken = await claimAnEntryOfAQueue({ queueFile: '/runs/q.json', record, now: Date.now });
 ```
 */
export async function claimAnEntryOfAQueue({
  queueFile,
  record,
  now,
  waitMilliseconds,
}: {
  readonly queueFile: string;
  readonly record: QueueRecord;
  readonly now: () => number;
  readonly waitMilliseconds?: number;
},): Promise<ClaimedAnEntry> {
  return await changeTheQueueOfRuns<ClaimedAnEntry>({
    queueFile,
    ...(waitMilliseconds === undefined ? {} : { waitMilliseconds, }),
    change: function claimIt(entries,) {
      /**
       Where the entry this filling read stands in the queue now, or minus one when the queue no longer holds it as it was read.
       */
      const at = entries.findIndex(function isTheEntryRead(queued,) {
        return (queued.name === record.name)
          && (claimOnAnEntry({ record: queued, },) === NO_CLAIM_ON_AN_ENTRY)
          && theEntryAsRead({
            queued,
            read: record,
          },);
      },);
      if (at === (-1)) {
        return {
          entries,
          answer: entries.some(function isNamed(entry,) {
            return entry.name === record.name;
          },)
            ? { kind: 'claimed-by-another', }
            : { kind: 'no-longer-queued', },
        };
      }
      /**
       The entry as the queue holds it now, which is there because the index above found it.
       */
      const queued = entries[at] ?? record;
      /**
       The identity the queue already holds for the entry, or one minted now.
       */
      const known = queueIdOf({ record: queued, },);
      /**
       The entry's identity, minted now when no write has claimed the entry before.
       */
      const queueId = (known === NO_QUEUE_ID)
        ? anIdentityNoEntryHolds({ entries, },)
        : known;
      /**
       The claim this filling takes, whose token names the one start attempt.
       */
      const claim: QueueClaim = {
        token: randomUUID(),
        pid: process.pid,
        at: new Date(now(),).toISOString(),
      };
      /**
       The entry with its identity and its claim.
       */
      const claimed = withTheClaim({
        record: {
          ...queued,
          queueId,
        },
        claim,
      },);
      return {
        entries: [
          ...entries.slice(
            0,
            at,
          ),
          claimed,
          ...entries.slice(at + 1,),
        ],
        answer: {
          kind: 'claimed',
          queueId,
          claim,
        },
      };
    },
  },);
}
