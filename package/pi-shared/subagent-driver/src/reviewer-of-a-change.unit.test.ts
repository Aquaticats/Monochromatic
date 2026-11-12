/**
 Who reviews a change and what a review of one is called: the run, branch, folder and file names, and the provider and model the
 pool table gives that review. Nothing here names a model in a name, and nothing here reads a machine.

 @module
 */

import { join, } from 'node:path';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  CHANGE_DIFF_FILE_NAME,
  DEFAULT_REVIEWS_FOLDER_IN_HOME,
  REPRODUCTIONS_FOLDER_NAME,
  REVIEWING_MODEL,
  REVIEWING_PROVIDER,
  REVIEW_BRANCH_PREFIX,
  REVIEW_BRIEF_FILE_NAME,
  REVIEW_FILE_NAME,
  REVIEW_RUN_PREFIX,
  branchOfAReview,
  briefFileOfAReview,
  changeNameOfBranch,
  diffFileOfAReview,
  poolNamed,
  reproductionsFolderOfAReview,
  reviewFileOfAReview,
  reviewFolderOf,
  runNameOfAReview,
} from '../dist/final/node/index.mjs';

/**
 A change's name, which a test builds its expectations from.

 @returns the change's name

 @example
 ```ts
 const change = aChange();
 ```
 */
function aChange(): string {
  return 'rewrite-the-gate';
}

/**
 A folder that holds one folder per review.

 @returns the folder's path

 @example
 ```ts
 const folder = aReviewsFolder();
 ```
 */
function aReviewsFolder(): string {
  return '/reviews';
}

await describe({
  name: '',
  children: [
    describe({
      name: changeNameOfBranch.name,
      children: [
        it({
          name: 'reads a branch name without the type that says what kind of work it is, and a name without a type as it is',
          fn: async () => {
            expect(changeNameOfBranch({ branch: `fix/${aChange()}`, },),).toBe(aChange(),);
            expect(changeNameOfBranch({ branch: `feat/${aChange()}`, },),).toBe(aChange(),);
            expect(changeNameOfBranch({ branch: aChange(), },),).toBe(aChange(),);
          },
        },),
      ],
    },),
    describe({
      name: runNameOfAReview.name,
      children: [
        it({
          name: 'names a review of a change by the change alone, so no run name says which model reads it',
          fn: async () => {
            expect(runNameOfAReview({ review: aChange(), },),).toBe(`${REVIEW_RUN_PREFIX}${aChange()}`,);
          },
        },),
      ],
    },),
    describe({
      name: branchOfAReview.name,
      children: [
        it({
          name: 'puts a review of a change on a test branch of the change name, so no branch name says which model reads it',
          fn: async () => {
            expect(branchOfAReview({ review: aChange(), },),).toBe(`${REVIEW_BRANCH_PREFIX}${aChange()}`,);
          },
        },),
      ],
    },),
    describe({
      name: reviewFolderOf.name,
      children: [
        it({
          name: 'puts one review in its own folder below the reviews folder, and names its brief, diff, reproductions and review file there',
          fn: async () => {
            /**
             The folder the review's own files go in.
             */
            const folder = reviewFolderOf({ reviewsFolder: aReviewsFolder(), review: aChange(), },);
            expect(folder,).toBe(`${aReviewsFolder()}/${aChange()}`,);
            expect(briefFileOfAReview({ reviewsFolder: aReviewsFolder(), review: aChange(), },),)
              .toBe(join(folder, REVIEW_BRIEF_FILE_NAME,),);
            expect(diffFileOfAReview({ reviewsFolder: aReviewsFolder(), review: aChange(), },),)
              .toBe(join(folder, CHANGE_DIFF_FILE_NAME,),);
            expect(reproductionsFolderOfAReview({ reviewsFolder: aReviewsFolder(), review: aChange(), },),)
              .toBe(join(folder, REPRODUCTIONS_FOLDER_NAME,),);
            expect(reviewFileOfAReview({ reviewsFolder: aReviewsFolder(), review: aChange(), },),)
              .toBe(join(folder, REVIEW_FILE_NAME,),);
          },
        },),
      ],
    },),
    describe({
      name: poolNamed.name,
      children: [
        it({
          name: 'puts the reviewer\'s provider and model on a pool of their own that takes no default run, which is what makes a review run where the project says it does',
          fn: async () => {
            /**
             The membership the reviewer's own provider and model name.
             */
            const membership = poolNamed({
              provider: REVIEWING_PROVIDER,
              model: REVIEWING_MODEL,
            },);
            expect(membership.holds,).toBe('a-pool',);
            expect((membership.holds === 'a-pool') ? membership.pool.takesDefaultRuns : true,).toBe(false,);
          },
        },),
      ],
    },),
    describe({
      name: 'the reviews folder',
      children: [
        it({
          name: 'names no model, so a review of another model needs no new folder',
          fn: async () => {
            /**
             The folder's own name, which is the last part of its path.
             */
            const name = DEFAULT_REVIEWS_FOLDER_IN_HOME.at(-1,)
              ?? '';
            expect(name,).not.toBe('',);
            expect(name.toLowerCase(),).not.toContain('kimi',);
            expect(name.toLowerCase(),).not.toContain('qwen',);
          },
        },),
      ],
    },),
  ],
},);
