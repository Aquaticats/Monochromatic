/**
 Controls that every decision this package acts on is made inside one held step of the queue's lock, across the two files a decision
 can touch: the queue and an entry's set-aside record.

 The review of 2026-11-14 found the same shape one level above the queue's own change: a decision made on one read and acted on by a
 second change, across the two files or across two instants. A put-back deleted the record of a set-aside a filling was still
 committing, so the entry was neither queued nor set aside; a filling's sweep decided on a snapshot and deleted a record another
 writer had just committed; and a put-back review took the place of an integration pass queued at the head meanwhile.

 Each test here fails against the code before the fix and passes after it. The two lock-held controls are in this file's second half
 because they need the lock's own wait bound; the others need no seam at all.

 Every queue file, record and fixture lives in a throwaway folder.

 @module
 */

import {
  mkdir,
  mkdtemp,
  rm,
  writeFile,
} from 'node:fs/promises';
import { join, } from 'node:path';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';
import {
  acquireTheQueueLock,
  BUILD_VOLUME,
  fillFreePlacesFromAQueue,
  type FillingSeams,
  type FreeSpace,
  putAQueueEntryIn,
  putASetAsideEntryBack,
  putEndedReviewsBack,
  QueueLockHeldError,
  readQueueOfRuns,
  setAsideAnEntryOfAQueue,
  setAsideFileOf,
  setAsideRecordIsThere,
  writeQueueOfRuns,
  writeSetAsideRecord,
} from '../dist/final/node/index.mjs';

/**
 A folder a test may write in, with its queue, its runs and its reviews.
 */
type AThrowawayFolder = {
  /**
   The folder's path.
   */
  readonly path: string;

  /**
   The folder the runs live in.
   */
  readonly runs: string;

  /**
   The folder one folder per review goes in.
   */
  readonly reviews: string;

  /**
   The queue's path.
   */
  readonly queueFile: string;

  /**
   Removes the folder and everything under it.
   */
  readonly [Symbol.asyncDispose]: () => Promise<void>;
};

/**
 A folder a test may write in, holding a queue with the entries given.

 @param entries - what the queue holds

 @returns the folder
 */
async function makeFolder(
  entries: readonly (Record<string, unknown> & { readonly name: string })[],
): Promise<AThrowawayFolder> {
  /**
   The folder's path.
   */
  const path = await mkdtemp(join(scratchRoot(), 'pi-every-decision-',),);
  /**
   The folder the runs live in.
   */
  const runs = join(path, 'runs',);
  /**
   The folder one folder per review goes in.
   */
  const reviews = join(path, 'reviews',);
  await mkdir(runs, { recursive: true, },);
  await mkdir(reviews, { recursive: true, },);
  /**
   The queue's path.
   */
  const queueFile = join(runs, 'queue.json',);
  if (entries.length > 0) {
    await writeQueueOfRuns({
      queueFile,
      entries,
    },);
  }
  return {
    path,
    runs,
    reviews,
    queueFile,
    async [Symbol.asyncDispose]() {
      await rm(path, { recursive: true, force: true, },);
    },
  };
}

/**
 The names a queue holds, in its order.

 @param queueFile - the queue's path

 @returns the names
 */
async function namesIn(queueFile: string,): Promise<readonly string[]> {
  return (await readQueueOfRuns({ queueFile, },)).map(function nameOf(entry,) {
    return entry.name;
  },);
}

/**
 One entry of a queue that the tests here set aside.
 */
const THE_ENTRY = {
  name: 'a',
  branch: 'feat/a',
  prompt: 'the user-defined run',
};

/**
 Writes the run folder and summary of a review the provider's rate limit ended.

 @param folder - the folder whose runs and reviews hold it

 @param review - the review's name, as `review-<change>`

 @returns the run's folder name
 */
async function writeAReviewTheRateLimitEnded({
  folder,
  review,
}: {
  readonly folder: AThrowawayFolder;
  readonly review: string;
},): Promise<string> {
  /**
   The run's folder name.
   */
  const runFolder = `${review}-2`;
  await mkdir(
    join(
      folder.runs,
      runFolder,
    ),
    { recursive: true, },
  );
  await writeFile(
    join(
      folder.runs,
      runFolder,
      'summary.json',
    ),
    `${JSON.stringify({
      endReason: 'model-error',
      lastAssistantText: '429 {"error":{"code":"insufficient_quota","message":"You exceeded your current quota"}}',
    },)}\n`,
  );
  return runFolder;
}

await describe({
  name: '',
  children: [
    describe({
      name: 'a put-back over the two files',
      children: [
        it({
          name: 'leaves the record of an entry that is still queued alone, because deleting it loses an entry in flight',
          fn: async () => {
            await using folder = await makeFolder([
              THE_ENTRY,
              { name: 'b', branch: 'feat/b', },
            ],);
            // The state a kill between a record and its entry leaving the queue leaves: the entry is queued and its record says
            // why the start failed. Nothing else can produce it once both writes are one held step.
            await writeSetAsideRecord({
              runsFolder: folder.runs,
              record: {
                name: THE_ENTRY.name,
                entry: THE_ENTRY,
                reason: 'the entry failed its start',
                setAsideAt: '2026-11-14T09:00:00.000-05:00',
              },
            },);
            /**
             What putting the entry back answered.
             */
            const put = await putASetAsideEntryBack({
              queueFile: folder.queueFile,
              runsFolder: folder.runs,
              name: THE_ENTRY.name,
              position: 0,
            },);
            expect(put.line,).toContain('is still in the queue',);
            // Before the fix this branch deleted the record and the entry then left the queue: neither queued nor set aside.
            expect(await setAsideRecordIsThere({
              runsFolder: folder.runs,
              name: THE_ENTRY.name,
            },),).toBe(true,);
            expect(await namesIn(folder.queueFile,),).toEqual(['a', 'b',],);
          },
        },),
      ],
    },),
    describe({
      name: 'where a put-back review goes',
      children: [
        it({
          name: 'never stands ahead of an integration pass queued at the head while the review is put back',
          fn: async () => {
            await using folder = await makeFolder([{ name: 'integration-pass-1', branch: 'feat/p1', },],);
            /**
             The review run the rate limit ended.
             */
            const runFolder = await writeAReviewTheRateLimitEnded({
              folder,
              review: 'review-x',
            },);
            /**
             The rounds this control runs: the review put back and a second pass queued at the head at the same time.
             */
            const ROUNDS = 60;
            /**
             The orders in which a review stood ahead of a pass.
             */
            const reviewsAhead: string[][] = [];
            /* oxlint-disable eslint/no-await-in-loop -- one round after another, each with two writers racing, which is what this
               control measures. See doc/lint/sequential-awaits.md. */
            for (let round = 0; round < ROUNDS; round += 1) {
              await writeQueueOfRuns({
                queueFile: folder.queueFile,
                entries: [{ name: 'integration-pass-1', branch: 'feat/p1', },],
              },);
              await Promise.all([
                putEndedReviewsBack({
                  queueFile: folder.queueFile,
                  runsFolder: folder.runs,
                  reviewsFolder: folder.reviews,
                  since: 0,
                  going: new Set(),
                  now: Date.now,
                },),
                putAQueueEntryIn({
                  queueFile: folder.queueFile,
                  entry: { name: `integration-pass-2-round-${String(round,)}`, branch: 'feat/p2', },
                  position: 0,
                },),
              ],);
              /**
               This round's order.
               */
              const names = await namesIn(folder.queueFile,);
              /**
               Where the review stands.
               */
              const reviewAt = names.findIndex(function isTheReview(name,) {
                return name.startsWith('review-',);
              },);
              // Any pass standing after the review is a violation, not only the first one: an earlier version of the review's own
              // script compared against the first pass alone and under-counted.
              if ((reviewAt !== (-1)) && names.some(function isAPassAfterIt(name, at,) {
                return (at > reviewAt) && name.startsWith('integration-pass-',);
              },)) {
                reviewsAhead.push([...names,],);
              }
              expect(names,).toContain('review-x',);
            }
            // Before the fix the position came from a read taken before the insert and the second pass jumped past the review in
            // most rounds; the review's own script measured 250 of 250.
            expect(reviewsAhead,).toEqual([],);
          },
        },),
      ],
    },),
    describe({
      name: 'an operation that would decide on the queue while another writer holds it',
      children: [
        it({
          name: 'writes no set-aside record and takes no entry out, so a record cannot appear while the queue is another writer\'s',
          fn: async () => {
            await using folder = await makeFolder([
              THE_ENTRY,
              { name: 'b', branch: 'feat/b', },
            ],);
            await using lock = await acquireTheQueueLock({ queueFile: folder.queueFile, },);
            /**
             What setting the entry aside threw.
             */
            let caught: unknown;
            try {
              await setAsideAnEntryOfAQueue({
                queueFile: folder.queueFile,
                runsFolder: folder.runs,
                record: THE_ENTRY,
                reason: 'the entry failed its start',
                now: Date.now,
                waitMilliseconds: 30,
              },);
            }
            catch (error) {
              caught = error;
            }
            expect(caught,).toBeInstanceOf(QueueLockHeldError,);
            // Before the fix the record was written here although another writer held the queue, which is the state a put-back could
            // then delete while the entry was in flight.
            expect(await setAsideRecordIsThere({
              runsFolder: folder.runs,
              name: THE_ENTRY.name,
            },),).toBe(false,);
            expect(await namesIn(folder.queueFile,),).toEqual(['a', 'b',],);
          },
        },),
        it({
          name: 'drops no record in a filling pass that cannot take the queue, because the sweep decides inside the held step',
          fn: async () => {
            await using folder = await makeFolder([
              THE_ENTRY,
              { name: 'b', branch: 'feat/b', },
            ],);
            await writeSetAsideRecord({
              runsFolder: folder.runs,
              record: {
                name: THE_ENTRY.name,
                entry: THE_ENTRY,
                reason: 'the entry failed its start',
                setAsideAt: '2026-11-14T09:00:00.000-05:00',
              },
            },);
            await using lock = await acquireTheQueueLock({ queueFile: folder.queueFile, },);
            /**
             The room the build volume is said to have, so the pass reaches its sweep and nothing else.
             */
            const room: FreeSpace = {
              volume: BUILD_VOLUME,
              freeBytes: 1_000 * 1_024 * 1_024 * 1_024,
            };
            /**
             The pass's seams, none of which may be asked anything: the sweep comes first and the queue is another writer's.
             */
            const seams: FillingSeams = {
              runsGoing: function runsGoing() {
                return Promise.reject(new Error('the pass reached the count of runs, which the sweep should have stopped',));
              },
              freeSpace: function freeSpace() {
                return room;
              },
              start: function start() {
                return Promise.reject(new Error('the pass reached a start, which the sweep should have stopped',));
              },
              setAside: function setAside() {
                return Promise.reject(new Error('the pass reached a set-aside, which the sweep should have stopped',));
              },
              wait: function wait() {
                return Promise.resolve();
              },
              report: function report() {
                return undefined;
              },
            };
            /**
             What the pass threw.
             */
            let caught: unknown;
            try {
              await fillFreePlacesFromAQueue({
                request: {
                  queueFile: folder.queueFile,
                  runsFolder: folder.runs,
                  level: 1,
                  lockWaitMilliseconds: 30,
                },
                seams,
              },);
            }
            catch (error) {
              caught = error;
            }
            expect(caught,).toBeInstanceOf(QueueLockHeldError,);
            // Before the fix the sweep read the queue itself and dropped this record while another writer held the queue.
            expect(await setAsideRecordIsThere({
              runsFolder: folder.runs,
              name: THE_ENTRY.name,
            },),).toBe(true,);
          },
        },),
        it({
          name: 'puts no review back when the queue is another writer\'s, so no position is decided on a queue it does not hold',
          fn: async () => {
            await using folder = await makeFolder([{ name: 'integration-pass-1', branch: 'feat/p1', },],);
            await writeAReviewTheRateLimitEnded({
              folder,
              review: 'review-x',
            },);
            await using lock = await acquireTheQueueLock({ queueFile: folder.queueFile, },);
            /**
             What putting the ended reviews back threw.
             */
            let caught: unknown;
            try {
              await putEndedReviewsBack({
                queueFile: folder.queueFile,
                runsFolder: folder.runs,
                reviewsFolder: folder.reviews,
                since: 0,
                going: new Set(),
                now: Date.now,
                waitMilliseconds: 30,
              },);
            }
            catch (error) {
              caught = error;
            }
            expect(caught,).toBeInstanceOf(QueueLockHeldError,);
            expect(await namesIn(folder.queueFile,),).toEqual(['integration-pass-1',],);
          },
        },),
      ],
    },),
    describe({
      name: 'a queue file and a record in the shape they have',
      children: [
        it({
          name: 'are read and acted on as they are, with no shape carried over and nothing lost',
          fn: async () => {
            await using folder = await makeFolder([],);
            // Today's queue file, written by hand, and today's record file: both shapes are unchanged by the lock and by this fix.
            await writeFile(
              folder.queueFile,
              '[\n  {"name":"a","branch":"feat/a"},\n  {"name":"b","branch":"feat/b"}\n]\n',
            );
            await writeFile(
              setAsideFileOf({
                runsFolder: folder.runs,
                name: 'c',
              },),
              `${JSON.stringify({
                name: 'c',
                entry: { name: 'c', branch: 'feat/c', },
                reason: 'the entry failed its start',
                setAsideAt: '2026-11-14T09:00:00.000-05:00',
              },)}\n`,
            );
            expect(await namesIn(folder.queueFile,),).toEqual(['a', 'b',],);
            /**
             What putting the old record's entry back answered.
             */
            const put = await putASetAsideEntryBack({
              queueFile: folder.queueFile,
              runsFolder: folder.runs,
              name: 'c',
              position: 0,
            },);
            expect(put.line,).toContain('put c back at position 0 of 3',);
            expect(await namesIn(folder.queueFile,),).toEqual(['c', 'a', 'b',],);
            expect(await setAsideRecordIsThere({
              runsFolder: folder.runs,
              name: 'c',
            },),).toBe(false,);
          },
        },),
      ],
    },),
  ],
},);
