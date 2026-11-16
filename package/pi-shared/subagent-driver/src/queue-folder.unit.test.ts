/**
 The folder a queue is: one file per entry, its order a key inside the entry, and the moves that change one entry at a time. What is held
 here is what the folder reader and the three moves promise, and the order keys that make a put at a position one file.

 @module
 */

import {
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
  type QueueRecord,
  QueueFileError,
  anOrderBetween,
  moveAQueueEntryTo,
  orderOf,
  putAQueueEntryIn,
  queueIdOf,
  readQueueOfRuns,
  takeAQueueEntryOut,
  writeTheQueueOfRuns,
} from '../dist/final/node/index.mjs';

/**
 A folder a test may write in, removed when the test ends.
 */
type AThrowawayFolder = {
  /**
   The folder's path.
   */
  readonly path: string;

  /**
   The queue's folder.
   */
  readonly queueFolder: string;

  /**
   Removes the folder and everything under it.
   */
  readonly [Symbol.asyncDispose]: () => Promise<void>;
};

/**
 Makes a throwaway folder with a queue of the entries given.

 @param entries - what the queue holds, in order

 @returns the folder and the queue's folder

 @example
 ```ts
 await using folder = await aFolderOf([{ name: 'a', branch: 'feat/a' }]);
 ```
 */
async function aFolderOf(entries: readonly QueueRecord[],): Promise<AThrowawayFolder> {
  /**
   The folder everything lives in.
   */
  const path = await mkdtemp(join(scratchRoot(), 'pi-queue-the-folder-',),);
  /**
   The queue's folder.
   */
  const queueFolder = join(path, 'queue',);
  await writeTheQueueOfRuns({
    queueFolder,
    entries,
  },);
  return {
    path,
    queueFolder,
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

await describe({
  name: '',
  children: [
    it({
      name: 'reads a whole queue in the order it was written, and gives every entry a place and an identity',
      fn: async () => {
        await using folder = await aFolderOf([runNamed('first',), runNamed('second',), runNamed('third',),],);
        /**
         The queue as it is read back.
         */
        const entries = await readQueueOfRuns({ queueFolder: folder.queueFolder, },);
        expect(entries.map(function nameOf(entry,) {
          return entry.name;
        },),).toEqual(['first', 'second', 'third',],);
        for (const entry of entries) {
          expect(typeof orderOf({ record: entry, },),).toBe('string',);
          expect(typeof queueIdOf({ record: entry, },),).toBe('string',);
        }
      },
    },),
    it({
      name: 'puts an entry between two others by writing one file, and the order it wrote stands between theirs',
      fn: async () => {
        await using folder = await aFolderOf([runNamed('first',), runNamed('third',),],);
        /**
         The keys the two entries carry.
         */
        const before = await readQueueOfRuns({ queueFolder: folder.queueFolder, },);
        await putAQueueEntryIn({
          queueFolder: folder.queueFolder,
          entry: runNamed('second',),
          position: 1,
        },);
        expect(await namesIn(folder.queueFolder,),).toEqual(['first', 'second', 'third',],);
        /**
         The queue after the put.
         */
        const after = await readQueueOfRuns({ queueFolder: folder.queueFolder, },);
        // The two entries the put did not touch keep the very keys they had, which is what says nothing was renumbered.
        expect(orderOf({ record: after[0] as QueueRecord, },),).toBe(orderOf({ record: before[0] as QueueRecord, },),);
        expect(orderOf({ record: after[2] as QueueRecord, },),).toBe(orderOf({ record: before[1] as QueueRecord, },),);
      },
    },),
    it({
      name: 'takes an entry out by name and answers it, changing nothing when no entry has that name',
      fn: async () => {
        await using folder = await aFolderOf([runNamed('a',), runNamed('b',),],);
        /**
         What taking the entry out answered.
         */
        const taken = await takeAQueueEntryOut({
          queueFolder: folder.queueFolder,
          name: 'a',
        },);
        expect(taken.taken.map(function nameOf(entry,) {
          return entry.name;
        },),).toEqual(['a',],);
        expect(await namesIn(folder.queueFolder,),).toEqual(['b',],);
        expect((await takeAQueueEntryOut({
          queueFolder: folder.queueFolder,
          name: 'gone',
        },)).taken,).toEqual([],);
      },
    },),
    it({
      name: 'moves one entry to a position, rewriting that entry alone',
      fn: async () => {
        await using folder = await aFolderOf([runNamed('a',), runNamed('b',), runNamed('c',),],);
        expect(await moveAQueueEntryTo({
          queueFolder: folder.queueFolder,
          name: 'c',
          position: 0,
        },),).toEqual({
          from: 2,
          to: 0,
          count: 3,
        },);
        expect(await namesIn(folder.queueFolder,),).toEqual(['c', 'a', 'b',],);
      },
    },),
    it({
      name: 'refuses an entry of a name the queue holds, because two runs of one name collide everywhere the name goes',
      fn: async () => {
        await using folder = await aFolderOf([runNamed('a',),],);
        /**
         What queueing the same name threw.
         */
        let caught: unknown;
        try {
          await putAQueueEntryIn({
            queueFolder: folder.queueFolder,
            entry: runNamed('a',),
            position: 0,
          },);
        }
        catch (error) {
          caught = error;
        }
        expect(caught,).toBeInstanceOf(QueueFileError,);
        expect(String(caught,),).toContain('already in the queue',);
        expect(await namesIn(folder.queueFolder,),).toEqual(['a',],);
      },
    },),
    it({
      name: 'refuses an entry whose file carries no place, and a folder that is not there, naming each',
      fn: async () => {
        await using folder = await aFolderOf([runNamed('a',),],);
        await writeFile(join(folder.queueFolder, 'nameless.json',), `${JSON.stringify({ name: 'nameless', },)}\n`,);
        /**
         What reading a queue with an entry that carries no place threw.
         */
        let noPlace: unknown;
        try {
          await readQueueOfRuns({ queueFolder: folder.queueFolder, },);
        }
        catch (error) {
          noPlace = error;
        }
        expect(String(noPlace,),).toContain('nameless',);
        /**
         What reading a folder that is not there threw.
         */
        let notThere: unknown;
        try {
          await readQueueOfRuns({ queueFolder: join(folder.queueFolder, 'elsewhere',), },);
        }
        catch (error) {
          notThere = error;
        }
        expect(String(notThere,),).toContain('cannot be read',);
      },
    },),
    it({
      name: 'reads two order keys as the digits of a fraction, and always finds a key strictly between any two',
      fn: async () => {
        /**
         Two keys with one between them, and one between those two, which is what a put at one place does again and again.
         */
        const pairs: readonly (readonly [string, string])[] = [
          ['i', 'j',],
          ['a', 'a1',],
          ['1', '2',],
          ['zz', 'zzz',],
        ];
        for (const [lower, upper,] of pairs) {
          /**
           The key between them.
           */
          const between = anOrderBetween({
            lower,
            upper,
          },);
          expect(between > lower,).toBe(true,);
          expect(between < upper,).toBe(true,);
        }
        // The head and the tail are the two bounds that need no other key: a key is found below the first and above the last.
        expect(anOrderBetween({ upper: '1', },) < '1',).toBe(true,);
        expect(anOrderBetween({ lower: 'z', },) > 'z',).toBe(true,);
      },
    },),
  ],
},);
