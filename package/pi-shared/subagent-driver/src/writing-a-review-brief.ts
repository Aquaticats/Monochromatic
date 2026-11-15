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

 A review's folder is taken by the start that writes its inputs, by creating an owner file only one of them can create, and only
 once the change is known to leave something to read. A second start with the same change name learns from that file whose the
 folder is and is refused with both branches named, while a folder whose owner has ended is taken again: a start killed between
 taking a folder and writing its brief blocks nobody for good.

 @module
 */

import {
  mkdir,
  writeFile,
} from 'node:fs/promises';
import { randomUUID, } from 'node:crypto';
import { dirname, } from 'node:path';
import {
  type ReviewFolderOwner,
  takeReviewFolder,
} from './claiming-a-review-folder.ts';
import { processIdentityOfThisProcess, } from './liveness-of-a-process.ts';
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
  ownerFileOfAReview,
  readingFileOfAReview,
  reproductionsFolderOfAReview,
  reviewFolderOf,
  runNameOfAReview,
} from './reviewer-of-a-change.ts';
import {
  type RunCommand,
  runCommandWithChildProcess,
} from './run-command.ts';
import { unitNameOfGiven, } from './running-pi-units.ts';

/**
 Writes the diff, the brief and the reproductions folder of one review.

 @param git - how one git question is run

 @param checkoutFolder - the checkout the diff is read from

 @param review - the change the review reads

 @param commit - the commit under review

 @param ranges - what changed, each as `git diff` takes one

 @param source - the folder whose changes are reviewed

 @param branch - the branch whose change the review reads, when the caller names one; a second branch of one change name is refused
 with both named

 @param focus - what the reviewer looks hardest at

 @param reviewsFolder - folder that holds one folder per review

 @param runsFolder - folder the driver writes its runs into

 @param rulesFile - the rules file's path, which the brief names first

 @param manager - how the user's manager is asked whether a review of this folder still runs, which a test hands in

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
  branch,
  focus,
  reviewsFolder,
  runsFolder,
  rulesFile,
  manager = runCommandWithChildProcess,
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
  readonly manager?: RunCommand;
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
   The review's own folder.
   */
  const reviewFolder = reviewFolderOf({
    reviewsFolder,
    review,
  },);
  // The folder is taken only now: a change that leaves nothing to read writes nothing and holds no folder, so a branch with
  // nothing to review never stands in the way of one with something to review under the same change name.
  /**
   The unit this review runs as, whose life the folder's claim follows.
   */
  const unit = unitNameOfGiven({ name: runNameOfAReview({ review, },), },);
  await takeReviewFolder({
    reviewFolder,
    ownerFile: ownerFileOfAReview({
      reviewsFolder,
      review,
    },),
    ...((branch === undefined) ? {} : { branch, }),
    unit,
    manager,
    owner: {
      ...((branch === undefined) ? {} : { branch, }),
      commit,
      ranges,
      source,
      claimedAt: new Date()
        .toISOString(),
      unit,
      claim: randomUUID(),
      process: await processIdentityOfThisProcess(),
    },
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
    (material.kind === 'tests-and-fixtures')
      ? briefOfAChangeToTests({
        review,
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
        review,
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
      review,
    },),
    `${JSON.stringify(
      {
        review,
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
    line: `${runNameOfAReview({ review, },)}: ${String(material.files
      .length,)} files to read, brief at ${briefFile}; `
      + `its worktree starts at ${commit}. Its folder is ${reviewFolder}`,
  };
}
