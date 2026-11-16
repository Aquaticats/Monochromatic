/**
 What a filling does with a queue after a kill: it finishes a set-aside whose record was written and whose claim never completed, drops
 a record a completed put-back left behind, and recovers the claim of a filling that died holding one. One held step of the queue's lock,
 so every decision is made on the queue and on the records as they are, and never on a snapshot another writer has since changed.

 The records are settled first. An interrupted set-aside leaves a record and a claimed entry of one identity, and that claim's holder is
 gone with no output file, so a recovery that ran first would give the claim back and the settlement would then read the state as a
 put-back whose record is stale and drop the record, losing the reason its start failed and the whole of what that start printed.

 Nothing here matches on a name alone. A record holds the identity of the entry it copied, and an entry is taken out only when both its
 identity and the token of the claim that took it are the ones written down; a name may be queued again once its earlier entry is gone,
 and a decision on the name alone once deleted the only copy of an entry (reproduced on 2026-11-15, against the sweep of that day's
 queue).

 @module
 */

import { processIsThere, } from './queue-lock.ts';
import {
  NO_CLAIM_ON_AN_ENTRY,
  NO_QUEUE_ID,
  type QueueRecord,
  claimOnAnEntry,
  queueIdOf,
  withTheQueueLock,
  withoutTheClaim,
} from './queue-of-runs.ts';
import { startOfAnAttemptTookEffect, } from './claims-on-the-entries-of-a-queue.ts';
import {
  NO_SET_ASIDE_RECORD,
  removeSetAsideRecord,
  setAsideRecordOf,
} from './set-aside-entry.ts';

/**
 What one settlement decided about the queue.
 */
type SettledQueue = {
  /**
   The entries the queue holds after the decision.
   */
  readonly entries: readonly QueueRecord[];

  /**
   Whether anything was written, which is what says the queue has to be written back.
   */
  readonly changed: boolean;
};

/**
 Settles each set-aside record whose name the queue holds, comparing identities rather than names.

 @param entries - the queue as it stands under the lock

 @param runsFolder - folder the driver writes its runs into

 @param report - where the pass says what it found, one line each

 @returns the entries the queue holds afterwards, once a record is finished or dropped

 @example
 ```ts
 const settled = await recordsSettled({ entries, runsFolder: '/runs', report: () => {} });
 ```
 */
async function recordsSettled({
  entries,
  runsFolder,
  report,
}: {
  readonly entries: readonly QueueRecord[];
  readonly runsFolder: string;
  readonly report: (line: string) => void;
},): Promise<SettledQueue> {
  /**
   When the lines are said.
   */
  const at = new Date().toISOString();
  /**
   The identities of the attempts an interrupted set-aside took out of the queue, as `identity` and `token` joined.
   */
  const finished: string[] = [];
  /**
   The names whose stale record was dropped, which change no entry and still have to be said.
   */
  const dropped: string[] = [];
  /* oxlint-disable eslint/no-await-in-loop -- one entry after another, and each record is its own file */
  for (const entry of entries) {
    /**
     The entry's identity, which every claimed entry has.
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
     The identity the record's entry carries.
     */
    const recordId = queueIdOf({ record: record.entry, },);
    if (
      (queueId === NO_QUEUE_ID)
        || (recordId === NO_QUEUE_ID)
        || (recordId !== queueId)
    ) {
      // The record is another entry's only copy: it holds an entry queued under this name before, and no identity joins it to the entry
      // standing there now. Deleting it on the strength of the name is how a sweep once destroyed the only copy of such an entry.
      report(`${at} ${entry.name}: the set-aside record beside the runs holds an entry other than the one the queue holds under that name, so it is left as it is; putting it back or removing it is a person's to do\n`,);
      continue;
    }
    /**
     The claim the queued entry carries, when a filling holds one.
     */
    const claim = claimOnAnEntry({ record: entry, },);
    if (claim === NO_CLAIM_ON_AN_ENTRY) {
      // The record's own entry is queued and unclaimed: a put-back wrote the entry and was killed before removing the record, which
      // leaves the record as a second copy of an entry already in the queue.
      await removeSetAsideRecord({
        runsFolder,
        name: entry.name,
      },);
      dropped.push(entry.name,);
      report(`${at} ${entry.name}: its set-aside record was still there after its entry was put back into the queue, so the record is dropped and the entry stands\n`,);
      continue;
    }
    // The record's own entry is queued and claimed: the set-aside wrote the record and was killed before the claim completed, so this
    // finishes it. The token is carried so that an entry queued under that name with another identity is never taken out here.
    finished.push(`${queueId}|${claim.token}`,);
    report(`${at} ${entry.name}: its set-aside wrote the record and was killed before the entry left the queue, so the set-aside is finished and the record stands\n`,);
  }
  /* oxlint-enable eslint/no-await-in-loop */
  if ((finished.length === 0) && (dropped.length === 0)) {
    return {
      entries,
      changed: false,
    };
  }
  return {
    entries: entries.filter(function isNotAFinishedSetAside(entry,) {
      /**
       The entry's identity, when a claim wrote one.
       */
      const queueId = queueIdOf({ record: entry, },);
      if (queueId === NO_QUEUE_ID) {
        return true;
      }
      /**
       The claim the entry carries, when a filling holds one.
       */
      const claim = claimOnAnEntry({ record: entry, },);
      if (claim === NO_CLAIM_ON_AN_ENTRY) {
        return true;
      }
      return !finished.includes(`${queueId}|${claim.token}`,);
    },),
    changed: true,
  };
}

/**
 Recovers the claim of every filling that died holding one, for each entry whose claim names a process that is gone.

 @param entries - the queue as the records left it

 @param runsFolder - folder the driver writes its runs into

 @param report - where the pass says what it found, one line each

 @returns the entries the queue holds afterwards, with a claim given back or an attempt taken out

 @example
 ```ts
 const recovered = await claimsRecovered({ entries, runsFolder: '/runs', report: () => {} });
 ```
 */
async function claimsRecovered({
  entries,
  runsFolder,
  report,
}: {
  readonly entries: readonly QueueRecord[];
  readonly runsFolder: string;
  readonly report: (line: string) => void;
},): Promise<SettledQueue> {
  /**
   When the lines are said.
   */
  const at = new Date().toISOString();
  /**
   The attempts whose start took effect before their filling died, which are out of the queue and never started again.
   */
  const started: string[] = [];
  /**
   The attempts whose filling died before their start took effect, whose claims are given back.
   */
  const givenBack: string[] = [];
  /* oxlint-disable eslint/no-await-in-loop -- one entry after another, and each attempt's evidence is its own file */
  for (const entry of entries) {
    /**
     The claim the entry carries, when a filling holds one.
     */
    const claim = claimOnAnEntry({ record: entry, },);
    if (claim === NO_CLAIM_ON_AN_ENTRY) {
      continue;
    }
    if (processIsThere({ pid: claim.pid, },)) {
      continue;
    }
    /**
     The entry's identity, which a claim always wrote.
     */
    const queueId = queueIdOf({ record: entry, },);
    if (queueId === NO_QUEUE_ID) {
      continue;
    }
    /**
     The attempt, named by its identity and its token.
     */
    const attempt = `${queueId}|${claim.token}`;
    if (await startOfAnAttemptTookEffect({
      runsFolder,
      name: entry.name,
      claim,
    },)) {
      started.push(attempt,);
      report(`${at} ${entry.name}: the filling that claimed it died after its start took effect, so it is taken out of the queue and never started again\n`,);
    }
    else {
      givenBack.push(attempt,);
      report(`${at} ${entry.name}: the filling that claimed it died before its start took effect, so the claim is given back and the entry stands for the next pass\n`,);
    }
  }
  /* oxlint-enable eslint/no-await-in-loop */
  if ((started.length === 0) && (givenBack.length === 0)) {
    return {
      entries,
      changed: false,
    };
  }
  /**
   The entries that are not attempts whose start took effect.
   */
  const kept = entries.filter(function isNotAnAttemptThatStarted(entry,) {
      /**
       The entry's identity, when a claim wrote one.
       */
    const queueId = queueIdOf({ record: entry, },);
    if (queueId === NO_QUEUE_ID) {
      return true;
    }
      /**
       The claim the entry carries, when a filling holds one.
       */
    const claim = claimOnAnEntry({ record: entry, },);
    if (claim === NO_CLAIM_ON_AN_ENTRY) {
      return true;
    }
    return !started.includes(`${queueId}|${claim.token}`,);
  },);
  return {
    entries: kept.map(function withoutTheClaimGivenBack(entry,) {
      /**
       The entry's identity, when a claim wrote one.
       */
      const queueId = queueIdOf({ record: entry, },);
      if (queueId === NO_QUEUE_ID) {
        return entry;
      }
      /**
       The claim the entry carries, when a filling holds one.
       */
      const claim = claimOnAnEntry({ record: entry, },);
      if (claim === NO_CLAIM_ON_AN_ENTRY) {
        return entry;
      }
      return givenBack.includes(`${queueId}|${claim.token}`,)
        ? withoutTheClaim({ record: entry, },)
        : entry;
    },),
    changed: true,
  };
}

/**
 Settles a queue after a kill: finishes an interrupted set-aside, drops a record a completed put-back left behind, and recovers the claims
 of fillings that died holding them. One held step, so every decision is made on the queue and the records as they are.

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
    work: async function settle({
      entries,
      write,
    },) {
      /**
       The queue once every record whose name it holds is settled.
       */
      const settledRecords = await recordsSettled({
        entries,
        runsFolder,
        report,
      },);
      /**
       The queue once the claims of fillings that are gone are recovered.
       */
      const settledClaims = await claimsRecovered({
        entries: settledRecords.entries,
        runsFolder,
        report,
      },);
      if (settledRecords.changed || settledClaims.changed) {
        await write(settledClaims.entries,);
      }
      return settledClaims.entries
        .length;
    },
  },);
}
