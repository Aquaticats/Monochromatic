/**
 What a filling does with a queue folder after a kill: it finishes a set-aside whose record was written and whose entry never left the queue,
 drops a record a completed put-back left behind, and clears the claim of a filling that died holding one. Every decision is one atomic step
 on one file, so nothing is held and nothing waits.

 The records are settled first. An interrupted set-aside leaves a record and a still-queued entry of one identity whose claim's holder is
 gone with no output file, so a recovery that ran first would give that claim back and the settlement would then read the state as a
 put-back whose record is stale and drop the record, losing the reason its start failed and the whole of what that start printed.

 Nothing here matches on a name alone. A record holds the identity of the entry it copied, and a claim names the identity it was taken on,
 so a name queued again under a later entry is another entry and its record is left with a sentence rather than deleted.

 @module
 */

import { rm, } from 'node:fs/promises';
import {
  entryFileOf,
  NO_QUEUE_ID,
  type QueueRecord,
  claimFileOf,
  queueIdOf,
} from './queue-of-runs.ts';
import { startOfAnAttemptTookEffect, } from './claiming-an-entry-of-a-queue.ts';
import {
  NO_ENTRY_OF_THAT_NAME,
  readQueueOfRuns,
  readTheClaimsOnAQueue,
  readTheEntryNamed,
} from './reading-and-writing-the-folder-a-queue-is.ts';
import {
  NO_SET_ASIDE_RECORD,
  removeSetAsideRecord,
  setAsideRecordOf,
} from './set-aside-entry.ts';

/**
 The code the operating system gives one process's number when no process of the account has it.
 */
const NO_SUCH_PROCESS_CODE = 'ESRCH';

/**
 Says whether the process a claim names is still there.

 @param pid - the process number the claim names

 @returns whether it is there, which a process this account may not signal counts as

 @example
 ```ts
 const there = processIsThere({ pid: 1 });
 ```
 */
function processIsThere({ pid, }: { readonly pid: number },): boolean {
  try {
    process.kill(
      pid,
      0,
    );
    return true;
  }
  catch (error) {
    // A process number this account may not signal still names a live process, so only "no such process" says the holder is gone.
    return ((error as { readonly code?: unknown }).code) !== NO_SUCH_PROCESS_CODE;
  }
}

/**
 Settles each set-aside record whose name the queue holds, comparing identities rather than names.

 @param queueFolder - the queue's folder

 @param runsFolder - folder the driver writes its runs into

 @param report - where the pass says what it found, one line each

 @returns the entries the queue holds afterwards, one file at a time

 @example
 ```ts
 const left = await recordsSettled({ queueFolder: '/runs/q', runsFolder: '/runs', report: () => {} });
 ```
 */
async function recordsSettled({
  queueFolder,
  runsFolder,
  report,
}: {
  readonly queueFolder: string;
  readonly runsFolder: string;
  readonly report: (line: string) => void;
},): Promise<readonly QueueRecord[]> {
  /**
   When the lines are said.
   */
  const at = new Date().toISOString();
  /**
   Every claim the queue holds, read once, so an interrupted set-aside is told from a put-back without a read per entry.
   */
  const claims = await readTheClaimsOnAQueue({ queueFolder, },);
  /* oxlint-disable eslint/no-await-in-loop -- one entry after another, and each record is its own file */
  for (const entry of await readQueueOfRuns({ queueFolder, },)) {
    /**
     The entry's identity, which every put writes.
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
    if ((queueId === NO_QUEUE_ID) || (recordId === NO_QUEUE_ID)
      || (recordId !== queueId)) {
      // The record is another entry's only copy: it holds an entry queued under this name before, and no identity joins it to the entry
      // standing there now. Deleting it on the strength of the name is how a sweep once destroyed the only copy of such an entry.
      report(`${at} ${entry.name}: the set-aside record beside the runs holds an entry other than the one the queue holds under that name, so it is left as it is; putting it back or removing it is a person's to do\n`,);
      continue;
    }
    if (claims.some(function isOnThisAttempt(claim,) {
      return (claim.name === entry.name) && (claim.queueId === queueId);
    },)) {
      // The record's own entry is queued and claimed: the set-aside wrote the record and was killed before its entry left the queue, so
      // this finishes it. The record stays, because it is the entry's copy.
      await rm(
        entryFileOf({
          queueFolder,
          name: entry.name,
        },),
        { force: true, },
      );
      report(`${at} ${entry.name}: its set-aside wrote the record and was killed before the entry left the queue, so the set-aside is finished and the record stands\n`,);
      continue;
    }
    // The record's own entry is queued and unclaimed: a put-back wrote the entry and was killed before removing the record, which
    // leaves the record as a second copy of an entry already in the queue.
    await removeSetAsideRecord({
      runsFolder,
      name: entry.name,
    },);
    report(`${at} ${entry.name}: its set-aside record was still there after its entry was put back into the queue, so the record is dropped and the entry stands\n`,);
  }
  /* oxlint-enable eslint/no-await-in-loop */
  return await readQueueOfRuns({ queueFolder, },);
}

/**
 Clears the claim of every filling that is gone, and takes out the entry of one whose start took effect.

 @param queueFolder - the queue's folder

 @param runsFolder - folder the driver writes its runs into

 @param report - where the pass says what it found, one line each

 @returns the entries the queue holds afterwards

 @example
 ```ts
 const left = await claimsCleared({ queueFolder: '/runs/q', runsFolder: '/runs', report: () => {} });
 ```
 */
async function claimsCleared({
  queueFolder,
  runsFolder,
  report,
}: {
  readonly queueFolder: string;
  readonly runsFolder: string;
  readonly report: (line: string) => void;
},): Promise<readonly QueueRecord[]> {
  /**
   When the lines are said.
   */
  const at = new Date().toISOString();
  for (const held of await readTheClaimsOnAQueue({ queueFolder, },)) {
    if (processIsThere({ pid: held.claim
      .pid, },)) {
      continue;
    }
    /**
     The claim's file.
     */
    const claimFile = claimFileOf({
      queueFolder,
      name: held.name,
    },);
    /* oxlint-disable eslint/no-await-in-loop -- one claim after another, each with its own attempt's evidence */
    /**
     The entry the queue holds under that name now, or that it holds none.
     */
    const entry = await readTheEntryNamed({
      queueFolder,
      name: held.name,
    },);
    if ((entry === NO_ENTRY_OF_THAT_NAME) || (queueIdOf({ record: entry, },) !== held.queueId)) {
      // The entry this claim was taken on has left the queue, or the queue holds another entry of that name now: either way nothing is
      // left to start or to give back, and the claim is cleared.
      await rm(
        claimFile,
        { force: true, },
      );
      report(`${at} ${held.name}: the queue no longer holds the entry its claim names, so the claim is cleared\n`,);
      continue;
    }
    if (await startOfAnAttemptTookEffect({
      runsFolder,
      name: held.name,
      claim: held.claim,
    },)) {
      await rm(
        entryFileOf({
          queueFolder,
          name: held.name,
        },),
        { force: true, },
      );
      await rm(
        claimFile,
        { force: true, },
      );
      report(`${at} ${held.name}: the filling that claimed it died after its start took effect, so it is taken out of the queue and never started again\n`,);
      continue;
    }
    await rm(
      claimFile,
      { force: true, },
    );
    report(`${at} ${held.name}: the filling that claimed it died before its start took effect, so the claim is given back and the entry stands for the next pass\n`,);
    /* oxlint-enable eslint/no-await-in-loop */
  }
  return await readQueueOfRuns({ queueFolder, },);
}

/**
 Settles a queue folder after a kill: finishes an interrupted set-aside, drops a record a completed put-back left behind, and clears the
 claims of fillings that died holding them. Every decision is one atomic step on one file, so a pass may run beside another.

 @param queueFolder - the queue's folder

 @param runsFolder - folder the driver writes its runs into

 @param report - where the pass says what it found, one line each

 @returns how many entries the queue holds after the step

 @throws QueueFileError when the queue or one of its records cannot be read, which changes nothing

 @example
 ```ts
 const left = await settleTheQueueAfterAKill({ queueFolder: '/runs/q', runsFolder: '/runs', report: () => {} });
 ```
 */
export async function settleTheQueueAfterAKill({
  queueFolder,
  runsFolder,
  report,
}: {
  readonly queueFolder: string;
  readonly runsFolder: string;
  readonly report: (line: string) => void;
},): Promise<number> {
  /**
   The queue once every record whose name it holds is settled.
   */
  await recordsSettled({
    queueFolder,
    runsFolder,
    report,
  },);
  return (await claimsCleared({
    queueFolder,
    runsFolder,
    report,
  },)).length;
}
