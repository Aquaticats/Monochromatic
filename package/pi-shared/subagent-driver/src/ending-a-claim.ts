/**
 The two ways a claim on an entry of a queue ends: completed, when the filling that holds it started the entry, or given back, when the
 machine refused the start and the entry keeps its place.

 Both are one write under the queue's lock, matched on the entry's own identity and on the claim's token, so neither acts on an entry
 queued under the same name after the one the attempt was claimed on: a removal matched on a name alone once deleted a replacement a
 person had just queued (reproduced on 2026-11-15, against the filling of that day).

 @module
 */

import {
  type QueueRecord,
  changeTheQueueOfRuns,
  entryIsTheAttempt,
  withoutTheClaim,
} from './queue-of-runs.ts';

/**
 Completes one claim by taking its entry out of the queue, matched on the entry's identity and on the claim's token.

 This is the step that makes a started entry started: a replacement queued under the same name is another entry with another identity, and
 is left where it is.

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
      const held = entries.some(function isTheAttempt(entry,) {
        return entryIsTheAttempt({
          record: entry,
          queueId,
          token,
        },);
      },);
      /**
       The entries with this attempt's claim gone, and no other entry touched.
       */
      const next = entries.map(function withoutThisClaim(entry,) {
        return entryIsTheAttempt({
          record: entry,
          queueId,
          token,
        },)
          ? withoutTheClaim({ record: entry, },)
          : entry;
      },);
      return {
        entries: next,
        answer: {
          givenBack: held,
          left: next.length,
        },
      };
    },
  },);
}
