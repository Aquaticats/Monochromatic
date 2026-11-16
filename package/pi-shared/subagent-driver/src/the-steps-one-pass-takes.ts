/**
 The steps one pass of filling takes on one entry once it is claimed: starting it, giving its claim back when the start threw, and setting
 it aside when its own start failed.

 Each answers what it did, and each gives the claim back when the step after the claim throws, so a filling that is still alive never leaves a
 claim that no later pass may take from it.

 @module
 */

import { rm, } from 'node:fs/promises';
import {
  type ClaimedAnEntry,
  claimAnEntryOfAQueue,
  giveBackTheClaim,
  namesOfAnAttemptIn,
} from './claiming-an-entry-of-a-queue.ts';
import type {
  QueueClaim,
  QueueRecord,
} from './queue-of-runs.ts';
import type {
  FailureOfAStart,
  StartOfAnEntry,
} from './starting-an-entry-of-a-queue.ts';

/**
 Nothing claimed by this pass, as a symbol so no claim can be mistaken for none.
 */
export const NOTHING_TO_CLAIM: unique symbol = Symbol('every entry that may start now belongs to another filling',);

/**
 The claim this pass holds on one entry, with the entry as the queue held it.
 */
export type AClaimedEntry = {
  /**
   The entry as this pass read it.
   */
  readonly record: QueueRecord;

  /**
   The entry's own identity, minted by the claim when it had none.
   */
  readonly queueId: string;

  /**
   The claim this pass holds, whose token names the attempt.
   */
  readonly claim: QueueClaim;
};

/**
 Claims the first entry of the queue that may start now and that no other filling holds, so two passes never start one entry.

 @param queueFolder - the queue's folder

 @param candidates - every entry of the queue that may start now, in the queue's order

 @param say - where the pass says which entry another filling already holds

 @returns the claim this pass holds, or that every candidate belongs to another filling

 @example
 ```ts
 const mine = await claimOneOf({ queueFolder: '/runs/q', candidates, say });
 ```
 */
export async function claimOneOf({
  queueFolder,
  candidates,
  say,
}: {
  readonly queueFolder: string;
  readonly candidates: readonly QueueRecord[];
  readonly say: (line: string) => void;
},): Promise<AClaimedEntry | typeof NOTHING_TO_CLAIM> {
  /* oxlint-disable eslint/no-await-in-loop -- each candidate is its own step on its own entry */
  for (const candidate of candidates) {
    /**
     What taking a claim on this candidate answered, or why this candidate could not be claimed at all.
     */
    try {
      /**
       What taking a claim on this candidate answered.
       */
      const taken: ClaimedAnEntry = await claimAnEntryOfAQueue({
        queueFolder,
        record: candidate,
        now: Date.now,
      },);
      if (taken.kind === 'claimed') {
        return {
          record: candidate,
          queueId: taken.queueId,
          claim: taken.claim,
        };
      }
      // Another filling claimed it, or it left the queue, between this pass's read and its claim: the entry behind it is asked about
      // rather than the pass stopping, because an entry another filling is starting is being started.
      say(`${new Date().toISOString()} ${candidate.name}: ${taken.kind === 'claimed-by-another' ? 'another filling claimed it first' : 'it left the queue before this pass claimed it'}, so this pass asks about the entry behind it\n`,);
    }
    catch (error) {
      // A candidate this filling cannot claim is one bad entry, and one bad entry must never stop the queue behind it: it is said by
      // name, with what refused it, and the pass asks about the entry after it.
      say(`${new Date().toISOString()} ${JSON.stringify(candidate.name,)} could not be claimed and is passed over: ${String(error,)}\n`,);
    }
  }
  /* oxlint-enable eslint/no-await-in-loop */
  return NOTHING_TO_CLAIM;
}

/**
 Starts one claimed entry, and gives its claim back when the start threw rather than answering.

 A claim whose holder is alive is never recovered by another filling, so a claim left behind here would keep its entry where no pass
 starts it; giving it back is what a kill would leave too.

 @param queueFolder - the queue's folder

 @param mine - the claim this pass holds, with the entry as it read it

 @param record - the entry as the queue holds it, without the claim

 @param start - how one entry is started

 @returns what starting the entry answered

 @throws the error the start threw, after the claim is given back

 @example
 ```ts
 const outcome = await startedWithTheClaimGivenBack({ queueFolder, mine, record, start: seams.start });
 ```
 */
export async function startedWithTheClaimGivenBack({
  queueFolder,
  mine,
  record,
  start,
}: {
  readonly queueFolder: string;
  readonly mine: AClaimedEntry;
  readonly record: QueueRecord;
  readonly start: (request: {
    readonly record: QueueRecord;
    readonly claim: QueueClaim;
  }) => Promise<StartOfAnEntry>;
},): Promise<StartOfAnEntry> {
  try {
    return await start({
      record,
      claim: mine.claim,
    },);
  }
  catch (error) {
    // The claim is given back only while it is still this attempt's, so a claim another filling took meanwhile is left alone.
    await giveBackTheClaim({
      queueFolder,
      name: record.name,
      token: mine.claim
        .token,
    },);
    throw error;
  }
}

/**
 Removes the batch list and the output file of one attempt whose unit was never made.

 @param runsFolder - folder the driver writes its runs into

 @param name - the entry's name

 @param claim - the claim naming the attempt

 @example
 ```ts
 await removeAnAttemptsList({ runsFolder: '/runs', name: 'a', claim });
 ```
 */
export async function removeAnAttemptsList({
  runsFolder,
  name,
  claim,
}: {
  readonly runsFolder: string;
  readonly name: string;
  readonly claim: QueueClaim;
},): Promise<void> {
  /**
   The names the attempt would have written.
   */
  const names = namesOfAnAttemptIn({
    runsFolder,
    name,
    claim,
  },);
  await rm(
    names.batchFile,
    { force: true, },
  );
  await rm(
    names.outputFile,
    { force: true, },
  );
}

/**
 Sets one entry aside, and gives its claim back when the set-aside itself threw, so a filling's own number never keeps an entry.

 A claim whose holder is alive is never taken from it by a later pass, so a claim left behind by a set-aside that failed would keep its
 entry where no pass starts it for as long as the filling lives.

 @param queueFolder - the queue's folder

 @param record - the entry the start failed for

 @param failure - why the start failed

 @param queueId - the identity of the entry the claim was taken on

 @param token - the token naming this pass's attempt

 @param setAside - how one entry is set aside

 @returns where the record was written

 @throws the error the set-aside threw, after the claim is given back

 @example
 ```ts
 const kept = await setAsideWithTheClaimGivenBack({ queueFolder, record, failure, queueId, token, setAside: seams.setAside });
 ```
 */
export async function setAsideWithTheClaimGivenBack({
  queueFolder,
  record,
  failure,
  queueId,
  token,
  setAside,
}: {
  readonly queueFolder: string;
  readonly record: QueueRecord;
  readonly failure: FailureOfAStart;
  readonly queueId: string;
  readonly token: string;
  readonly setAside: (request: {
    readonly record: QueueRecord;
    readonly failure: FailureOfAStart;
    readonly queueId: string;
  }) => Promise<{
    readonly file: string;
    readonly left: number;
  }>;
},): Promise<{
  readonly file: string;
  readonly left: number;
}> {
  try {
    return await setAside({
      record,
      failure,
      queueId,
    },);
  }
  catch (error) {
    await giveBackTheClaim({
      queueFolder,
      name: record.name,
      token,
    },);
    throw error;
  }
}
