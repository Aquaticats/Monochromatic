/**
 Who reviews a change, and what a review of one is called: the provider and model every review runs on, the names of its run,
 branch and folder, and the files a review writes.

 The provider and model are named here alone, so a review of another model is one edit, and no command, option, run, branch or
 folder carries a model's name: a review of the change `x` is the run `review-x`, on branch `test/review-x`, in the folder
 `<reviews folder>/x`.

 @module
 */

import { join, } from 'node:path';

/**
 The provider every review of a change runs on.

 The owner, 2026-11-11T19:27-05:00: "Make the Qwen 4 pool 18, and use it for reviews INSTEAD of Kimi." So reviews go to the
 qwen plan's `qwen4-max-preview` pool, whose provider and model are this pair.
 */
export const REVIEWING_PROVIDER = 'qwen-token-plan-individual';

/**
 The model every review of a change runs on, which only the qwen plan's `qwen-token-plan` pool takes.
 */
export const REVIEWING_MODEL = 'qwen4-max-preview';

/**
 The start of the run name of a review, so a review of the change `x` is the run `review-x`.
 */
export const REVIEW_RUN_PREFIX = 'review-';

/**
 The start of the branch name a review works on, so a review of the change `x` is on branch `test/review-x`.
 */
export const REVIEW_BRANCH_PREFIX = 'test/review-';

/**
 The parts below the home directory of whoever runs a command where the Pi runs live, which is the same folder every command of
 this package writes runs into.
 */
export const DEFAULT_RUNS_FOLDER_IN_HOME: readonly string[] = [
  'temp',
  'agent',
  'pi-runs',
];

/**
 The parts below the home directory of whoever runs a command where the reviews live. The name says what the folder holds and no
 model's name, so a review of another model needs no new folder.
 */
export const DEFAULT_REVIEWS_FOLDER_IN_HOME: readonly string[] = [
  'temp',
  'agent',
  'change-reviews',
];

/**
 The file a review writes the change under review to.
 */
export const CHANGE_DIFF_FILE_NAME = 'change.diff';

/**
 The folder a review puts its reproduction scripts in.
 */
export const REPRODUCTIONS_FOLDER_NAME = 'reproductions';

/**
 The file a review's brief is written to, inside the review's own folder.
 */
export const REVIEW_BRIEF_FILE_NAME = 'brief.md';

/**
 The file a review's findings are read from, which a review that ends without one has its last message saved as.
 */
export const REVIEW_FILE_NAME = 'review.md';

/**
 Reads the change name a branch is reviewed as: the branch without the type that says what kind of work it is.

 @param branch - the branch, as `fix/rewrite-the-gate` or `feat/a-batch-in-a-unit`

 @returns the change's name, which is the branch without its first path segment

 @example
 ```ts
 const change = changeNameOfBranch({ branch: 'fix/rewrite-the-gate', },);
 ```
 */
export function changeNameOfBranch({ branch, }: { readonly branch: string },): string {
  /**
   Where the type ends.
   */
  const slash = branch.indexOf('/',);
  return (slash === (-1))
    ? branch
    : branch.slice(slash + 1,);
}

/**
 Reads the run name of a review.

 @param review - the change the review reads

 @returns the run's name

 @example
 ```ts
 const run = runNameOfAReview({ review: 'rewrite-the-gate', },);
 ```
 */
export function runNameOfAReview({ review, }: { readonly review: string },): string {
  return `${REVIEW_RUN_PREFIX}${review}`;
}

/**
 Reads the branch name a review's worktree works on.

 @param review - the change the review reads

 @returns the branch's name

 @example
 ```ts
 const branch = branchOfAReview({ review: 'rewrite-the-gate', },);
 ```
 */
export function branchOfAReview({ review, }: { readonly review: string },): string {
  return `${REVIEW_BRANCH_PREFIX}${review}`;
}

/**
 Reads the folder one review keeps its own files in.

 @param reviewsFolder - folder that holds one folder per review

 @param review - the change the review reads

 @returns the review's folder

 @example
 ```ts
 const folder = reviewFolderOf({ reviewsFolder: '/reviews', review: 'rewrite-the-gate', },);
 ```
 */
export function reviewFolderOf({
  reviewsFolder,
  review,
}: {
  readonly reviewsFolder: string;
  readonly review: string;
}): string {
  return join(
    reviewsFolder,
    review,
  );
}

/**
 Reads the file a review's brief is written to.

 @param reviewsFolder - folder that holds one folder per review

 @param review - the change the review reads

 @returns the brief's path

 @example
 ```ts
 const brief = briefFileOfAReview({ reviewsFolder: '/reviews', review: 'rewrite-the-gate', },);
 ```
 */
export function briefFileOfAReview({
  reviewsFolder,
  review,
}: {
  readonly reviewsFolder: string;
  readonly review: string;
}): string {
  return join(
    reviewFolderOf({
      reviewsFolder,
      review,
    },),
    REVIEW_BRIEF_FILE_NAME,
  );
}

/**
 Reads the file a review's change is written to.

 @param reviewsFolder - folder that holds one folder per review

 @param review - the change the review reads

 @returns the diff's path

 @example
 ```ts
 const diff = diffFileOfAReview({ reviewsFolder: '/reviews', review: 'rewrite-the-gate', },);
 ```
 */
export function diffFileOfAReview({
  reviewsFolder,
  review,
}: {
  readonly reviewsFolder: string;
  readonly review: string;
}): string {
  return join(
    reviewFolderOf({
      reviewsFolder,
      review,
    },),
    CHANGE_DIFF_FILE_NAME,
  );
}

/**
 Reads the folder a review puts its reproduction scripts in.

 @param reviewsFolder - folder that holds one folder per review

 @param review - the change the review reads

 @returns the folder's path

 @example
 ```ts
 const scripts = reproductionsFolderOfAReview({ reviewsFolder: '/reviews', review: 'rewrite-the-gate', },);
 ```
 */
export function reproductionsFolderOfAReview({
  reviewsFolder,
  review,
}: {
  readonly reviewsFolder: string;
  readonly review: string;
}): string {
  return join(
    reviewFolderOf({
      reviewsFolder,
      review,
    },),
    REPRODUCTIONS_FOLDER_NAME,
  );
}

/**
 Reads the file a review writes its findings to.

 @param reviewsFolder - folder that holds one folder per review

 @param review - the change the review reads

 @returns the review file's path

 @example
 ```ts
 const file = reviewFileOfAReview({ reviewsFolder: '/reviews', review: 'rewrite-the-gate', },);
 ```
 */
export function reviewFileOfAReview({
  reviewsFolder,
  review,
}: {
  readonly reviewsFolder: string;
  readonly review: string;
}): string {
  return join(
    reviewFolderOf({
      reviewsFolder,
      review,
    },),
    REVIEW_FILE_NAME,
  );
}
