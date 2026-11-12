/**
 Writes what a review of one change reads: the diff of its ranges under the reviewed source folder, the brief that names its
 files, and the folder its reproductions go in.

 A change that leaves no code under the source folder writes nothing and answers one line, because a reviewer given no file
 reports nothing, which reads as a clean review although nothing was reviewed.

 @module
 */

import {
  mkdir,
  writeFile,
} from 'node:fs/promises';
import { dirname, } from 'node:path';
import type { BriefOfOneReview } from './reviews-of-finished-branches.ts';
import {
  type GitQuestion,
  materialOfAReview,
} from './material-of-a-review.ts';
import { briefOfAReview, } from './review-brief.ts';
import {
  briefFileOfAReview,
  diffFileOfAReview,
  reproductionsFolderOfAReview,
  reviewFolderOf,
  runNameOfAReview,
} from './reviewer-of-a-change.ts';

/**
 Writes the diff, the brief and the reproductions folder of one review.

 @param git - how one git question is run

 @param checkoutFolder - the checkout the diff is read from

 @param review - the change the review reads

 @param commit - the commit under review

 @param ranges - what changed, each as `git diff` takes one

 @param source - the folder whose changes are reviewed

 @param focus - what the reviewer looks hardest at

 @param reviewsFolder - folder that holds one folder per review

 @param runsFolder - folder the driver writes its runs into

 @param rulesFile - the rules file's path, which the brief names first

 @returns whether the brief was written, and the line to print

 @throws ReviewGitError when git did not answer with a success

 @example
 ```ts
 const briefed = await writeReviewBrief({ git, checkoutFolder: '/repo', review: 'a', commit: 'abc123', ranges: ['main...fix/a'], source: 'package/a/src', focus: 'data loss', reviewsFolder: '/reviews', runsFolder: '/runs', rulesFile: '/repo/package/pi-shared/subagent-driver/doc/rules-every-pi-brief-carries.md', },);
 ```
 */
export async function writeReviewBrief({
  git,
  checkoutFolder,
  review,
  commit,
  ranges,
  source,
  focus,
  reviewsFolder,
  runsFolder,
  rulesFile,
}: {
  readonly git: GitQuestion;
  readonly checkoutFolder: string;
  readonly review: string;
  readonly commit: string;
  readonly ranges: readonly string[];
  readonly source: string;
  readonly focus: string;
  readonly reviewsFolder: string;
  readonly runsFolder: string;
  readonly rulesFile: string;
}): Promise<BriefOfOneReview> {
  /**
   The diff and the files a reviewer is given.
   */
  const material = await materialOfAReview({
    git,
    checkoutFolder,
    ranges,
    source,
    commit,
  },);
  if (material.files
    .length
    === 0) {
    return {
      written: false,
      line: `nothing to review under ${source} in ${ranges.join(' and ',)}`,
    };
  }
  /**
   The review's own folder.
   */
  const reviewFolder = reviewFolderOf({
    reviewsFolder,
    review,
  },);
  /**
   The file the diff is written to.
   */
  const diffFile = diffFileOfAReview({
    reviewsFolder,
    review,
  },);
  /**
   The file the brief is written to.
   */
  const briefFile = briefFileOfAReview({
    reviewsFolder,
    review,
  },);
  await mkdir(
    reproductionsFolderOfAReview({
      reviewsFolder,
      review,
    },),
    { recursive: true, },
  );
  await mkdir(
    dirname(diffFile,),
    { recursive: true, },
  );
  await writeFile(
    diffFile,
    material.diff,
    'utf8',
  );
  await writeFile(
    briefFile,
    briefOfAReview({
      review,
      commit,
      source,
      files: material.files,
      focus,
      reviewsFolder,
      runsFolder,
      rulesFile,
    },),
    'utf8',
  );
  return {
    written: true,
    line: `${runNameOfAReview({ review, },)}: ${String(material.files
      .length,)} files to read, brief at ${briefFile}; `
      + `its worktree starts at ${commit}. Its folder is ${reviewFolder}`,
  };
}
