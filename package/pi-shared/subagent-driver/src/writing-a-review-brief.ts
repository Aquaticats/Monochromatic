/**
 Writes what a review of one change reads: the diff of its ranges under the reviewed source folder, the brief that names its
 files, and the folder its reproductions go in.

 A change that leaves nothing to read under the source folder writes nothing and answers one line, because a reviewer given no
 file reports nothing, which reads as a clean review although nothing was reviewed, and that line says what the ranges changed
 there and why a review reads none of it. What a change leaves to read is the code under that folder and the files that decide
 how that code is built, checked or run, and where it changes nothing but tests and fixtures, those files, whose brief asks the
 reviewer to plant a break under each changed test and say whether it fails.

 A file whose extension is in no list of `which-files-a-review-reads.ts` is read as code and named in the line this module answers,
 because a kind the writer does not know must not be dropped silently.

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
  if (material.kind === 'nothing-a-review-reads') {
    return {
      written: false,
      line: `nothing to review under ${source} in ${ranges.join(' and ',)}: ${material.whyNothingIsRead}`,
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
    (material.kind === 'tests-and-fixtures')
      ? briefOfAChangeToTests({
        review,
        commit,
        source,
        files: material.files,
        filesOfCode: material.filesOfCode,
        filesLeftOut: material.filesLeftOut,
        filesRemoved: material.filesRemoved,
        filesOfTests: material.filesOfTests,
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
        filesLeftOut: material.filesLeftOut,
        filesRemoved: material.filesRemoved,
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
  /**
   The sentence that says where the review's worktree starts and where the review's own folder is.
   */
  const whereItStarts = `its worktree starts at ${commit}. Its folder is ${reviewFolder}`;
  /**
   What the line adds about the files of a kind no list names, which is nothing when every file is of a kind one names: the plan
   names them for both kinds of change, so a file of an unlisted kind beside a test is reported as read as code too.
   */
  const unlisted = sentenceOfFilesOfAnUnlistedKind({ files: material.filesOfAnUnlistedKind, },);
  /**
   How many files the reviewer is given: the ones it reads in full and the ones the change removed, which it reads in the diff.
   */
  const fileCount = material.files
    .length
    + material.filesRemoved
      .length;
  /**
   The count as the line says it, so a line about one file says one file.
   */
  const count = `${String(fileCount,)} ${(fileCount === 1) ? 'file' : 'files'}`;
  /**
   The opening of the line: the run, how many files it reads, and where the brief it reads is kept.
   */
  const opening = `${runNameOfAReview({ review, },)}: ${count} to read, brief at ${briefFile}`;
  return {
    written: true,
    line: `${opening}; ${whereItStarts}${unlisted}`,
  };
}

/**
 Reads the sentence a written line carries when the change holds a file of a kind the lists of `which-files-a-review-reads.ts` do
 not name, so a kind nobody listed is read and seen rather than read as though somebody had named it.

 A file of an unlisted kind that the test rule of `material-of-a-review.ts` claims, a `.zig` beside a `tests` folder for example,
 is read as a test instead and the line says the change is to tests, so nothing of it goes unread either way.

 @param files - the files of a kind no list names, from the top of the checkout

 @returns the sentence, with the period and space before it, or an empty string when the change holds no such file

 @example
 ```ts
 const sentence = sentenceOfFilesOfAnUnlistedKind({ files: ['package/a/src/thing.zig'], },);
 ```
 */
export function sentenceOfFilesOfAnUnlistedKind({ files, }: { readonly files: readonly string[] },): string {
  if (files.length === 0) {
    return '';
  }
  return `. Files of a kind no list of this writer names are read as code: ${files.map(function asAPath(path,) {
    return `\`${path}\``;
  },)
    .join(', ',)}. Those extensions belong in \`src/which-files-a-review-reads.ts\`, so the next change of one is named there.`;
}
