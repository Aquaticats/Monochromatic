/**
 Writes what a review of one change reads: the diff of its ranges under the reviewed source folder, the brief that names its
 files, and the record of what the review was given, as one generation in a folder of its own.

 A change that leaves nothing to read under the source folder writes nothing and answers one line, because a reviewer given no
 file reports nothing, which reads as a clean review although nothing was reviewed. What a change leaves to read is the code
 under that folder and the files that decide how that code is built, checked or run.

 The three files are written whole or not at all (`src/a-review-generation-in-place.ts`), and a review folder that is already there
 is never written into: the caller is told what it holds, because a folder that holds a brief may be a review a reviewer is reading
 or one that was killed before its verdict.

 @module
 */

import type { BriefOfOneReview } from './reviews-of-finished-branches.ts';
import {
  type GitQuestion,
  type ReviewReading,
  materialOfAReview,
} from './material-of-a-review.ts';
import { putAGenerationInPlace, } from './a-review-generation-in-place.ts';
import { briefOfAReview, } from './review-brief.ts';
import {
  briefFileOfAReview,
  reviewFolderOf,
  runNameOfAReview,
} from './reviewer-of-a-change.ts';

/**
 Writes the diff, the brief, the record of what the review was given and the reproductions folder of one review, as one generation.

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

 @returns whether the brief was written, and the line to print; a review folder that is already there is answered with what it
 holds and nothing is written into it

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
   The diff, the files and the ranges as the one commit under review reads them.
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
      kind: 'nothing-to-review',
      line: `nothing to review under ${source} in ${material.ranges
        .join(' and ',)}`,
    };
  }
  /**
   The folder the review's own files are read from.
   */
  const reviewFolder = reviewFolderOf({
    reviewsFolder,
    review,
  },);
  /**
   The three files of this review, which are written whole or not at all.
   */
  const generation = {
    brief: briefOfAReview({
      review,
      commit: material.commit,
      ranges: material.ranges,
      source,
      files: material.files,
      focus,
      reviewsFolder,
      runsFolder,
      rulesFile,
    },),
    diff: material.diff,
    record: `${JSON.stringify(
      {
        review,
        commit: material.commit,
        ranges: material.ranges,
        source,
        files: material.files,
      } satisfies ReviewReading,
      undefined,
      2
    )}\n`,
  };
  try {
    /**
     What putting the three files in place answered.
     */
    const put = await putAGenerationInPlace({
      reviewsFolder,
      review,
      generation,
    },);
    if (put.kind === 'already-there') {
      return {
        kind: 'not-written',
        line: `${runNameOfAReview({ review, },)}: ${reviewFolder} already holds `
          + `${put.held
            .toSorted()
            .join(', ',)}, so no brief was written`,
      };
    }
  }
  catch (error) {
    // A brief whose files could not be written is a review that did not happen, and the line says which folder it was for and
    // what went wrong, so the command that reads it can count it rather than exit 0.
    return {
      kind: 'not-written',
      line: `${runNameOfAReview({ review, },)}: ${reviewFolder} could not be written: ${
        String(error,)
      }`,
    };
  }
  return {
    kind: 'written',
    line: `${runNameOfAReview({ review, },)}: ${String(material.files
      .length,)} files to read, brief at ${briefFileOfAReview({
      reviewsFolder,
      review,
    },)}; its worktree starts at ${material.commit}. Its folder is ${reviewFolder}`,
  };
}
