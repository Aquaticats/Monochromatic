/**
 The folders one change's reviews keep: a start makes the first free name and owns it for good, a second start takes the next name
 beside it and leaves the first as it is, and the starter finds a branch's review by the record each folder holds, not by a name
 alone.

 Every folder here is one the test made and throws away.

 @module
 */

import { existsSync, } from 'node:fs';
import {
  mkdir,
  mkdtemp,
  readFile,
  rm,
  writeFile,
} from 'node:fs/promises';
import { dirname, join, } from 'node:path';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';
import {
  NOT_A_GENERATION,
  NO_REVIEWED_FOLDER,
  generationInAName,
  recordFileOfAReview,
  recordTextOf,
  reviewFileOfAReview,
  reviewFolderOf,
  reviewNameOf,
  takeTheNextReviewFolder,
  theReviewOfABranch,
} from '../dist/final/node/index.mjs';

/**
 The change name every folder here is for.
 */
const REVIEW = 'shared-change';

/**
 A branch of that change.
 */
const BRANCH = 'fix/shared-change';

/**
 A folder the test throws away.
 */
async function makeFolder(): Promise<{
  readonly path: string;
  readonly [Symbol.asyncDispose]: () => Promise<void>
}> {
  /**
   The folder.
   */
  const path = await mkdtemp(join(scratchRoot(), 'pi-review-folders-',),);
  return {
    path,
    async [Symbol.asyncDispose]() {
      await rm(path, { recursive: true, force: true, },);
    },
  };
}

/**
 Writes what a folder records of itself.

 @param reviewsFolder - folder that holds one folder per review

 @param review - the review name the folder carries

 @param branch - the branch the record names

 @returns the record file's path
 */
async function writeRecord({
  reviewsFolder,
  review,
  branch,
}: {
  readonly reviewsFolder: string;
  readonly review: string;
  readonly branch: string;
},): Promise<string> {
  /**
   The record file.
   */
  const recordFile = recordFileOfAReview({ reviewsFolder, review, },);
  await mkdir(dirname(recordFile,), { recursive: true, },);
  await writeFile(
    recordFile,
    recordTextOf({
      branch,
      commit: 'abc1234',
      ranges: [`main...${branch}`,],
      source: 'package/a/src',
      claimedAt: '2026-11-15T00:00:00.000Z',
    },),
    'utf8',
  );
  return recordFile;
}

await describe({
  name: '',
  children: [
    describe({
      name: takeTheNextReviewFolder.name,
      children: [
        it({
          name: 'makes the change\'s own folder first, then the next free name beside it, and never touches a folder it did not make',
          fn: async () => {
            await using folder = await makeFolder();
            const reviewsFolder = join(folder.path, 'reviews',);
            const first = await takeTheNextReviewFolder({ reviewsFolder, review: REVIEW, },);
            expect(first.review,).toBe(REVIEW,);
            await writeFile(join(first.folder, 'a-file-of-the-first-start',), 'kept\n', 'utf8',);
            const second = await takeTheNextReviewFolder({ reviewsFolder, review: REVIEW, },);
            expect(second.review,).toBe(`${REVIEW}-2`,);
            const third = await takeTheNextReviewFolder({ reviewsFolder, review: REVIEW, },);
            expect(third.review,).toBe(`${REVIEW}-3`,);
            expect(existsSync(second.folder,),).toBe(true,);
            expect(existsSync(third.folder,),).toBe(true,);
            expect(await readFile(join(first.folder, 'a-file-of-the-first-start',), 'utf8',),).toBe('kept\n',);
          },
        },),
      ],
    },),
    describe({
      name: generationInAName.name,
      children: [
        it({
          name: 'reads the change\'s own name and the names beside it as generations, and no other name as one',
          fn: async () => {
            expect(generationInAName({ review: REVIEW, name: REVIEW, },),).toBe(0,);
            expect(generationInAName({ review: REVIEW, name: `${REVIEW}-1`, },),).toBe(NOT_A_GENERATION,);
            expect(generationInAName({ review: REVIEW, name: `${REVIEW}-2`, },),).toBe(2,);
            expect(generationInAName({ review: REVIEW, name: `${REVIEW}-10`, },),).toBe(10,);
            expect(generationInAName({ review: REVIEW, name: `${REVIEW}-0`, },),).toBe(NOT_A_GENERATION,);
            expect(generationInAName({ review: REVIEW, name: `${REVIEW}-two`, },),).toBe(NOT_A_GENERATION,);
            expect(generationInAName({ review: REVIEW, name: `${REVIEW}-2-3`, },),).toBe(NOT_A_GENERATION,);
            expect(generationInAName({ review: REVIEW, name: 'another-change', },),).toBe(NOT_A_GENERATION,);
          },
        },),
        it({
          name: 'names the first generation after the change and later ones after it, so a run, a branch and a unit all carry the generation',
          fn: async () => {
            expect(reviewNameOf({ review: REVIEW, generation: 0, },),).toBe(REVIEW,);
            expect(reviewNameOf({ review: REVIEW, generation: 2, },),).toBe(`${REVIEW}-2`,);
          },
        },),
      ],
    },),
    describe({
      name: theReviewOfABranch.name,
      children: [
        it({
          name: 'finds the newest folder of this branch that wrote its review file, through a newer leftover',
          fn: async () => {
            await using folder = await makeFolder();
            const reviewsFolder = join(folder.path, 'reviews',);
            const base = reviewFolderOf({ reviewsFolder, review: REVIEW, },);
            const second = reviewFolderOf({ reviewsFolder, review: `${REVIEW}-2`, },);
            const third = reviewFolderOf({ reviewsFolder, review: `${REVIEW}-3`, },);
            await mkdir(base, { recursive: true, },);
            await mkdir(second, { recursive: true, },);
            await mkdir(third, { recursive: true, },);
            await writeRecord({ reviewsFolder, review: REVIEW, branch: BRANCH, },);
            await writeRecord({ reviewsFolder, review: `${REVIEW}-2`, branch: BRANCH, },);
            await writeRecord({ reviewsFolder, review: `${REVIEW}-3`, branch: BRANCH, },);
            await writeFile(join(second, 'review.md',), 'the review\n', 'utf8',);
            expect(await theReviewOfABranch({ reviewsFolder, review: REVIEW, branch: BRANCH, },),).toBe(`${REVIEW}-2`,);
          },
        },),
        it({
          name: 'leaves out a folder whose record names another branch, even when its name is a generation of this change',
          fn: async () => {
            await using folder = await makeFolder();
            const reviewsFolder = join(folder.path, 'reviews',);
            const base = reviewFolderOf({ reviewsFolder, review: REVIEW, },);
            const second = reviewFolderOf({ reviewsFolder, review: `${REVIEW}-2`, },);
            await mkdir(base, { recursive: true, },);
            await mkdir(second, { recursive: true, },);
            await writeRecord({ reviewsFolder, review: `${REVIEW}-2`, branch: 'fix/shared-change-2', },);
            await writeFile(join(second, 'review.md',), 'the other change\n', 'utf8',);
            expect(await theReviewOfABranch({ reviewsFolder, review: REVIEW, branch: BRANCH, },),).toBe(NO_REVIEWED_FOLDER,);
            expect(await theReviewOfABranch({
              reviewsFolder,
              review: `${REVIEW}-2`,
              branch: 'fix/shared-change-2',
            },),).toBe(`${REVIEW}-2`,);
          },
        },),
        it({
          name: 'says nothing written when the branch\'s folders are all leftovers without a review file',
          fn: async () => {
            await using folder = await makeFolder();
            const reviewsFolder = join(folder.path, 'reviews',);
            await mkdir(reviewFolderOf({ reviewsFolder, review: REVIEW, },), { recursive: true, },);
            await writeRecord({ reviewsFolder, review: REVIEW, branch: BRANCH, },);
            expect(await theReviewOfABranch({ reviewsFolder, review: REVIEW, branch: BRANCH, },),).toBe(NO_REVIEWED_FOLDER,);
            expect(
              existsSync(reviewFileOfAReview({ reviewsFolder, review: REVIEW, },),),
            ).toBe(false,);
          },
        },),
      ],
    },),
  ],
},);
