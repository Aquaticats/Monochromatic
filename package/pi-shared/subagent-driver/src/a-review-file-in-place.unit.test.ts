/**
 Putting a review's text in place: whole or not at all, never replacing a review another writer put there first.

 @module
 */

import {
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
import { putAReviewInPlace, } from '../dist/final/node/index.mjs';

/**
 The code the operating system gives a path that is not there.
 */
const MISSING_PATH_CODE = 'ENOENT';

/**
 The name a review is read from, inside its own folder.

 @returns the file's path, from a folder the test names

 @example
 ```ts
 const file = theReviewFile('/reviews/a');
 ```
 */
function theReviewFile(folder: string,): string {
  return join(folder, 'review.md',);
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
  const path = await mkdtemp(join(scratchRoot(), 'pi-review-in-place-',),);
  return {
    path,
    async [Symbol.asyncDispose]() {
      await rm(path, { recursive: true, force: true, },);
    },
  };
}

/**
 Says what a folder holds, so a test can tell a temporary name left behind from a clean one.

 @param folder - the folder to list

 @returns its entries' names

 @example
 ```ts
 const entries = await entriesIn('/reviews/a');
 ```
 */
async function entriesIn(folder: string,): Promise<readonly string[]> {
  return await readdir(folder,);
}

await describe({
  name: '',
  children: [
    describe({
      name: putAReviewInPlace.name,
      children: [
        it({
          name: 'writes the whole text as the review when nothing is there, and leaves no name of its own behind',
          fn: async () => {
            await using folder = await makeFolder();
            expect(await putAReviewInPlace({
              folder: folder.path,
              reviewFile: theReviewFile(folder.path,),
              text: 'a whole review\n',
            },),).toEqual({ kind: 'in-place', },);
            expect(await readFile(theReviewFile(folder.path,), 'utf8',),).toBe('a whole review\n',);
            expect(await entriesIn(folder.path,),).toEqual(['review.md',],);
          },
        },),
        it({
          name: 'keeps the review another writer put there first, and puts none of its own text in place',
          fn: async () => {
            await using folder = await makeFolder();
            await writeFile(theReviewFile(folder.path,), 'the first writer\'s review\n', 'utf8',);
            expect(await putAReviewInPlace({
              folder: folder.path,
              reviewFile: theReviewFile(folder.path,),
              text: 'the second writer\'s review\n',
            },),).toEqual({ kind: 'already-there', },);
            // The second writer held its text and put none of it in place: the review is the first writer's, whole.
            expect(await readFile(theReviewFile(folder.path,), 'utf8',),).toBe('the first writer\'s review\n',);
            expect(await entriesIn(folder.path,),).toEqual(['review.md',],);
          },
        },),
        it({
          name: 'leaves no review standing when the write fails after its first bytes, and a retry puts the whole text in place',
          fn: async () => {
            await using folder = await makeFolder();
            /**
             The first bytes every failing write puts down before it fails.
             */
            const firstBytes = 'a review that was cut\n';
            await expect(putAReviewInPlace({
              folder: folder.path,
              reviewFile: theReviewFile(folder.path,),
              text: 'a review of the whole change\n',
              write: async function cutShort({ file, },) {
                await writeFile(file, firstBytes, 'utf8',);
                throw new Error('the writer was killed after its first bytes',);
              },
            },),).rejects.toThrow('the writer was killed after its first bytes',);
            // Nothing stands that a reader would take for a review, and no half-written name is left for a retry to trip over.
            await expect(
              readFile(theReviewFile(folder.path,),),
            ).rejects.toThrow(MISSING_PATH_CODE,);
            expect(await entriesIn(folder.path,),).toEqual([],);
            expect(await putAReviewInPlace({
              folder: folder.path,
              reviewFile: theReviewFile(folder.path,),
              text: 'a review of the whole change\n',
            },),).toEqual({ kind: 'in-place', },);
            expect(await readFile(theReviewFile(folder.path,), 'utf8',),).toBe('a review of the whole change\n',);
          },
        },),
      ],
    },),
  ],
},);
