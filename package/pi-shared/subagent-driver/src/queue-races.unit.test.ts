/**
 Two writers to one queue file at the same time: the lost update the review of 2026-11-14 reproduced with two OS processes, and the
 lock that makes each queue's read, change and write one caller's own.

 The review's own scripts, run against the commit this file fixes, lost an entry in 47 rounds of 200 with both processes exiting 0,
 and brought a started entry back into the queue in 27 rounds of 200 while dropping a concurrent put in 35. The controls here are
 that shape, held by the code's own seams: a queue is changed by two real processes, and a lock is written by hand so a holder that
 is alive, gone or damaged is each met on purpose.

 Every queue file and every lock file lives in a throwaway folder.

 @module
 */

import {
  spawn,
} from 'node:child_process';
import { once, } from 'node:events';
import {
  mkdtemp,
  readdir,
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
  QueueFileError,
  QueueLockHeldError,
  acquireTheQueueLock,
  changeTheQueueOfRuns,
  isThere,
  lockFileOfAQueue,
  putAQueueEntryIn,
  readQueueOfRuns,
  takeAQueueEntryOut,
  writeQueueOfRuns,
  aPathWrittenBeside,
} from '../dist/final/node/index.mjs';

/**
 The stand-in one queue command's own change, run as a program.
 */
const A_QUEUE_WRITER = new URL('stand-in-queue-writer-for-tests.ts', import.meta.url,).pathname;

/**
 Seconds a child of this file may take, which only keeps one that hangs from holding the test.
 */
const CHILD_SECONDS = 30;

/**
 Rounds of a cross-process control. The review measured a lost entry in about a quarter of its rounds, and this lock makes the loss
 impossible; the rounds are what would show a caller waiting for a lock and then dropping its own change.
 */
const ROUNDS = 25;

/**
 A folder a test may write in, holding the queue files of its rounds.
 */
type AThrowawayFolder = {
  /**
   The folder's path.
   */
  readonly path: string;

  /**
   Removes the folder and everything under it.
   */
  readonly [Symbol.asyncDispose]: () => Promise<void>;
};

/**
 A folder a test may write in.

 @returns the folder
 */
async function makeFolder(): Promise<AThrowawayFolder> {
  /**
   The folder's path.
   */
  const path = await mkdtemp(join(scratchRoot(), 'pi-queue-races-',),);
  return {
    path,
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
 Runs one change of the stand-in as its own process.

 @param request - the folder the stand-in lives in, the change, the queue and the entry's name

 @returns its status and both streams
 */
async function runAWriter({
  folder,
  what,
  queueFile,
  name,
  position = 0,
}: {
  readonly folder: AThrowawayFolder;
  readonly what: 'put' | 'take';
  readonly queueFile: string;
  readonly name: string;
  readonly position?: number;
},): Promise<{
  readonly status: number;
  readonly stdout: string;
  readonly stderr: string;
}> {
  /**
   The child.
   */
  const child = spawn(
    process.execPath,
    [
      A_QUEUE_WRITER,
      what,
      queueFile,
      name,
      String(position,),
    ],
    {
      cwd: folder.path,
      stdio: [
        'ignore',
        'pipe',
        'pipe',
      ],
    },
  );
  /**
   What the child printed.
   */
  const printed: string[] = [];
  /**
   What the child complained about.
   */
  const complained: string[] = [];
  child.stdout
    .on(
    'data',
    function keep(chunk: Buffer,) {
      printed.push(chunk.toString(),);
    },
    );
  child.stderr
    .on(
    'data',
    function keep(chunk: Buffer,) {
      complained.push(chunk.toString(),);
    },
    );
  /**
   What the child reported when it closed: its status, or the signal that ended it.
   */
  const closed: readonly unknown[] = await once(
    child,
    'close',
  );
  /**
   The status the child exited with, or nothing when a signal ended it.
   */
  const [status,] = closed;
  return {
    status: ((typeof status) === 'number') ? status : -1,
    stdout: printed.join('',),
    stderr: complained.join('',),
  };
}

/**
 Runs a program that ends at once and answers its process number, which is gone by the time this answers.

 @returns the process number of a process that has ended
 */
async function pidOfAProcessThatHasEnded(): Promise<number> {
  /**
   The process that ends at once.
   */
  const child = spawn(
    'true',
    [],
    { stdio: 'ignore', },
  );
  /**
   Its process number, which its own end frees.
   */
  const pid = child.pid ?? 0;
  await once(
    child,
    'close',
  );
  return pid;
}

await describe({
  name: '',
  children: [
    describe({
      name: 'a queue file in the shape it had before the lock',
      children: [
        it({
          name: 'is read and changed as it is, with every entry kept and no file carried over',
          fn: async () => {
            await using folder = await makeFolder();
            /**
             The queue's path.
             */
            const queueFile = join(folder.path, 'queue.json',);
            // Today's shape, written by hand: one JSON array, one entry per line, exactly what the commands wrote before the lock.
            await writeFile(
              queueFile,
              '[\n  {"name":"a","branch":"feat/a"},\n  {"name":"b","branch":"feat/b"}\n]\n',
            );
            expect(await namesIn(queueFile,),).toEqual(['a', 'b',],);
            await putAQueueEntryIn({
              queueFile,
              entry: { name: 'c', branch: 'feat/c', },
              position: 2,
            },);
            /**
             What taking one entry out answered.
             */
            const taken = await takeAQueueEntryOut({
              queueFile,
              name: 'b',
            },);
            expect(taken.taken,).toHaveLength(1,);
            /**
             The names the queue holds after the change.
             */
            const names = await namesIn(queueFile,);
            expect(names,).toEqual(['a', 'c',],);
            // The only new name beside the queue is the lock's, and it is gone once the change ended.
            /**
             The lock's path.
             */
            const lockFile = lockFileOfAQueue({ queueFile, },);
            expect(await isThere(lockFile,),).toBe(false,);
          },
        },),
      ],
    },),
    describe({
      name: 'the queue lock',
      children: [
        it({
          name: 'is one file beside the queue while a change is made, and is gone once the block that took it ends',
          fn: async () => {
            await using folder = await makeFolder();
            /**
             The queue's path.
             */
            const queueFile = join(folder.path, 'queue.json',);
            await writeQueueOfRuns({
              queueFile,
              entries: [{ name: 'a', },],
            },);
            /**
             The lock's path.
             */
            const lockFile = lockFileOfAQueue({ queueFile, },);
            expect(await isThere(lockFile,),).toBe(false,);
            /**
             Whether the lock was there while the block ran.
             */
            let thereWhileHeld = false;
            await (async function changeUnderTheLock() {
              await using lock = await acquireTheQueueLock({ queueFile, },);
              thereWhileHeld = await isThere(lockFile,);
            })();
            expect(thereWhileHeld,).toBe(true,);
            expect(await isThere(lockFile,),).toBe(false,);
          },
        },),
        it({
          name: 'is given back when the change itself refuses, so a refused entry does not hold the queue',
          fn: async () => {
            await using folder = await makeFolder();
            /**
             The queue's path.
             */
            const queueFile = join(folder.path, 'queue.json',);
            await writeQueueOfRuns({
              queueFile,
              entries: [{ name: 'a', },],
            },);
            /**
             What the refused put threw.
             */
            let caught: unknown;
            try {
              await putAQueueEntryIn({
                queueFile,
                entry: { name: 'a', },
                position: 0,
              },);
            }
            catch (error) {
              caught = error;
            }
            expect(caught,).toBeInstanceOf(QueueFileError,);
            /**
             The lock's path, which a refused change must not leave behind.
             */
            const lockFile = lockFileOfAQueue({ queueFile, },);
            expect(await isThere(lockFile,),).toBe(false,);
            expect(await putAQueueEntryIn({
              queueFile,
              entry: { name: 'b', },
              position: 1,
            },),).toHaveProperty('count', 2,);
          },
        },),
        it({
          name: 'is broken when its holder\'s process is gone, so a killed writer never holds the queue',
          fn: async () => {
            await using folder = await makeFolder();
            /**
             The queue's path.
             */
            const queueFile = join(folder.path, 'queue.json',);
            await writeQueueOfRuns({
              queueFile,
              entries: [{ name: 'a', },],
            },);
            /**
             The process number of a writer that has ended.
             */
            const gonePid = await pidOfAProcessThatHasEnded();
            await writeFile(
              lockFileOfAQueue({ queueFile, },),
              `${JSON.stringify({
                pid: gonePid,
                at: new Date().toISOString(),
              },)}\n`,
            );
            /**
             When the lock was taken by the process that died.
             */
            expect(await putAQueueEntryIn({
              queueFile,
              entry: { name: 'b', },
              position: 1,
            },),).toHaveProperty('count', 2,);
            expect(await namesIn(queueFile,),).toEqual(['a', 'b',],);
            /**
             The lock's path, which a broken lock must not leave behind.
             */
            const lockFile = lockFileOfAQueue({ queueFile, },);
            expect(await isThere(lockFile,),).toBe(false,);
          },
        },),
        it({
          name: 'is broken when it names no process, which only a damaged lock file does',
          fn: async () => {
            await using folder = await makeFolder();
            /**
             The queue's path.
             */
            const queueFile = join(folder.path, 'queue.json',);
            await writeQueueOfRuns({
              queueFile,
              entries: [{ name: 'a', },],
            },);
            await writeFile(lockFileOfAQueue({ queueFile, },), '{}\n',);
            expect(await putAQueueEntryIn({
              queueFile,
              entry: { name: 'b', },
              position: 1,
            },),).toHaveProperty('count', 2,);
          },
        },),
        it({
          name: 'refuses a change after its wait when another live writer holds it, naming that writer and changing nothing',
          fn: async () => {
            await using folder = await makeFolder();
            /**
             The queue's path.
             */
            const queueFile = join(folder.path, 'queue.json',);
            await writeQueueOfRuns({
              queueFile,
              entries: [{ name: 'a', },],
            },);
            // This test's own process holds the lock, so the process number it names is alive and the wait runs out.
            await writeFile(
              lockFileOfAQueue({ queueFile, },),
              `${JSON.stringify({
                pid: process.pid,
                at: '2026-11-14T00:00:00.000-05:00',
              },)}\n`,
            );
            /**
             What the refused change threw.
             */
            let caught: unknown;
            try {
              await changeTheQueueOfRuns({
                queueFile,
                waitMilliseconds: 30,
                change: function putTheEntry(queue,) {
                  return {
                    entries: [...queue, { name: 'b', },],
                    answer: undefined,
                  };
                },
              },);
            }
            catch (error) {
              caught = error;
            }
            expect(caught,).toBeInstanceOf(QueueLockHeldError,);
            expect(String(caught,),).toContain(`process ${String(process.pid,)}`,);
            expect(String(caught,),).toContain(queueFile,);
            expect(String(caught,),).toContain('2026-11-14T00:00:00.000-05:00',);
            expect(await namesIn(queueFile,),).toEqual(['a',],);
            /**
             The live holder's lock, which is the holder's to give back: it is still there and the test removes it.
             */
            const lockFile = lockFileOfAQueue({ queueFile, },);
            expect(await isThere(lockFile,),).toBe(true,);
            await rm(lockFile, { force: true, },);
          },
        },),
        it({
          name: 'names each file written beside a path by the process number and a random part, so two writes of one process never share one',
          fn: async () => {
            expect(aPathWrittenBeside({ path: '/runs/queue.json', },),).not.toBe(aPathWrittenBeside({ path: '/runs/queue.json', },),);
            expect(aPathWrittenBeside({ path: '/runs/queue.json', },),).toContain(String(process.pid,),);
          },
        },),
      ],
    },),
    describe({
      name: 'two writers of one queue at the same time',
      children: [
        it({
          name: 'puts one entry each in one process, and both entries are there',
          fn: async () => {
            await using folder = await makeFolder();
            /**
             The queue's path.
             */
            const queueFile = join(folder.path, 'queue.json',);
            await writeQueueOfRuns({
              queueFile,
              entries: [{ name: 'base', },],
            },);
            await Promise.all([
              putAQueueEntryIn({
                queueFile,
                entry: { name: 'A', },
                position: 1,
              },),
              putAQueueEntryIn({
                queueFile,
                entry: { name: 'B', },
                position: 1,
              },),
            ],);
            /**
             The names the queue holds.
             */
            const names = await namesIn(queueFile,);
            expect([...names,].toSorted(),).toEqual(['A', 'B', 'base',],);
            // Neither write left a file beside the queue.
            expect((await readdir(folder.path,)).filter(function isNotTheQueue(name,) {
              return name !== 'queue.json';
            },),).toEqual([],);
          },
        },),
        it({
          name: 'puts one entry each in two processes for every round, and no entry is lost and both say they queued it',
          fn: async () => {
            await using folder = await makeFolder();
            /* oxlint-disable eslint/no-await-in-loop -- one round after another, each with its own queue file and its own pair of
               writers, which is what a race control measures. See doc/lint/sequential-awaits.md. */
            for (let round = 0; round < ROUNDS; round += 1) {
              /**
               This round's queue.
               */
              const queueFile = join(folder.path, `puts-${String(round,)}.json`,);
              await writeQueueOfRuns({
                queueFile,
                entries: [{ name: 'base', },],
              },);
              /**
               What the two writers answered.
               */
              const answered = await Promise.all([
                runAWriter({
                  folder,
                  what: 'put',
                  queueFile,
                  name: 'A',
                  position: 1,
                },),
                runAWriter({
                  folder,
                  what: 'put',
                  queueFile,
                  name: 'B',
                  position: 1,
                },),
              ],);
              expect(answered.map(function statusOf(answer,) {
                return answer.status;
              },),).toEqual([0, 0,],);
              /**
               The names this round's queue holds.
               */
              const names = await namesIn(queueFile,);
              expect([...names,].toSorted(),).toEqual(['A', 'B', 'base',],);
            }
          },
        },),
        it({
          name: 'takes one entry out while another puts one in, for every round, and the taken entry stays out and the put entry is kept',
          fn: async () => {
            await using folder = await makeFolder();
            /* oxlint-disable eslint/no-await-in-loop -- one round after another, each with its own queue file and its own pair of
               writers. See doc/lint/sequential-awaits.md. */
            for (let round = 0; round < ROUNDS; round += 1) {
              /**
               This round's queue.
               */
              const queueFile = join(folder.path, `take-put-${String(round,)}.json`,);
              await writeQueueOfRuns({
                queueFile,
                entries: [{ name: 'X', }, { name: 'Y', },],
              },);
              /**
               What the two writers answered.
               */
              const answered = await Promise.all([
                runAWriter({
                  folder,
                  what: 'take',
                  queueFile,
                  name: 'X',
                },),
                runAWriter({
                  folder,
                  what: 'put',
                  queueFile,
                  name: 'R',
                  position: 0,
                },),
              ],);
              expect(answered.map(function statusOf(answer,) {
                return answer.status;
              },),).toEqual([0, 0,],);
              expect(await namesIn(queueFile,),).toEqual(['R', 'Y',],);
            }
          },
        },),
      ],
    },),
  ],
},);
