/**
 The claim one filling holds on one entry of a queue, and the four steps that use it: taking it before anything is started, completing
 it when the entry has started, giving it back when the machine refused the start, and recovering the claims of fillings that died
 with it.

 An entry used to be started first and taken out of the queue afterwards, by name, so two fillings started one entry, a filling
 killed between its start and its removal left the entry to be started again, and a completed start removed whatever entry bore that
 name when it finished, which could be a replacement queued meanwhile. A name is not an identity: the entry is known by the
 `queueId` this module mints when it is first claimed, and the attempt to start it by the token of its claim, and every step here
 matches on both.

 The claim is written under the queue's lock, so exactly one filling takes it; the start happens outside the lock, because a review's
 brief is minutes of git; and the batch list, its unit and that unit's output file are named from the claim's token, so the claim
 names the evidence a reader asks about after a kill without naming a path.

 What a kill leaves, and what the next pass does with it, is written in the README's "The states of an entry, and the one step
 between each two".

 @module
 */

import { randomUUID, } from 'node:crypto';
import { join, } from 'node:path';
import { isThere, } from './newest-finished-run-of-a-name.ts';
import { processIsThere, } from './queue-lock.ts';
import {
  aFreshQueueId,
  entryIsTheAttempt,
  type QueueClaim,
  type QueueRecord,
  claimOnAnEntry,
  queueIdOf,
  withTheClaim,
  withoutTheClaim,
  withTheQueueLock,
  changeTheQueueOfRuns,
} from './queue-of-runs.ts';
import {
  NO_SET_ASIDE_RECORD,
  removeSetAsideRecord,
  setAsideRecordOf,
} from './set-aside-entry.ts';

/**
 Characters in the stamp a batch list's name begins with, up to the second: `2026-11-14T09:21:14`.
 */
const STAMP_CHARACTERS = 19;

/**
 How many characters of an entry's name go into the batch list's file name, which also becomes its unit's name: the same number
 `starting-an-entry-of-a-queue.ts` uses, and this module writes the name the start must then use.
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
 The batch list, the unit and the output file one start attempt is named with.

 The names come from the claim's token, so the filling that recovers a dead claim asks about the same file the dead attempt would
 have written, and a second attempt on the same entry never answers for the first.

 @param request - the runs folder, the entry's name and the claim naming the attempt

 @returns the list's path, the unit's name and the path the unit writes its streams to

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
   The stamp the list's name begins with, taken from the claim's own time so both fillings name one attempt alike.
   */
  const stamp = claim.at.slice(
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

 A unit's output file is opened by the manager when the unit starts, and it stays after the unit ends, so its presence is the
 evidence that the attempt was started. A claim whose holder is gone and whose output file is not there names an attempt that never
 reached its unit, which is the one an entry may be started for again.

 @param request - the runs folder, the entry's name and the claim naming the attempt

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
 Says whether two records are the same entry: every field of the one read is in the other, and no other.

 @param queued - the entry the queue holds

 @param read - the entry read before the claim was asked for

 @returns whether the queue still holds that very entry

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
   The fields the queue's entry has, which are the fields an entry read from it has: a `QueueRecord` is a JSON object, and both were
   read from the same file.
   */
  const fields = Object.keys(queued,);
  if (fields.length !== Object.keys(read,).length) {
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
     The identity just minted.
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

 The claim is one write under the queue's lock, matched on the entry as it was read and on its holding no claim: another filling
 that read the same entry writes second and is answered that the entry is claimed.

 @param queueFile - the queue's path

 @param record - the entry as this filling read it

 @param now - the clock the claim's time is read from

 @param waitMilliseconds - how long to keep asking for the queue's lock before refusing

 @returns the claim this filling holds and the entry's identity, or that there was nothing to claim

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
        return (queued.name === record.name) && (claimOnAnEntry({ record: queued, },) === undefined)
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
       The entry as the queue holds it now.
       */
      const queued = entries[at] as QueueRecord;
      /**
       The entry's identity, minted now when no write has held the entry before.
       */
      const queueId = queueIdOf({ record: queued, },) ?? anIdentityNoEntryHolds({ entries, },);
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

/**
 Completes one claim by taking its entry out of the queue, matched on the entry's identity and on the claim's token.

 This is the step that makes a started entry started: a replacement queued under the same name is another entry with another
 identity, and is left where it is.

 @param queueFile - the queue's path

 @param queueId - the identity of the entry the attempt was claimed on

 @param token - the token naming the attempt

 @param waitMilliseconds - how long to keep asking for the queue's lock before refusing

 @returns how many entries were taken out, and how many the queue holds afterwards

 @throws QueueFileError when the queue cannot be read or written

 @throws QueueLockHeldError when another writer holds the queue for the lock's whole wait

 @example
 ```ts
 const completed = await completeTheClaim({ queueFile: '/runs/q.json', queueId: 'q1', token: 't1' });
 ```
 */
export async function completeTheClaim({
  queueFile,
  queueId,
  token,
  waitMilliseconds,
}: {
  readonly queueFile: string;
  readonly queueId: string;
  readonly token: string;
  readonly waitMilliseconds?: number;
},): Promise<{
  readonly taken: number;
  readonly left: number;
}> {
  return await changeTheQueueOfRuns({
    queueFile,
    ...(waitMilliseconds === undefined ? {} : { waitMilliseconds, }),
    change: function takeTheAttemptOut(entries,) {
      /**
       The entries that are not this attempt.
       */
      const kept = entries.filter(function isNoLongerTheAttempt(entry,) {
        return !entryIsTheAttempt({
          record: entry,
          queueId,
          token,
        },);
      },);
      return {
        entries: kept,
        answer: {
          taken: entries.length - kept.length,
          left: kept.length,
        },
      };
    },
  },);
}

/**
 Gives one claim back, leaving its entry where it stands in the queue for the next pass to start.

 This is what a failure of the machine leaves: the entry keeps its place, and the queue's order is the whole point of the queue.

 @param queueFile - the queue's path

 @param queueId - the identity of the entry the attempt was claimed on

 @param token - the token naming the attempt

 @param waitMilliseconds - how long to keep asking for the queue's lock before refusing

 @returns whether the claim was given back, and how many entries the queue holds

 @throws QueueFileError when the queue cannot be read or written

 @throws QueueLockHeldError when another writer holds the queue for the lock's whole wait

 @example
 ```ts
 const givenBack = await giveBackTheClaim({ queueFile: '/runs/q.json', queueId: 'q1', token: 't1' });
 ```
 */
export async function giveBackTheClaim({
  queueFile,
  queueId,
  token,
  waitMilliseconds,
}: {
  readonly queueFile: string;
  readonly queueId: string;
  readonly token: string;
  readonly waitMilliseconds?: number;
},): Promise<{
  readonly givenBack: boolean;
  readonly left: number;
}> {
  return await changeTheQueueOfRuns({
    queueFile,
    ...(waitMilliseconds === undefined ? {} : { waitMilliseconds, }),
    change: function giveItBack(entries,) {
      /**
       Whether this attempt was still the queue's.
       */
      let givenBack = false;
      /**
       The entries with this attempt's claim gone, and no other entry touched.
       */
      const next = entries.map(function withoutThisClaim(entry,) {
        if (!entryIsTheAttempt({
          record: entry,
          queueId,
          token,
        },)) {
          return entry;
        }
        givenBack = true;
        return withoutTheClaim({ record: entry, },);
      },);
      return {
        entries: next,
        answer: {
          givenBack,
          left: next.length,
        },
      };
    },
  },);
}

/**
 Settles a queue after a kill: finishes an interrupted set-aside, drops a record a completed put-back left behind, and recovers the
 claims of fillings that died holding them. One held step, so every decision is made on the queue and the records as they are.

 The records are settled first. An interrupted set-aside leaves a record and a claimed entry of one identity, and the claim's holder
 is gone with no output file, so a recovery that ran first would give that claim back and the settlement would then read the state
 as a killed put-back and drop the record, losing the reason its start failed and the whole of what that start printed.

 @param queueFile - the queue's path

 @param runsFolder - folder the driver writes its runs into

 @param report - where the pass says what it found, one line each

 @param now - the clock the lock's wait is read from

 @param wait - the pause between two attempts to take the lock

 @param waitMilliseconds - how long to keep asking for the queue's lock before refusing

 @returns how many entries the queue holds after the step

 @throws QueueFileError when the queue cannot be read or written

 @throws QueueLockHeldError when another writer holds the queue for the lock's whole wait

 @throws SetAsideRecordError when a record cannot be read as one, which changes nothing

 @example
 ```ts
 const left = await settleTheQueueAfterAKill({ queueFile: '/runs/q.json', runsFolder: '/runs', report: () => {} });
 ```
 */
export async function settleTheQueueAfterAKill({
  queueFile,
  runsFolder,
  report,
  now,
  wait,
  waitMilliseconds,
}: {
  readonly queueFile: string;
  readonly runsFolder: string;
  readonly report: (line: string) => void;
  readonly now?: () => number;
  readonly wait?: (milliseconds: number) => Promise<unknown>;
  readonly waitMilliseconds?: number;
},): Promise<number> {
  return await withTheQueueLock({
    queueFile,
    ...(now === undefined ? {} : { now, }),
    ...(wait === undefined ? {} : { wait, }),
    ...(waitMilliseconds === undefined ? {} : { waitMilliseconds, }),
    work: async function settle({ entries, write, },) {
      /**
       The entries as they stand after each decision.
       */
      let next = entries;
      /**
       Whether anything was decided, which is what says the queue has to be written.
       */
      let changed = false;
      /**
       When the lines are said.
       */
      const at = new Date().toISOString();
      /* oxlint-disable eslint/no-await-in-loop -- one entry after another, and each record is its own file */
      // The records first, so an interrupted set-aside is finished rather than read as a put-back whose record is stale.
      for (const entry of entries) {
        /**
         The entry's identity, when a claim or a record has written one.
         */
        const queueId = queueIdOf({ record: entry, },);
        /**
         The record beside the runs under this entry's name, when there is one.
         */
        const record = await setAsideRecordOf({
          runsFolder,
          name: entry.name,
        },);
        if (record === NO_SET_ASIDE_RECORD) {
          continue;
        }
        /**
         The identity the record's entry carries, when it carries one.
         */
        const recordId = queueIdOf({ record: record.entry, },);
        if (
          (queueId === undefined)
            || (recordId === undefined)
            || (recordId !== queueId)
        ) {
          // The record is another entry's only copy: it holds an entry queued under this name before, and no identity joins it to
          // the entry standing there now. Deleting it on the strength of the name is how a sweep once destroyed the only copy of an
          // entry whose name had been queued again.
          report(`${at} ${entry.name}: the set-aside record beside the runs holds an entry other than the one the queue holds under that name, so it is left as it is; putting it back or removing it is a person's to do\n`,);
          continue;
        }
        /**
         The claim the queued entry carries, when a filling holds one.
         */
        const claim = claimOnAnEntry({ record: entry, },);
        if (claim === undefined) {
          // The record's own entry is queued and unclaimed: a put-back wrote the entry and was killed before removing the record,
          // which leaves the record as a second copy of an entry that is already in the queue.
          await removeSetAsideRecord({
            runsFolder,
            name: entry.name,
          },);
          report(`${at} ${entry.name}: its set-aside record was still there after its entry was put back into the queue, so the record is dropped and the entry stands\n`,);
          continue;
        }
        // The record's own entry is queued and claimed: the set-aside wrote the record and was killed before its entry left the
        // queue, so this finishes it. The claim's token matters, so an entry queued under that name with another identity is never
        // taken out here.
        next = next.filter(function isNoLongerTheAttempt(queued,) {
          return !entryIsTheAttempt({
            record: queued,
            queueId,
            token: claim.token,
          },);
        },);
        changed = true;
        report(`${at} ${entry.name}: its set-aside wrote the record and was killed before the entry left the queue, so the set-aside is finished and the record stands\n`,);
      }
      // The claims of fillings that are gone, whose attempts are named by their own tokens.
      for (const entry of next) {
        /**
         The claim the entry carries, when a filling holds one.
         */
        const claim = claimOnAnEntry({ record: entry, },);
        if (claim === undefined) {
          continue;
        }
        if (processIsThere({ pid: claim.pid, },)) {
          continue;
        }
        /**
         The entry's identity, which a claim always wrote.
         */
        const queueId = queueIdOf({ record: entry, },);
        if (queueId === undefined) {
          continue;
        }
        // oxlint-disable-next-line eslint/no-await-in-loop -- the attempt of the entry just read
        if (await startOfAnAttemptTookEffect({
          runsFolder,
          name: entry.name,
          claim,
        },)) {
          next = next.filter(function isNoLongerTheAttempt(queued,) {
            return !entryIsTheAttempt({
              record: queued,
              queueId,
              token: claim.token,
            },);
          },);
          report(`${at} ${entry.name}: the filling that claimed it died after its start took effect, so it is taken out of the queue and never started again\n`,);
        }
        else {
          next = next.map(function withoutThisClaim(queued,) {
            return entryIsTheAttempt({
              record: queued,
              queueId,
              token: claim.token,
            },)
              ? withoutTheClaim({ record: queued, },)
              : queued;
          },);
          report(`${at} ${entry.name}: the filling that claimed it died before its start took effect, so the claim is given back and the entry stands for the next pass\n`,);
        }
        changed = true;
      }
      /* oxlint-enable eslint/no-await-in-loop */
      if (changed) {
        await write(next,);
      }
      return next.length;
    },
  },);
}
