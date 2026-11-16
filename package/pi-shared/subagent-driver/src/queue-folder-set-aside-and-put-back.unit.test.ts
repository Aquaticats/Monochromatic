/**
 What a set-aside record holds, what a failure of the machine leaves, and where a put-back review goes: the three properties the tests of
 the one-file queue proved, asserted again on the queue folder.

 A set-aside writes the record first and takes the entry out after it, each one atomic step on one file, so a kill between the two leaves
 the entry in both places and the next pass finishes it. The pass's first step is that settlement, which is what these controls read.

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
  BUILD_VOLUME,
  type FillingSeams,
  type QueueRecord,
  claimAnEntryOfAQueue,
  fillFreePlacesFromAQueue,
  namesOfAnAttemptIn,
  NO_SET_ASIDE_RECORD,
  putASetAsideEntryBack,
  putAQueueEntryIn,
  putEndedReviewsBack,
  readQueueOfRuns,
  setAsideAnEntryOfAQueue,
  setAsideRecordOf,
  writeTheQueueOfRuns,
} from '../dist/final/node/index.mjs';

/**
 A folder a test may write in, with one queue folder and the runs folder beside it, removed when the test ends.
 */
type AQueueInAThrowawayFolder = {
  /**
   The queue's folder.
   */
  readonly queueFolder: string;

  /**
   The runs folder, where a record and a unit's output file go.
   */
  readonly runsFolder: string;

  /**
   The folder everything lives in.
   */
  readonly path: string;

  /**
   Removes the folder and everything under it.
   */
  readonly [Symbol.asyncDispose]: () => Promise<void>;
};

/**
 Makes a queue of the entries given in a throwaway folder.

 @param entries - what the queue holds, in order

 @returns the queue's folder and the runs folder beside it

 @example
 ```ts
 await using queue = await aQueueOf([{ name: 'a', branch: 'feat/a' }]);
 ```
 */
async function aQueueOf(entries: readonly QueueRecord[],): Promise<AQueueInAThrowawayFolder> {
  /**
   The folder everything lives in.
   */
  const path = await mkdtemp(join(scratchRoot(), 'pi-queue-set-aside-',),);
  /**
   The queue's folder.
   */
  const queueFolder = join(path, 'queue',);
  /**
   The runs folder beside it.
   */
  const runsFolder = join(path, 'runs',);
  await mkdir(runsFolder, { recursive: true, },);
  // An empty queue is a folder with nothing in it, which the first put makes when the folder is not there.
  await mkdir(queueFolder, { recursive: true, },);
  await writeTheQueueOfRuns({
    queueFolder,
    entries,
  },);
  return {
    path,
    queueFolder,
    runsFolder,
    async [Symbol.asyncDispose]() {
      await rm(path, { recursive: true, force: true, },);
    },
  };
}

/**
 One entry of the queue, named after the word given.

 @param name - the entry's name

 @returns the entry

 @example
 ```ts
 const entry = runNamed('a');
 ```
 */
function runNamed(name: string,): QueueRecord {
  return {
    name,
    branch: `feat/${name}`,
    prompt: `/runs/${name}-prompt.md`,
    wall: 28_800,
  };
}

/**
 The names a queue holds, in its order.

 @param queueFolder - the queue's folder

 @returns the names

 @example
 ```ts
 const names = await namesIn('/runs/q');
 ```
 */
async function namesIn(queueFolder: string,): Promise<readonly string[]> {
  return (await readQueueOfRuns({ queueFolder, },)).map(function nameOf(entry,) {
    return entry.name;
  },);
}

/**
 The seams one pass asks, whose start answers what the test says and whose set-aside is the package's own.

 @param request - the queue and the runs folder, and what a start answers

 @returns the seams

 @example
 ```ts
 const seams = seamsOf({ queueFolder: '/runs/q', runsFolder: '/runs', kind: 'the-machine' });
 ```
 */
function seamsOf({
  queueFolder,
  runsFolder,
  kind,
}: {
  readonly queueFolder: string;
  readonly runsFolder: string;
  readonly kind: 'the-entry' | 'the-machine';
},): FillingSeams {
  return {
    runsGoing: function runsGoing() {
      return Promise.resolve([],);
    },
    freeSpace: function freeSpaceNow() {
      return {
        volume: BUILD_VOLUME,
        freeBytes: 900 * 1_024 * 1_024 * 1_024,
      };
    },
    start: function startOne({ record, claim, },) {
      /**
       The names the attempt's own unit writes.
       */
      const names = namesOfAnAttemptIn({
        runsFolder,
        name: record.name,
        claim,
      },);
      return Promise.resolve({
        started: false as const,
        failure: {
          kind,
          reason: (kind === 'the-machine') ? 'the manager is not there' : 'nothing to review under src',
          output: 'the step refused',
        },
      },);
    },
    setAside: function setAside({
      record,
      failure,
      queueId,
    },) {
      return setAsideAnEntryOfAQueue({
        queueFolder,
        runsFolder,
        record,
        queueId,
        reason: failure.reason,
        now: Date.now,
      },);
    },
    wait: function wait() {
      return Promise.resolve();
    },
    report: function report() {
      return undefined;
    },
  };
}

await describe({
  name: '',
  children: [
    it({
      name: 'writes the record holding the entry whole, its reason, its time and its identity, and then takes the entry out',
      fn: async () => {
        await using queue = await aQueueOf([runNamed('a',),],);
        /**
         The entry as the queue holds it, which is the entry the claim and the record are made for.
         */
        const held = (await readQueueOfRuns({ queueFolder: queue.queueFolder, },))[0] as QueueRecord;
        await setAsideAnEntryOfAQueue({
          queueFolder: queue.queueFolder,
          runsFolder: queue.runsFolder,
          record: held,
          queueId: String(held.queueId,),
          reason: 'nothing to review under src in main...a',
          now: function nowOfTheRecord() {
            return Date.parse('2026-11-16T00:00:00.000-05:00',);
          },
        },);
        /**
         The record the set-aside left.
         */
        const record = await setAsideRecordOf({
          runsFolder: queue.runsFolder,
          name: 'a',
        },);
        expect(record === NO_SET_ASIDE_RECORD ? undefined : record.reason,).toBe('nothing to review under src in main...a',);
        expect(record === NO_SET_ASIDE_RECORD ? undefined : record.setAsideAt,).toBe('2026-11-16T05:00:00.000Z',);
        // The record holds the entry whole, with the identity its claim was taken on, and the queue no longer holds it.
        const { queueId, order, ...own } = (record as { readonly entry: QueueRecord }).entry;
        expect(own,).toEqual(runNamed('a',),);
        expect(queueId,).toBe(String(held.queueId,),);
        expect(typeof order,).toBe('string',);
        expect(await namesIn(queue.queueFolder,),).toEqual([],);
      },
    },),
    it({
      name: 'leaves the entry in place and starts nothing behind it when the machine refuses its start',
      fn: async () => {
        await using queue = await aQueueOf([runNamed('a',), runNamed('b',),],);
        /**
         What the one pass answered.
         */
        const outcome = await fillFreePlacesFromAQueue({
          request: {
            queueFolder: queue.queueFolder,
            runsFolder: queue.runsFolder,
            level: 2,
            startsAtMost: 2,
            staggerMilliseconds: 1,
          },
          seams: seamsOf({
            queueFolder: queue.queueFolder,
            runsFolder: queue.runsFolder,
            kind: 'the-machine',
          },),
        },);
        // The entry keeps its place and its place in the order, nothing is set aside, and the pass stops rather than starting the
        // entry behind it into the place the earlier one waits for.
        expect(outcome.started,).toEqual([],);
        expect(outcome.setAsideNames,).toEqual([],);
        expect(outcome.stoppedBecause,).toBe('the machine said no: the manager is not there',);
        expect(await namesIn(queue.queueFolder,),).toEqual(['a', 'b',],);
        expect(await setAsideRecordOf({ runsFolder: queue.runsFolder, name: 'a', },),).toBe(NO_SET_ASIDE_RECORD,);
      },
    },),
    it({
      name: 'sets aside an entry whose start is the entry\'s own failure, and starts the entry behind it in the same pass',
      fn: async () => {
        await using queue = await aQueueOf([runNamed('a',), runNamed('b',),],);
        /**
         The starts the pass asked for.
         */
        const started: string[] = [];
        /**
         What the one pass answered.
         */
        const outcome = await fillFreePlacesFromAQueue({
          request: {
            queueFolder: queue.queueFolder,
            runsFolder: queue.runsFolder,
            level: 2,
            startsAtMost: 2,
            staggerMilliseconds: 1,
          },
          seams: {
            ...seamsOf({
              queueFolder: queue.queueFolder,
              runsFolder: queue.runsFolder,
              kind: 'the-entry',
            },),
            start({ record, claim, },) {
              if (record.name === 'a') {
                return seamsOf({
                  queueFolder: queue.queueFolder,
                  runsFolder: queue.runsFolder,
                  kind: 'the-entry',
                },).start({
                  record,
                  claim,
                },);
              }
              started.push(record.name,);
              /**
               The names this attempt's unit writes.
               */
              const names = namesOfAnAttemptIn({
                runsFolder: queue.runsFolder,
                name: record.name,
                claim,
              },);
              return Promise.resolve({
                started: true as const,
                lines: [],
                batchFile: names.batchFile,
                unit: `launch-${record.name}`,
                outputFile: names.outputFile,
              },);
            },
          },
        },);
        expect(outcome.setAsideNames,).toEqual(['a',],);
        expect(started,).toEqual(['b',],);
        expect(await namesIn(queue.queueFolder,),).toEqual([],);
      },
    },),
    it({
      name: 'puts an entry back with the identity its record holds, and leaves the record of a put-back that could not place it',
      fn: async () => {
        await using queue = await aQueueOf([runNamed('a',),],);
        /**
         The entry as the queue holds it.
         */
        const held = (await readQueueOfRuns({ queueFolder: queue.queueFolder, },))[0] as QueueRecord;
        await claimAnEntryOfAQueue({
          queueFolder: queue.queueFolder,
          record: held,
          now: Date.now,
        },);
        await setAsideAnEntryOfAQueue({
          queueFolder: queue.queueFolder,
          runsFolder: queue.runsFolder,
          record: held,
          queueId: String(held.queueId,),
          reason: 'the entry failed its start',
          now: Date.now,
        },);
        /**
         What putting the entry back answered.
         */
        const putBack = await putASetAsideEntryBack({
          queueFolder: queue.queueFolder,
          runsFolder: queue.runsFolder,
          name: 'a',
          position: 0,
        },);
        expect(putBack.line,).toContain('put a back at position 0 of 1',);
        /**
         The entry the queue holds again.
         */
        const back = (await readQueueOfRuns({ queueFolder: queue.queueFolder, },))[0] as QueueRecord;
        expect(String(back.queueId,),).toBe(String(held.queueId,),);
        expect(await setAsideRecordOf({ runsFolder: queue.runsFolder, name: 'a', },),).toBe(NO_SET_ASIDE_RECORD,);
        // A name queued while the record stands is not written over: the put-back answers that it could not place the entry and the
        // record stays, which is the entry's copy.
        await using second = await aQueueOf([],);
        await writeFile(
          join(second.runsFolder, 'a-set-aside.json',),
          `${JSON.stringify({
            name: 'a',
            entry: {
              ...runNamed('a',),
              order: 'i',
              queueId: String(held.queueId,),
            },
            reason: 'the entry failed its start',
            setAsideAt: '2026-11-16T00:00:00.000-05:00',
          },)}\n`,
        );
        await putAQueueEntryIn({
          queueFolder: second.queueFolder,
          entry: runNamed('a',),
          position: 0,
        },);
        /**
         What the put-back answered with the name taken.
         */
        const refused = await putASetAsideEntryBack({
          queueFolder: second.queueFolder,
          runsFolder: second.runsFolder,
          name: 'a',
          position: 0,
        },);
        expect(refused.line,).toContain('is still in the queue, so it is not set aside',);
        expect(await setAsideRecordOf({ runsFolder: second.runsFolder, name: 'a', },),).toMatchObject({
          reason: 'the entry failed its start',
        },);
      },
    },),
    it({
      name: 'puts a rate-limited review back after every integration pass waiting at the head, and after none of a queue without one',
      fn: async () => {
        await using queue = await aQueueOf([
          { name: 'integration-pass-1', branch: 'fix/pass', },
          { name: 'other', branch: 'feat/other', },
        ],);
        /**
         The review's own attempt folder and its summary, which is what the put-back reads.
         */
        const attempt = join(queue.runsFolder, 'review-change-1',);
        await mkdir(attempt, { recursive: true, },);
        await writeFile(
          join(attempt, 'summary.json',),
          `${JSON.stringify({
            endReason: 'model-error',
            lastAssistantText: '429 insufficient_quota',
          },)}\n`,
        );
        /**
         What putting the ended reviews back answered.
         */
        const putBack = await putEndedReviewsBack({
          queueFolder: queue.queueFolder,
          runsFolder: queue.runsFolder,
          reviewsFolder: join(queue.path, 'reviews',),
          since: 0,
          going: new Set(),
          now: Date.now,
        },);
        expect(putBack.putBack,).toEqual(['review-change',],);
        expect(await namesIn(queue.queueFolder,),).toEqual(['integration-pass-1', 'review-change', 'other',],);
        // A review whose file is marked final is not put back, and the line says so.
        await using final = await aQueueOf([],);
        /**
         The final review's attempt folder and summary.
         */
        const finalAttempt = join(final.runsFolder, 'review-final-1',);
        await mkdir(finalAttempt, { recursive: true, },);
        await writeFile(
          join(finalAttempt, 'summary.json',),
          `${JSON.stringify({
            endReason: 'model-error',
            lastAssistantText: '429 insufficient_quota',
          },)}\n`,
        );
        await mkdir(join(final.path, 'reviews', 'final',), { recursive: true, },);
        await writeFile(join(final.path, 'reviews', 'final', 'review.md',), 'This review is final.\n',);
        /**
         What the put-back of a final review answered.
         */
        const notPutBack = await putEndedReviewsBack({
          queueFolder: final.queueFolder,
          runsFolder: final.runsFolder,
          reviewsFolder: join(final.path, 'reviews',),
          since: 0,
          going: new Set(),
          now: Date.now,
        },);
        expect(notPutBack.putBack,).toEqual([],);
        expect(notPutBack.lines.join('',),).toContain('marked final: not put back',);
      },
    },),
  ],
},);
