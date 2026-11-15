/**
 One review's generation, written whole or not at all: the three files appear together in the review's folder, or the folder is not
 written into at all.

 @module
 */

import {
  mkdir,
  mkdtemp,
  readdir,
  readFile,
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
import { putAGenerationInPlace, } from '../dist/final/node/index.mjs';

/**
 The code the operating system gives a path that is not there.
 */
const MISSING_PATH_CODE = 'ENOENT';

/**
 The change every generation here is written for.

 @returns the change's name

 @example
 ```ts
 const review = aReview();
 ```
 */
function aReview(): string {
  return 'rewrite-the-gate';
}

/**
 One generation, with what each of its three files says.

 @returns the generation

 @example
 ```ts
 const generation = aGeneration();
 ```
 */
function aGeneration(): {
  readonly brief: string;
  readonly diff: string;
  readonly record: string;
} {
  return {
    brief: 'the brief of the review\n',
    diff: 'the diff of the change\n',
    record: '{"review":"a"}\n',
  };
}

/**
 A folder a test throws away.

 @returns the folder, and how to remove it

 @example
 ```ts
 await using folder = await makeFolder();
 ```
 */
async function makeFolder(): Promise<{
  readonly path: string;
  readonly [Symbol.asyncDispose]: () => Promise<void>
}> {
  /**
   The folder.
   */
  const path = await mkdtemp(join(scratchRoot(), 'pi-review-generation-',),);
  return {
    path,
    async [Symbol.asyncDispose]() {
      await rm(path, { recursive: true, force: true, },);
    },
  };
}

/**
 Reads what a folder holds, so a test can tell a generation from a folder with something of it in.

 @param folder - the folder to list

 @returns its entries' names, sorted

 @example
 ```ts
 const entries = await entriesIn('/reviews');
 ```
 */
async function entriesIn(folder: string,): Promise<readonly string[]> {
  return (await readdir(folder,)).toSorted();
}

await describe({
  name: '',
  children: [
    describe({
      name: putAGenerationInPlace.name,
      children: [
        it({
          name: 'writes the three files and the reproductions folder together under the review\'s name, and leaves no name of its own behind',
          fn: async () => {
            await using folder = await makeFolder();
            /**
             The folder that holds one folder per review.
             */
            const reviewsFolder = join(folder.path, 'reviews',);
            expect(await putAGenerationInPlace({
              reviewsFolder,
              review: aReview(),
              generation: aGeneration(),
            },),).toEqual({ kind: 'in-place', },);
            expect(await entriesIn(reviewsFolder,),).toEqual([aReview(),],);
            expect(
              await entriesIn(join(reviewsFolder, aReview(),),),
            )
              .toEqual(['brief.md', 'change.diff', 'reproductions', 'what-the-review-read.json',],);
            expect(await readFile(join(reviewsFolder, aReview(), 'brief.md',), 'utf8',),)
              .toBe(aGeneration().brief,);
          },
        },),
        it({
          name: 'never writes into a review folder that is already there, and answers with every name it holds',
          fn: async () => {
            await using folder = await makeFolder();
            /**
             The folder that holds one folder per review.
             */
            const reviewsFolder = join(folder.path, 'reviews',);
            /**
             The review's folder, with a brief and a file this writer did not make.
             */
            const reviewFolder = join(reviewsFolder, aReview(),);
            await mkdir(reviewFolder, { recursive: true, },);
            await writeFile(join(reviewFolder, 'brief.md',), 'an earlier brief\n', 'utf8',);
            await writeFile(join(reviewFolder, 'owner.json',), '{"unit":"a"}\n', 'utf8',);
            expect(await putAGenerationInPlace({
              reviewsFolder,
              review: aReview(),
              generation: aGeneration(),
            },),).toEqual({
              kind: 'already-there',
              held: ['brief.md', 'owner.json',],
            },);
            // Nothing of the folder is replaced or removed, and no dot name of this writer is left.
            expect(await entriesIn(reviewFolder,),).toEqual(['brief.md', 'owner.json',],);
            expect(await readFile(join(reviewFolder, 'brief.md',), 'utf8',),).toBe('an earlier brief\n',);
            expect(await entriesIn(reviewsFolder,),).toEqual([aReview(),],);
          },
        },),
        it({
          name: 'leaves no review standing when a file of the generation cannot be written, and a retry writes the whole generation',
          fn: async () => {
            await using folder = await makeFolder();
            /**
             The folder that holds one folder per review.
             */
            const reviewsFolder = join(folder.path, 'reviews',);
            await expect(putAGenerationInPlace({
              reviewsFolder,
              review: aReview(),
              generation: aGeneration(),
              write: async function cutShort({ file, },) {
                await writeFile(file, 'a brief that was cut\n', 'utf8',);
                throw new Error('the writer was killed after its first file',);
              },
            },),).rejects.toThrow('the writer was killed after its first file',);
            // The review's name is not taken, and the dot name the writer wrote under is not a review: no reader looks for one there.
            expect(await entriesIn(reviewsFolder,),).toEqual([],);
            expect(await putAGenerationInPlace({
              reviewsFolder,
              review: aReview(),
              generation: aGeneration(),
            },),).toEqual({ kind: 'in-place', },);
            expect(await readFile(join(reviewsFolder, aReview(), 'brief.md',), 'utf8',),)
              .toBe(aGeneration().brief,);
            expect(await readFile(join(reviewsFolder, aReview(), 'what-the-review-read.json',), 'utf8',),)
              .toBe(aGeneration().record,);
          },
        },),
        it({
          name: 'writes no review folder for a folder that holds nothing',
          fn: async () => {
            await using folder = await makeFolder();
            /**
             The folder that holds one folder per review, with the review's folder already made and empty.
             */
            const reviewsFolder = join(folder.path, 'reviews',);
            await mkdir(join(reviewsFolder, aReview(),), { recursive: true, },);
            expect(await putAGenerationInPlace({
              reviewsFolder,
              review: aReview(),
              generation: aGeneration(),
            },),).toEqual({ kind: 'in-place', },);
            expect(
              await entriesIn(join(reviewsFolder, aReview(),),),
            )
              .toEqual(['brief.md', 'change.diff', 'reproductions', 'what-the-review-read.json',],);
          },
        },),
      ],
    },),
  ],
},);
