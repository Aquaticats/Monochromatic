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

import {
  readdir,
  rm,
  stat,
} from 'node:fs/promises';
import { join, } from 'node:path';
import { MILLISECONDS_PER_SECOND, } from './driver-constants.ts';
import {
  aClaimRemovedOnlyIfItSays,
  anEntryRemovedOnlyIfTheAttempt,
} from './a-claim-taken-and-given-back.ts';
import { theHolderOfAClaim, } from './where-a-claims-holder-is-now.ts';
import {
  type ANameInAFolder,
  BEING_TAKEN_START,
  BEING_WRITTEN_START,
  claimedFolderOf,
  NO_QUEUE_ID,
  QueueFileError,
  type QueueRecord,
  queueIdOf,
} from './queue-of-runs.ts';
import { startOfAnAttemptTookEffect, } from './claiming-an-entry-of-a-queue.ts';
import { readTheClaimsOnAQueue, } from './reading-the-claims-beside-a-queue.ts';
import { readTheQueueReportingProblems, } from './reading-a-queue-a-kill-left.ts';
import {
  NO_ENTRY_OF_THAT_NAME,
  readTheEntryNamed,
} from './reading-and-writing-the-folder-a-queue-is.ts';
import {
  NO_SET_ASIDE_RECORD,
  removeSetAsideRecord,
  setAsideRecordOf,
} from './set-aside-entry.ts';

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
  for (const entry of await readTheQueueReportingProblems({
    queueFolder,
    report,
  },)) {
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
    /**
     The claim on this very entry, when a filling holds one.
     */
    const claimed = claims.find(function isOnThisAttempt(claim,) {
      return (claim.name === entry.name) && (claim.queueId === queueId);
    },);
    if (claimed !== undefined) {
      // The record's own entry is queued and claimed: the set-aside wrote the record and was killed before its entry left the queue, so
      // this finishes it. A claim a live filling holds is never finished here: that filling is starting the entry, and taking it out from
      // under that start would leave the entry's record saying its start failed while the entry runs.
      /**
       What the claim's holder is now.
       */
      const holder = await theHolderOfAClaim({ claim: claimed, },);
      if (holder.kind !== 'gone') {
        report(`${at} ${entry.name}: its set-aside wrote the record and left the entry queued, and the claim on it is ${('reason' in holder) ? holder.reason : 'a holder that may still be starting it'}, so the record and the entry are left for that holder and for a person\n`,);
        continue;
      }
      // The entry goes before its claim: a kill between the two would otherwise leave a started entry queued with nothing naming it.
      await anEntryRemovedOnlyIfTheAttempt({
        queueFolder,
        name: entry.name,
        queueId,
      },);
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
  return await readTheQueueReportingProblems({
    queueFolder,
    report,
  },);
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
  /* oxlint-disable eslint/no-await-in-loop -- one claim after another, each read and decided on its own, which is what makes each
     decision one step on one file. See doc/lint/sequential-awaits.md. */
  for (const held of await readTheClaimsOnAQueue({ queueFolder, },)) {
    /**
     What the process the claim names says about the holder: that it is there, that it has gone, or that its number now belongs to a
     process that started at another moment.
     */
    const holder = await theHolderOfAClaim({ claim: held, },);
    if (holder.kind === 'there') {
      continue;
    }
    if (holder.kind === 'unreadable') {
      // A holder this caller cannot judge is one it must not take a claim from: the entry waits, and the line says what stopped it.
      report(`${at} ${held.name}: the filling that claimed it cannot be judged gone, so its claim stands: ${holder.reason}\n`,);
      continue;
    }
    /**
     The entry the queue holds under that name now, or that it holds none.
     */
    const entry = await readTheEntryNamed({
      queueFolder,
      name: held.name,
    },);
    if ((entry === NO_ENTRY_OF_THAT_NAME) || (queueIdOf({ record: entry, },) !== held.queueId)) {
      // The entry this claim was taken on has left the queue, or the queue holds another entry of that name now: either way nothing is
      // left to start, so only the claim is cleared.
      await aClaimRemovedOnlyIfItSays({
        queueFolder,
        name: held.name,
        holds: function isTheClaimRead(claim,) {
          return (claim.claim
            .token
            === held.claim
              .token) && (claim.queueId === held.queueId);
        },
      },);
      report(`${at} ${held.name}: the queue no longer holds the entry its claim named, so the claim is cleared\n`,);
      continue;
    }
    /**
     Whether this attempt's own unit was made, which is what tells an entry that started from one that never did.
     */
    const started = await startOfAnAttemptTookEffect({
      runsFolder,
      name: held.name,
      claim: held.claim,
    },);
    // The evidence is read before anything is removed, and then the entry goes before its claim: a kill between the two would otherwise
    // leave a started entry queued with nothing left naming its attempt, which the next pass would start a second time. A claim whose
    // entry is gone is cleared by the next pass anyway, so removing it last costs nothing.
    if (started) {
      await anEntryRemovedOnlyIfTheAttempt({
        queueFolder,
        name: held.name,
        queueId: held.queueId,
      },);
      await aClaimRemovedOnlyIfItSays({
        queueFolder,
        name: held.name,
        holds: function isStillThisAttempt(claim,) {
          return claim.claim
            .token
            === held.claim
              .token;
        },
      },);
      report(`${at} ${held.name}: the filling that claimed it died after its start took effect, so it is taken out of the queue and never started again\n`,);
      continue;
    }
    /**
     Whether the claim this pass read was still there to give back, or another filling's claim stood under the name.
     */
    const givenBack = await aClaimRemovedOnlyIfItSays({
      queueFolder,
      name: held.name,
      holds: function isTheClaimRead(claim,) {
        return (claim.claim
          .token
          === held.claim
            .token) && (claim.queueId === held.queueId);
      },
    },);
    if (!givenBack) {
      report(`${at} ${held.name}: the filling that held the claim this pass read is gone, but another filling claimed it meanwhile, so that claim stands\n`,);
      continue;
    }
    report(`${at} ${held.name}: the filling that claimed it died before its start took effect, so the claim is given back and the entry stands for the next pass\n`,);
  }
  /* oxlint-enable eslint/no-await-in-loop */
  return await readTheQueueReportingProblems({
    queueFolder,
    report,
  },);
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
  /**
   The entries the queue holds once the claims of fillings that are gone are cleared.
   */
  const left = await claimsCleared({
    queueFolder,
    runsFolder,
    report,
  },);
  await halfWrittenFilesSwept({
    queueFolder,
    report,
  },);
  return left.length;
}

/**
 How old a file a command was writing or taking aside must be before a pass removes it.

 A step on one file is milliseconds, and the file written beside a name lives for that step alone (measured: a put is about 2 ms on a queue
 of seventy entries). An hour is far past any step, so a file that old belongs to a command that will never finish it, while one younger may
 belong to a command that is still running and is left alone.
 */
export const HALF_WRITTEN_FILE_AGE_MILLISECONDS = 3_600_000;

/**
 Removes the files a killed command left on its way to a name, in the queue's folder and in the folder its claims live in.

 Only a file written on its way to a name is removed, and only once it is far too old to belong to a step still running: such a file holds
 half of a write and is never an entry.

 A file a take renamed an entry to is not one of those: it holds the entry whole, and it is the only copy of it until the person's command
 discards it. It is never removed here however old it is: the pass says where it is, so a person finds the entry, which is what the page
 promises. Removing it for its age is how a killed take lost the entry it had set aside.

 @param queueFolder - the queue's folder

 @param report - where the pass says what it removed, one line each

 @param now - the clock the age of a file is read from

 @returns how many files it removed

 @example
 ```ts
 const removed = await halfWrittenFilesSwept({ queueFolder: '/runs/q', report: () => {} });
 ```
 */
export async function halfWrittenFilesSwept({
  queueFolder,
  report,
  now = Date.now,
}: {
  readonly queueFolder: string;
  readonly report: (line: string) => void;
  readonly now?: () => number;
},): Promise<number> {
  /**
   How many files were removed.
   */
  let removed = 0;
  /* oxlint-disable eslint/no-await-in-loop -- one folder after another, and one file after another inside it, each read for its own
     reason. See doc/lint/sequential-awaits.md. */
  for (const folder of [
    queueFolder,
    claimedFolderOf({ queueFolder, },),
  ]) {
    /**
     What the folder holds, or nothing when it is not there.
     */
    let held: readonly ANameInAFolder[] = [];
    try {
      held = await readdir(
        folder,
        { withFileTypes: true, },
      );
    }
    catch (error) {
      // A claims folder nothing has claimed in is not there, which is not a failure to sweep.
      if (String(error,)
        .includes('ENOENT',)) {
        continue;
      }
      throw new QueueFileError(`the folder ${folder} cannot be read to sweep what a kill left: ${String(error,)}`,);
    }
    for (const name of held) {
      if (!name.isFile()) {
        continue;
      }
      if (name.name
        .startsWith(BEING_TAKEN_START,)) {
        // An entry a take renamed aside, which is the only copy of it: it is named for a person and never removed.
        report(`${new Date(now(),).toISOString()} ${join(
          folder,
          name.name,
        )} holds an entry a take renamed aside and never finished; the entry is there and a person can put it back\n`,);
        continue;
      }
      if (!name.name
        .startsWith(BEING_WRITTEN_START,)) {
        continue;
      }
      /**
       The file's path.
       */
      const path = join(
        folder,
        name.name,
      );
      /**
       How long ago the file was last written.
       */
      const age = now() - (await stat(path,)).mtimeMs;
      if (age < HALF_WRITTEN_FILE_AGE_MILLISECONDS) {
        continue;
      }
      await rm(
        path,
        { force: true, },
      );
      removed += 1;
      report(`${new Date(now(),).toISOString()} ${path} was written ${String(Math.round(age / MILLISECONDS_PER_SECOND,),)} s ago by a command that never finished, so it is removed\n`,);
    }
  }
  /* oxlint-enable eslint/no-await-in-loop */
  return removed;
}
