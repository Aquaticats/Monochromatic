/**
 The queue file and the three shapes an entry takes: reading and writing the queue whole through a file beside it, taking one entry
 out, putting one in, moving one, and refusing an entry that names a field the shape does not read rather than dropping it.

 Every test writes in a throwaway folder, so no real queue is read or written.

 @module
 */

import {
  mkdtemp,
  readFile,
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
  changeOfAReviewRun,
  moveAQueueEntryTo,
  poolAPlaceAskedForWouldHold,
  poolOfAQueueEntry,
  putAQueueEntryIn,
  QueueFileError,
  readQueueOfRuns,
  resumesAQueueEntry,
  shapeOfAQueueEntry,
  type ShapeOfAQueueEntry,
  takeAQueueEntryOut,
  writtenQueueOfRuns,
  writeQueueOfRuns,
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
   Removes the folder and everything under it.
   */
  readonly [Symbol.asyncDispose]: () => Promise<void>;
};

/**
 A folder a test may write in, removed when the test ends.

 @returns the folder
 */
async function makeFolder(): Promise<AThrowawayFolder> {
  /**
   The folder's path.
   */
  const path = await mkdtemp(join(scratchRoot(), 'pi-queue-file-',),);
  return {
    path,
    async [Symbol.asyncDispose]() {
      await rm(path, { recursive: true, force: true, },);
    },
  };
}

/**
 Writes a queue file holding the entries given.

 @param request - where the queue goes and what it holds

 @returns the queue's path
 */
async function queueOf({
  folder,
  entries,
}: {
  readonly folder: AThrowawayFolder;
  readonly entries: readonly unknown[];
},): Promise<string> {
  /**
   The queue's path.
   */
  const queueFile = join(folder.path, 'queue.json',);
  await writeFile(
    queueFile,
    `${JSON.stringify(entries,)}\n`,
  );
  return queueFile;
}

/**
 A fresh run of the queue, named `a`.
 */
const A_RUN = {
  name: 'a',
  branch: 'feat/a',
  prompt: '/runs/a-prompt.md',
  base: 'abc1234',
  wall: 28_800,
};

/**
 A review of a range of a change, named `review-b`.
 */
const A_REVIEW_OF_A_RANGE = {
  name: 'review-b',
  reviewOfRange: 'main...353d5e06d7',
  commit: '353d5e06d7',
  source: 'package/cargo-shadow/cli/src',
};

await describe({
  name: '',
  children: [
    describe({
      name: readQueueOfRuns.name,
      children: [
        it({
          name: 'reads the entries of a queue file in the order it holds them',
          fn: async () => {
            await using folder = await makeFolder();
            /**
             The queue the test writes.
             */
            const queueFile = await queueOf({
              folder,
              entries: [
                A_RUN,
                A_REVIEW_OF_A_RANGE,
              ],
            },);
            expect(await readQueueOfRuns({ queueFile, },),).toEqual([
              A_RUN,
              A_REVIEW_OF_A_RANGE,
            ],);
          },
        },),
        it({
          name: 'refuses a file that holds no JSON array, naming the file',
          fn: async () => {
            await using folder = await makeFolder();
            /**
             The queue the test writes.
             */
            const queueFile = await queueOf({ folder, entries: [], },);
            await writeFile(
              queueFile,
              '{"name":"a"}\n',
            );
            /**
             What reading the queue threw.
             */
            let caught: unknown;
            try {
              await readQueueOfRuns({ queueFile, },);
            }
            catch (error) {
              caught = error;
            }
            expect(caught,).toBeInstanceOf(QueueFileError,);
            expect(String(caught,),).toContain(queueFile,);
            expect(String(caught,),).toContain('holds no JSON array',);
          },
        },),
        it({
          name: 'refuses an entry that is no object naming a run, naming which entry it is',
          fn: async () => {
            await using folder = await makeFolder();
            /**
             The queue the test writes.
             */
            const queueFile = await queueFileWithText({
              folder,
              text: '[{"name":"a"},{"branch":"feat/b"}]\n',
            },);
            /**
             What reading the queue threw.
             */
            let caught: unknown;
            try {
              await readQueueOfRuns({ queueFile, },);
            }
            catch (error) {
              caught = error;
            }
            expect(caught,).toBeInstanceOf(QueueFileError,);
            expect(String(caught,),).toContain('entry 2',);
          },
        },),
      ],
    },),
    describe({
      name: writtenQueueOfRuns.name,
      children: [
        it({
          name: 'writes one line per entry, which reads back as the same entries',
          fn: async () => {
            /**
             What the writer answered.
             */
            const text = writtenQueueOfRuns({
              entries: [
                A_RUN,
                A_REVIEW_OF_A_RANGE,
              ],
            },);
            expect(text,).toBe(`[\n  ${JSON.stringify(A_RUN,)},\n  ${JSON.stringify(A_REVIEW_OF_A_RANGE,)}\n]\n`,);
          },
        },),
      ],
    },),
    describe({
      name: 'the queue file a write leaves',
      children: [
        it({
          name: 'is the whole new queue and no file beside it, so a reader never meets half a queue',
          fn: async () => {
            await using folder = await makeFolder();
            /**
             The queue the test writes.
             */
            const queueFile = await queueOf({ folder, entries: [A_RUN,], },);
            await writeQueueOfRuns({
              queueFile,
              entries: [A_REVIEW_OF_A_RANGE,],
            },);
            expect(await readQueueOfRuns({ queueFile, },),).toEqual([A_REVIEW_OF_A_RANGE,],);
            expect(await readdir(folder.path,),).toEqual(['queue.json',],);
          },
        },),
      ],
    },),
    describe({
      name: takeAQueueEntryOut.name,
      children: [
        it({
          name: 'takes every entry of that name out, keeping the order of the rest',
          fn: async () => {
            await using folder = await makeFolder();
            /**
             The queue the test writes.
             */
            const queueFile = await queueOf({
              folder,
              entries: [
                A_RUN,
                A_REVIEW_OF_A_RANGE,
              ],
            },);
            /**
             What taking the entry out answered.
             */
            const taken = await takeAQueueEntryOut({
              queueFile,
              name: 'a',
            },);
            expect(taken.taken,).toEqual([A_RUN,],);
            expect(taken.entries,).toEqual([A_REVIEW_OF_A_RANGE,],);
            expect(await readQueueOfRuns({ queueFile, },),).toEqual([A_REVIEW_OF_A_RANGE,],);
          },
        },),
        it({
          name: 'changes nothing when no entry has that name, which is what a filling that started it leaves',
          fn: async () => {
            await using folder = await makeFolder();
            /**
             The queue the test writes.
             */
            const queueFile = await queueOf({ folder, entries: [A_RUN,], },);
            /**
             What taking an entry out that is not there answered.
             */
            const taken = await takeAQueueEntryOut({
              queueFile,
              name: 'gone',
            },);
            expect(taken.taken,).toEqual([],);
            expect(taken.entries,).toEqual([A_RUN,],);
            expect(await readFile(queueFile, 'utf8',),).toBe(`${JSON.stringify([A_RUN,],)}\n`,);
          },
        },),
      ],
    },),
    describe({
      name: putAQueueEntryIn.name,
      children: [
        it({
          name: 'puts an entry at the head, and at a position past the tail, which is the tail',
          fn: async () => {
            await using folder = await makeFolder();
            /**
             The queue the test writes.
             */
            const queueFile = await queueOf({ folder, entries: [A_RUN,], },);
            expect((await putAQueueEntryIn({
              queueFile,
              entry: A_REVIEW_OF_A_RANGE,
              position: 0,
            },)).position,).toBe(0,);
            expect((await readQueueOfRuns({ queueFile, },)).map(function nameOf(entry,) {
              return entry.name;
            },),).toEqual(['review-b', 'a',],);
            expect((await putAQueueEntryIn({
              queueFile,
              entry: { name: 'c', branch: 'feat/c', },
              position: 99,
            },)).position,).toBe(2,);
          },
        },),
        it({
          name: 'refuses an entry of a name the queue holds, because two runs of one name collide everywhere the name goes',
          fn: async () => {
            await using folder = await makeFolder();
            /**
             The queue the test writes.
             */
            const queueFile = await queueOf({ folder, entries: [A_RUN,], },);
            /**
             What queueing the same name threw.
             */
            let caught: unknown;
            try {
              await putAQueueEntryIn({
                queueFile,
                entry: A_RUN,
                position: 0,
              },);
            }
            catch (error) {
              caught = error;
            }
            expect(caught,).toBeInstanceOf(QueueFileError,);
            expect(String(caught,),).toContain('already in the queue',);
            expect(await readQueueOfRuns({ queueFile, },),).toHaveLength(1,);
          },
        },),
      ],
    },),
    describe({
      name: moveAQueueEntryTo.name,
      children: [
        it({
          name: 'moves the entry of one name to a position counted from the head',
          fn: async () => {
            await using folder = await makeFolder();
            /**
             The queue the test writes.
             */
            const queueFile = await queueOf({
              folder,
              entries: [
                A_RUN,
                A_REVIEW_OF_A_RANGE,
                { name: 'c', branch: 'feat/c', },
              ],
            },);
            expect(await moveAQueueEntryTo({
              queueFile,
              name: 'c',
              position: 0,
            },),).toEqual({
              from: 2,
              to: 0,
              count: 3,
            },);
            expect((await readQueueOfRuns({ queueFile, },)).map(function nameOf(entry,) {
              return entry.name;
            },),).toEqual(['c', 'a', 'review-b',],);
          },
        },),
        it({
          name: 'answers that no entry of that name is there, which is what a filling that started it leaves',
          fn: async () => {
            await using folder = await makeFolder();
            /**
             The queue the test writes.
             */
            const queueFile = await queueOf({ folder, entries: [A_RUN,], },);
            expect((await moveAQueueEntryTo({
              queueFile,
              name: 'gone',
              position: 0,
            },)).from,).toBeUndefined();
            expect(await readQueueOfRuns({ queueFile, },),).toEqual([A_RUN,],);
          },
        },),
      ],
    },),
    describe({
      name: shapeOfAQueueEntry.name,
      children: [
        ...([
          ['a fresh run', A_RUN,],
          ['a resume of a finished run', { name: 'd', branch: 'fix/d', sessionOf: 'd-3', prompt: '/runs/d-prompt.md', wall: 28_800, },],
          ['a run on a provider and model of its own', { name: 'e', branch: 'feat/e', provider: 'qwen-token-plan-individual', model: 'qwen4-max-preview', wall: 21_600, },],
        ] as const).map(function asARunTest([what, entry,],) {
          return it({
            name: `reads ${what} as a run of a batch`,
            fn: async () => {
              /**
               What the shape reader answered.
               */
              const shape = shapeOfAQueueEntry({ record: entry, },);
              expect(shape.kind,).toBe('a-run',);
              expect((shape as Extract<ShapeOfAQueueEntry, { readonly kind: 'a-run' }>).entry,).toEqual(entry,);
            },
          },);
        },),
        it({
          name: 'reads a review of a range as one, with the provider and model the package names once',
          fn: async () => {
            /**
             What the shape reader answered.
             */
            const shape = shapeOfAQueueEntry({ record: A_REVIEW_OF_A_RANGE, },);
            expect(shape.kind,).toBe('a-review-of-a-range',);
            expect(shapeOfAQueueEntry({ record: A_REVIEW_OF_A_RANGE, },).kind,).toBe('a-review-of-a-range',);
            expect(poolOfAQueueEntry({ record: A_REVIEW_OF_A_RANGE, },),).toHaveProperty('holds', 'a-pool',);
          },
        },),
        it({
          name: 'refuses an entry that names a field its shape does not read, rather than dropping the field',
          fn: async () => {
            /**
             What the shape reader answered.
             */
            const shape = shapeOfAQueueEntry({
              record: {
                ...A_RUN,
                reviewer: 'someone',
              },
            },);
            expect(shape.kind,).toBe('unreadable',);
            expect((shape as Extract<ShapeOfAQueueEntry, { readonly kind: 'unreadable' }>).reason,).toContain('reviewer',);
          },
        },),
        it({
          name: 'refuses a review of a range that names a field the review shape does not read',
          fn: async () => {
            /**
             What the shape reader answered.
             */
            const shape = shapeOfAQueueEntry({
              record: {
                ...A_REVIEW_OF_A_RANGE,
                wall: 60,
              },
            },);
            expect(shape.kind,).toBe('unreadable',);
            expect((shape as Extract<ShapeOfAQueueEntry, { readonly kind: 'unreadable' }>).reason,).toContain('wall',);
          },
        },),
        it({
          name: 'refuses a run that names no branch, and one whose wall is no number',
          fn: async () => {
            expect(shapeOfAQueueEntry({ record: { name: 'a', prompt: '/runs/a-prompt.md', }, },).kind,).toBe('unreadable',);
            expect(shapeOfAQueueEntry({ record: { ...A_RUN, wall: 'soon', }, },).kind,).toBe('unreadable',);
          },
        },),
        it({
          name: 'refuses a review of a range that names no source folder',
          fn: async () => {
            /**
             What the shape reader answered.
             */
            const shape = shapeOfAQueueEntry({
              record: {
                name: 'review-b',
                reviewOfRange: 'main...353d5e06d7',
                commit: '353d5e06d7',
              },
            },);
            expect(shape.kind,).toBe('unreadable',);
            expect((shape as Extract<ShapeOfAQueueEntry, { readonly kind: 'unreadable' }>).reason,).toContain('source',);
          },
        },),
      ],
    },),
    describe({
      name: poolOfAQueueEntry.name,
      children: [
        it({
          name: 'reads a run that names no pool as the default run it is, and a run that names one as that pool',
          fn: async () => {
            expect(poolOfAQueueEntry({ record: A_RUN, },),).toHaveProperty('holds', 'a-pool',);
            expect(poolOfAQueueEntry({ record: { ...A_RUN, provider: 'nobody', model: 'nothing', }, },),).toHaveProperty('holds', 'no-pool',);
          },
        },),
      ],
    },),
    describe({
      name: poolAPlaceAskedForWouldHold.name,
      children: [
        it({
          name: 'counts a place asked for by a run naming no pool in every pool it could fill, until its own unit says which',
          fn: async () => {
            expect(poolAPlaceAskedForWouldHold({ record: A_RUN, },),).toHaveProperty('holds', 'some-pools',);
            expect(poolAPlaceAskedForWouldHold({ record: A_REVIEW_OF_A_RANGE, },),).toHaveProperty('holds', 'a-pool',);
          },
        },),
      ],
    },),
    describe({
      name: resumesAQueueEntry.name,
      children: [
        it({
          name: 'says whether an entry goes on from a finished run',
          fn: async () => {
            expect(resumesAQueueEntry({ record: A_RUN, },),).toBe(false,);
            expect(resumesAQueueEntry({ record: { ...A_RUN, sessionOf: 'a-1', }, },),).toBe(true,);
          },
        },),
      ],
    },),
    describe({
      name: changeOfAReviewRun.name,
      children: [
        it({
          name: 'reads the change a review run name is about, and leaves a name that is no review whole',
          fn: async () => {
            expect(changeOfAReviewRun({ run: 'review-rewrite-the-gate', },),).toBe('rewrite-the-gate',);
            expect(changeOfAReviewRun({ run: 'rewrite-the-gate', },),).toBe('rewrite-the-gate',);
          },
        },),
      ],
    },),
  ],
},);

/**
 Writes a queue file holding the text given, which a test may make no JSON array of.

 @param request - where the queue goes and what text it holds

 @returns the queue's path
 */
async function queueFileWithText({
  folder,
  text,
}: {
  readonly folder: AThrowawayFolder;
  readonly text: string;
},): Promise<string> {
  /**
   The queue's path.
   */
  const queueFile = join(folder.path, 'queue.json',);
  await writeFile(queueFile, text,);
  return queueFile;
}
