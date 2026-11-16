/**
 Reads what a review's statement says against the diff and the session it left: the files its brief lists, the statement itself
 (this package's JSON block or the prose form the coordinator asks for), and the comparison with the commands the session records.

 @module
 */

import {
  type FilesInADiff,
  ReviewDiffError,
  filesInDiff,
} from './review-diff.ts';
import type { CommandComparison, } from './build-path-of-a-review.ts';
import { compareCommands, } from './review-command-matching.ts';
import {
  REVIEW_STATEMENT_HEADING,
  type ReviewStatement,
  ReviewStatementError,
} from './review-statement-shape.ts';
import { pathOf, } from './git-quoted-path.ts';
import { proseStatementIn, } from './review-statement-prose.ts';
import { statementInReview, } from './review-statement.ts';
import {
  briefFileOfAReview,
  diffFileOfAReview,
  reviewFileOfAReview,
  reviewFolderOf,
} from './reviewer-of-a-change.ts';
import type { SessionReadings, } from './session-commands-of-a-run.ts';
import {
  type TextAtAPath,
  textAtAPath,
} from './review-check-reading.ts';

/**
 What reading a review's statement, session and diff answered.
 */
export type StatementReading =
  | {
    readonly kind: 'read';
    readonly statement: ReviewStatement;
    readonly diff: FilesInADiff;
    readonly comparison: CommandComparison;

    /**
     The files the review's brief lists, which the statement had to account for.
     */
    readonly briefFiles: readonly string[];
  }
    | {
      readonly kind: 'unreadable';
      readonly reasons: readonly string[];
    };

/**
 The length of the bullet every file line of a review's brief starts with.
 */
const BULLET_PREFIX_LENGTH = 3;

/**
 Reads the files a review's brief lists.

 The list is the one the brief writes after "Read each of these files in full before you answer", so a review is compared with the
 files it was given rather than with every file the diff touches. A brief that is not there, or holds no such list, falls back to the
 diff's files.

 @param reviewsFolder - folder that holds one folder per review

 @param review - the change the review read

 @param diffFiles - the files the diff touches

 @returns the files the brief lists, or the diff's files when it lists none

 @example
 ```ts
 const files = await briefFilesIn({ reviewsFolder: '/reviews', review: 'a', diffFiles: ['package/a/src/x.ts'] });
 ```
 */
export async function briefFilesIn({
  reviewsFolder,
  review,
  diffFiles,
}: {
  readonly reviewsFolder: string;
  readonly review: string;
  readonly diffFiles: readonly string[];
},): Promise<readonly string[]> {
  /**
   The brief file's path.
   */
  const briefFile = briefFileOfAReview({
    reviewsFolder,
    review,
  },);
  /**
   What the brief holds.
   */
  const shown = await textAtAPath({ path: briefFile, },);
  if (shown.kind !== 'the text') {
    return diffFiles;
  }
  /**
   The lines of the brief.
   */
  const lines = shown.text
    .split('\n',);
  /**
   Where the list of files starts.
   */
  const at = lines.findIndex(function isTheListLine(line,) {
    return line.includes('Read each of these files in full',);
  },);
  if (at === (-1)) {
    return diffFiles;
  }
  /**
   The files the list names.
   */
  const files: string[] = [];
  for (const line of lines.slice(at + 1,)) {
    /**
     The line without the spaces around it.
     */
    const trimmed = line.trim();
    if ((!trimmed.startsWith('- `',)) || (!trimmed.endsWith('`',))) {
      break;
    }
    /**
     The path the line names.
     */
    const path = pathOf(trimmed.slice(
      BULLET_PREFIX_LENGTH,
      -1,
    ),);
    if (path !== '') {
      files.push(path,);
    }
  }
  return (files.length === 0) ? diffFiles : files;
}

/**
 Reads the statement, the diff and the session's comparison, so the verdict is one decision over all of them.

 A review file that holds a `review-statement` block is read as this package's JSON; one that holds no such block is read as the
 prose form the coordinator asks for, whose coverage count and built-and-run list are read from its text.

 @param review - the change the review read

 @param reviewsFolder - folder that holds one folder per review

 @param session - what the review's session records

 @param recordedCommit - the commit the review's record names, used when a prose statement states none

 @returns the statement, the diff and the files the brief lists

 @example
 ```ts
 const reading = await readingOfAStatement({ review, reviewsFolder: '/reviews', session, recordedCommit: 'abc1234' });
 ```
 */
export async function readingOfAStatement({
  review,
  reviewsFolder,
  session,
  recordedCommit,
}: {
  readonly review: string;
  readonly reviewsFolder: string;
  readonly session: SessionReadings;
  readonly recordedCommit: string;
},): Promise<StatementReading> {
  /**
   The review file the statement is read from.
   */
  const reviewFile = reviewFileOfAReview({
    reviewsFolder,
    review,
  },);
  /**
   What the review file holds.
   */
  const shown = await textAtAPath({ path: reviewFile, },);
  if (shown.kind === 'unreadable') {
    return {
      kind: 'unreadable',
      reasons: [`${reviewFile} cannot be read as a statement: ${shown.reason}`,],
    };
  }
  /**
   The review file's text.
   */
  const text = (shown.kind === 'the text') ? shown.text : '';
  /**
   The diff file's path, which the statement's files and line count are compared with.
   */
  const diffFile = diffFileOfAReview({
    reviewsFolder,
    review,
  },);
  /**
   The files the diff touches, or the reasons its text cannot be read as a diff.
   */
  const diffReading = await diffFilesOrReasons({ diffFile, },);
  if (diffReading.kind !== 'the files') {
    return {
      kind: 'unreadable',
      reasons: diffReading.reasons,
    };
  }
  /**
   The diff's own reading.
   */
  const {diff} = diffReading;
  /**
   The files the brief lists, which are the files the statement must account for.
   */
  const briefFiles = await briefFilesIn({
    reviewsFolder,
    review,
    diffFiles: diff.files,
  },);
  try {
    /**
     The statement the review wrote: its JSON block when it wrote one, its prose form otherwise.
     */
    const statement = text.includes(REVIEW_STATEMENT_HEADING,)
      ? statementInReview({
        shown: text,
        file: reviewFile,
      },)
      : proseStatementIn({
        shown: text,
        file: reviewFile,
        briefFiles,
        diffLines: diff.lines,
        recordedCommit,
      },);
    return {
      kind: 'read',
      statement,
      diff,
      briefFiles,
      comparison: await compareCommands({
        statement,
        session,
        reviewFolder: reviewFolderOf({
          reviewsFolder,
          review,
        },),
      },),
    };
  }
  catch (error) {
    // A statement this reader cannot read is what the verdict reports; any other failure is the machine's.
    if (error instanceof ReviewStatementError) {
      return {
        kind: 'unreadable',
        reasons: [error.message,],
      };
    }
    throw error;
  }
}

/**
 Reads the files a diff touches, answering the reasons its text cannot be read as a diff instead of raising them.

 @param diffFile - path of the diff file the review folder holds

 @returns the diff's files and line count, or one line per reason it cannot be read

 @example
 ```ts
 const reading = await diffFilesOrReasons({ diffFile: '/reviews/x/change.diff' });
 ```
 */
async function diffFilesOrReasons({ diffFile, }: { readonly diffFile: string },): Promise<
  | {
    readonly kind: 'the files';
    readonly diff: FilesInADiff
  }
    | {
      readonly kind: 'unreadable';
      readonly reasons: readonly string[]
    }
  > {
  /**
   What the diff file holds.
   */
  const shown = await textAtAPath({ path: diffFile, },);
  if (shown.kind !== 'the text') {
    return {
      kind: 'unreadable',
      reasons: [
        (shown.kind === 'no file')
          ? `${diffFile} is not there, so the files and lines the statement names cannot be compared`
          : `${shown.reason}, so the files and lines the statement names cannot be compared`,
      ],
    };
  }
  try {
    return {
      kind: 'the files',
      diff: filesInDiff({ diff: shown.text, },),
    };
  }
  catch (error) {
    if (error instanceof ReviewDiffError) {
      return {
        kind: 'unreadable',
        reasons: [error.message,],
      };
    }
    throw error;
  }
}
