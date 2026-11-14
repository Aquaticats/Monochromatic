/**
 Where an entry whose start failed is taken out of the way: the record written beside the runs holding the entry whole, its reason and
 the time, the entry leaving the queue after it, and a failure between the two losing nothing.

 The failing write is made real: the queue's folder is made unwritable, so the record is written and the queue write is the step that
 fails, which is what a kill between the two leaves.

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
  isASetAsideRecordFile,
  lockFileOfAQueue,
  putASetAsideEntryBack,
  QueueLockHeldError,
  readQueueOfRuns,
  removeSetAsideRecord,
  setAsideAnEntryOfAQueue,
  setAsideFileOf,
  setAsideRecordIn,
  setAsideRecordIsThere,
  setAsideRecordOf,
  setAsideRecordsIn,
  SetAsideRecordError,
  writeSetAsideRecord,
} from '../dist/final/node/index.mjs';

/**
 A folder a test may write in, with the runs and the queue in folders of their own, removed when the test ends.
 */
type AThrowawayFolder = {
  /**
   The folder's path.
   */
  readonly path: string;

  /**
   The folder the runs and the set-aside records live in.
   */
  readonly runs: string;

  /**
   The folder the queue lives in.
   */
  readonly queueFolder: string;

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
 A folder a test may write in, with the runs and the queue in folders of their own, removed when the test ends.

 @returns the folder
 */
async function makeFolder(): Promise<AThrowawayFolder> {
  /**
   The folder's path.
   */
  const path = await mkdtemp(join(scratchRoot(), 'pi-set-aside-',),);
  /**
   The folder the runs live in.
   */
  const runs = join(path, 'runs',);
  /**
   The folder the queue lives in, kept apart so one may be made unwritable while the other is written.
   */
  const queueFolder = join(path, 'queue',);
  await mkdir(runs, { recursive: true, },);
  await mkdir(queueFolder, { recursive: true, },);
  /**
   The queue's path.
   */
  const queueFile = join(queueFolder, 'queue.json',);
  await writeFile(
    queueFile,
    `${JSON.stringify([
      {
        name: 'a',
        branch: 'feat/a',
        prompt: '/runs/a-prompt.md',
        wall: 28_800,
      },
      { name: 'b', branch: 'feat/b', },
    ],)}\n`,
  );
  return {
    path,
    runs,
    queueFolder,
    queueFile,
    async [Symbol.asyncDispose]() {
      await rm(path, { recursive: true, force: true, },);
    },
  };
}

/**
 The entry every test here sets aside.
 */
const THE_ENTRY = {
  name: 'a',
  branch: 'feat/a',
  prompt: '/runs/a-prompt.md',
  wall: 28_800,
};

await describe({
  name: '',
  children: [
    describe({
      name: setAsideFileOf.name,
      children: [
        it({
          name: 'names one file per entry beside the runs, and says which file names a record',
          fn: async () => {
            expect(setAsideFileOf({ runsFolder: '/runs', name: 'a', },),).toBe('/runs/a-set-aside.json',);
            expect(isASetAsideRecordFile({ file: 'a-set-aside.json', },),).toBe(true,);
            expect(isASetAsideRecordFile({ file: 'a-start-failure.log', },),).toBe(false,);
          },
        },),
      ],
    },),
    describe({
      name: setAsideRecordIn.name,
      children: [
        it({
          name: 'reads a record that holds an entry, a reason and a time, and refuses text that holds no entry',
          fn: async () => {
            expect(setAsideRecordIn({
              shown: JSON.stringify({
                name: 'a',
                entry: THE_ENTRY,
                reason: 'nothing to review under src in main...a',
                setAsideAt: '2026-11-14T09:00:00.000-05:00',
              },),
              file: 'a-set-aside.json',
            },),).toMatchObject({ reason: 'nothing to review under src in main...a', },);
            /**
             What reading a record with no entry threw.
             */
            let caught: unknown;
            try {
              setAsideRecordIn({
                shown: '{"name":"a","reason":"x","setAsideAt":"y"}',
                file: 'a-set-aside.json',
              },);
            }
            catch (error) {
              caught = error;
            }
            expect(caught,).toBeInstanceOf(SetAsideRecordError,);
          },
        },),
      ],
    },),
    describe({
      name: setAsideAnEntryOfAQueue.name,
      children: [
        it({
          name: 'writes the record holding the entry whole and its reason and time, then takes the entry out of the queue',
          fn: async () => {
            await using folder = await makeFolder();
            /**
             When the record is written.
             */
            const now = Date.parse('2026-11-14T09:00:00.000-05:00',);
            /**
             What setting the entry aside answered.
             */
            const kept = await setAsideAnEntryOfAQueue({
              queueFile: folder.queueFile,
              runsFolder: folder.runs,
              record: THE_ENTRY,
              reason: 'nothing to review under src in main...a',
              now: function nowOfTheRecord() {
                return now;
              },
            },);
            expect(kept.file,).toBe(setAsideFileOf({ runsFolder: folder.runs, name: 'a', },),);
            expect(kept.left,).toBe(1,);
            expect(await setAsideRecordOf({ runsFolder: folder.runs, name: 'a', },),).toEqual({
              name: 'a',
              entry: THE_ENTRY,
              reason: 'nothing to review under src in main...a',
              setAsideAt: new Date(now,).toISOString(),
            },);
            expect((await readQueueOfRuns({ queueFile: folder.queueFile, },)).map(function nameOf(entry,) {
              return entry.name;
            },),).toEqual(['b',],);
          },
        },),
        it({
          name: 'writes no record at all when another writer holds the queue, so a set-aside is one held step of both files',
          fn: async () => {
            await using folder = await makeFolder();
            // This test's own process holds the queue's lock, so the process number the lock names is alive and the wait runs out.
            await writeFile(
              lockFileOfAQueue({ queueFile: folder.queueFile, },),
              `${JSON.stringify({
                pid: process.pid,
                at: '2026-11-14T00:00:00.000-05:00',
              },)}\n`,
            );
            /**
             What setting the entry aside threw.
             */
            let caught: unknown;
            try {
              await setAsideAnEntryOfAQueue({
                queueFile: folder.queueFile,
                runsFolder: folder.runs,
                record: THE_ENTRY,
                reason: 'nothing to review under src in main...a',
                now: Date.now,
                waitMilliseconds: 30,
              },);
            }
            catch (error) {
              caught = error;
            }
            expect(caught,).toBeInstanceOf(QueueLockHeldError,);
            // Before the lock covered both files, the record was written here even though another writer held the queue, which is
            // the state a put-back could then delete.
            expect(await setAsideRecordIsThere({ runsFolder: folder.runs, name: 'a', },),).toBe(false,);
            expect((await readQueueOfRuns({ queueFile: folder.queueFile, },)).map(function nameOf(entry,) {
              return entry.name;
            },),).toEqual(['a', 'b',],);
            await rm(lockFileOfAQueue({ queueFile: folder.queueFile, },), { force: true, },);
          },
        },),
        it({
          name: 'leaves a record a kill left with its entry still queued for the filling to finish or drop, losing nothing',
          fn: async () => {
            await using folder = await makeFolder();
            // A kill between the record write and the queue write leaves both: the entry is queued and its record says why the
            // start failed. The put-back must not delete that record; the filling's sweep decides inside its own held step.
            await writeSetAsideRecord({
              runsFolder: folder.runs,
              record: {
                name: 'a',
                entry: THE_ENTRY,
                reason: 'nothing to review under src in main...a',
                setAsideAt: '2026-11-14T09:00:00.000-05:00',
              },
            },);
            expect(await setAsideRecordIsThere({ runsFolder: folder.runs, name: 'a', },),).toBe(true,);
            expect((await readQueueOfRuns({ queueFile: folder.queueFile, },)).map(function nameOf(entry,) {
              return entry.name;
            },),).toEqual(['a', 'b',],);
          },
        },),
      ],
    },),
    describe({
      name: setAsideRecordsIn.name,
      children: [
        it({
          name: 'names every entry set aside beside the runs, and answers none for a folder holding no record',
          fn: async () => {
            await using folder = await makeFolder();
            expect(await setAsideRecordsIn({ runsFolder: folder.runs, },),).toEqual([],);
            await writeSetAsideRecord({
              runsFolder: folder.runs,
              record: {
                name: 'a',
                entry: THE_ENTRY,
                reason: 'the change leaves nothing to review',
                setAsideAt: '2026-11-14T09:00:00.000-05:00',
                keptOutputAt: join(folder.runs, 'a-start-failure.log',),
              },
            },);
            expect(await setAsideRecordsIn({ runsFolder: folder.runs, },),).toEqual([{
              name: 'a',
              entry: THE_ENTRY,
              reason: 'the change leaves nothing to review',
              setAsideAt: '2026-11-14T09:00:00.000-05:00',
              keptOutputAt: join(folder.runs, 'a-start-failure.log',),
            },],);
          },
        },),
      ],
    },),
    describe({
      name: putASetAsideEntryBack.name,
      children: [
        it({
          name: 'puts the entry back at the head of the queue and removes its record, so nothing is lost',
          fn: async () => {
            await using folder = await makeFolder();
            await setAsideAnEntryOfAQueue({
              queueFile: folder.queueFile,
              runsFolder: folder.runs,
              record: THE_ENTRY,
              reason: 'a prompt file that is not there',
              now: Date.now,
            },);
            /**
             What putting the entry back answered.
             */
            const put = await putASetAsideEntryBack({
              queueFile: folder.queueFile,
              runsFolder: folder.runs,
              name: 'a',
              position: 0,
            },);
            expect(put.line,).toContain('put a back at position 0 of 2',);
            expect(put.line,).toContain('a prompt file that is not there',);
            expect(await readQueueOfRuns({ queueFile: folder.queueFile, },),).toEqual([THE_ENTRY, { name: 'b', branch: 'feat/b', },],);
            expect(await setAsideRecordIsThere({ runsFolder: folder.runs, name: 'a', },),).toBe(false,);
          },
        },),
        it({
          name: 'says nothing changed when no record names that entry',
          fn: async () => {
            await using folder = await makeFolder();
            expect((await putASetAsideEntryBack({
              queueFile: folder.queueFile,
              runsFolder: folder.runs,
              name: 'gone',
              position: 0,
            },)).line,).toContain('nothing changed',);
          },
        },),
        it({
          name: 'leaves the record of an entry that is still queued alone, because deleting it would lose an entry in flight',
          fn: async () => {
            await using folder = await makeFolder();
            await writeSetAsideRecord({
              runsFolder: folder.runs,
              record: {
                name: 'a',
                entry: THE_ENTRY,
                reason: 'x',
                setAsideAt: '2026-11-14T09:00:00.000-05:00',
              },
            },);
            /**
             What putting the entry back answered.
             */
            const put = await putASetAsideEntryBack({
              queueFile: folder.queueFile,
              runsFolder: folder.runs,
              name: 'a',
              position: 0,
            },);
            expect(put.line,).toContain('is still in the queue',);
            expect(put.line,).toContain('the filling finishes or drops it',);
            expect((await readQueueOfRuns({ queueFile: folder.queueFile, },)).map(function nameOf(entry,) {
              return entry.name;
            },),).toEqual(['a', 'b',],);
            // The record is the only copy of why the entry failed; the filling's sweep keeps or drops it inside a held step.
            expect(await setAsideRecordIsThere({ runsFolder: folder.runs, name: 'a', },),).toBe(true,);
          },
        },),
      ],
    },),
    describe({
      name: removeSetAsideRecord.name,
      children: [
        it({
          name: 'answers that there was no record to delete rather than failing',
          fn: async () => {
            await using folder = await makeFolder();
            expect(await removeSetAsideRecord({ runsFolder: folder.runs, name: 'gone', },),).toBe(false,);
          },
        },),
      ],
    },),
  ],
},);
