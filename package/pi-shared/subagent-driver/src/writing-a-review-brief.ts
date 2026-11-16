/**
 Writes what a review of one change reads: the diff of its ranges under the reviewed source folder, the brief that names its
 files, and the folder its reproductions go in.

 A change that leaves nothing to read under the source folder writes nothing and answers one line, because a reviewer given no
 file reports nothing, which reads as a clean review although nothing was reviewed. What a change leaves to read is the code
 under that folder and the files that decide how that code is built, checked or run, and where it changes nothing but tests and
 fixtures, those files, whose brief asks the reviewer to plant a break under each changed test and say whether it fails.

 The kind is decided from every way a range can change a path, so a range that removes or renames code gets the brief of a change
 of code, which names what was removed or renamed and asks what depended on it, even when the files the commit still holds at the
 touched paths are tests alone.

 A review's folder is made by the start that writes its inputs, by an exclusive create and only once the change is known to leave
 something to read, and it belongs to that start for good. A start that finds the change's name taken takes the next free name
 beside it, `<change>-2`, `<change>-3`, so a second start never writes over the first, and a folder a killed start left is left as a
 leftover with its record.

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
  type ReviewReading,
  materialOfAReview,
} from './material-of-a-review.ts';
import {
  briefOfAChangeToTests,
  briefOfAReview,
} from './review-brief.ts';
import {
  briefFileOfAReview,
  diffFileOfAReview,
  readingFileOfAReview,
  recordFileOfAReview,
  reproductionsFolderOfAReview,
  runNameOfAReview,
} from './reviewer-of-a-change.ts';
import { takeTheNextReviewFolder, } from './the-folder-of-a-review.ts';
import { writeRecordIn, } from './the-record-of-a-review-folder.ts';

/**
 Writes the diff, the brief and the reproductions folder of one review.

 @param git - how one git question is run

 @param checkoutFolder - the checkout the diff is read from

 @param review - the change the review reads

 @param commit - the commit under review

 @param ranges - what changed, each as `git diff` takes one

 @param source - the folder whose changes are reviewed

 @param branch - the branch whose change the review reads, when the caller names one; a second start of one change name takes a
 name of its own beside the first

 @param focus - what the reviewer looks hardest at

 @param reviewsFolder - folder that holds one folder per review

 @param runsFolder - folder the driver writes its runs into

 @param rulesFile - the rules file's path, which the brief names first

 @returns whether the brief was written, the review name its folder carries, and the line to print

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
  branch,
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
  readonly branch?: string;
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
  if (material.kind === 'nothing-a-review-reads') {
    return {
      written: false,
      line: `nothing to review under ${source} in ${ranges.join(' and ',)}`,
    };
  }
  /**
   The review's own folder, whose name carries the generation this start took, and the record that says what the folder is for.
   */
  const taken = await takeTheNextReviewFolder({
    reviewsFolder,
    review,
  },);
  /**
   The review name this start's folder carries, which its run, branch and unit also carry.
   */
  const chosen = taken.review;
  await writeRecordIn({
    recordFile: recordFileOfAReview({
      reviewsFolder,
      review: chosen,
    },),
    record: {
      ...((branch === undefined) ? {} : { branch, }),
      commit,
      ranges,
      source,
      claimedAt: new Date()
        .toISOString(),
    },
  },);
  /**
   The file the diff is written to.
   */
  const diffFile = diffFileOfAReview({
    reviewsFolder,
    review: chosen,
  },);
  /**
   The file the brief is written to.
   */
  const briefFile = briefFileOfAReview({
    reviewsFolder,
    review: chosen,
  },);
  await mkdir(
    reproductionsFolderOfAReview({
      reviewsFolder,
      review: chosen,
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
    (material.kind === 'tests-and-fixtures')
      ? briefOfAChangeToTests({
        review: chosen,
        commit,
        source,
        files: material.files,
        removedOrRenamed: material.removedOrRenamed,
        focus,
        reviewsFolder,
        runsFolder,
        rulesFile,
      },)
      : briefOfAReview({
        review: chosen,
        commit,
        source,
        files: material.files,
        removedOrRenamed: material.removedOrRenamed,
        focus,
        reviewsFolder,
        runsFolder,
        rulesFile,
      },),
    'utf8',
  );
  // What the review read is kept beside its files: a branch moves on as soon as anyone commits to it, and a fix that answers
  // this review has to start at the commit the review actually read.
  await writeFile(
    readingFileOfAReview({
      reviewsFolder,
      review: chosen,
    },),
    `${JSON.stringify(
      {
        review: chosen,
        commit,
        ranges,
        source,
      } satisfies ReviewReading,
      undefined,
      2
    )}\n`,
    'utf8',
  );
  return {
    written: true,
    review: chosen,
    line: `${runNameOfAReview({ review: chosen, },)}: ${String(material.files
      .length,)} files to read, brief at ${briefFile}; `
      + `its worktree starts at ${commit}. Its folder is ${taken.folder}`,
  };
}
