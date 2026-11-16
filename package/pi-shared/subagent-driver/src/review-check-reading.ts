/**
 What one review's statement, diff and session say together: the coverage a statement describes, the lines a verdict prints about
 it, the files it leaves out, and the comparison with the commands the session records.

 The reading lives in a file of its own so the check's verdict stays within the line budget and each step can be read on its own:
 the checkout-independent reading, the comparison, and the verdict made from both.

 @module
 */

import { readRegularFileAt, } from './reading-a-regular-file.ts';
import {
  type ReviewReading,
  ReviewReadingError,
  readingIn,
} from './material-of-a-review.ts';
import {
  type CommandComparison,
  compareCommands,
} from './review-command-matching.ts';
import {
  type FilesInADiff,
  ReviewDiffError,
  filesInDiff,
} from './review-diff.ts';
import {
  type ReviewStatement,
  ReviewStatementError,
} from './review-statement-shape.ts';
import {
  statementInReview,
  commitIsTheSame
} from './review-statement.ts';
import {
  diffFileOfAReview,
  reviewFileOfAReview,
  reviewFolderOf,
} from './reviewer-of-a-change.ts';
import type { SessionReadings, } from './session-commands-of-a-run.ts';

/**
 What is at a path one review's file is read from.
 */
export type TextAtAPath =
  | {
    readonly kind: 'the text';
    readonly text: string;
  }
    | {
      readonly kind: 'no file';
    }
    | {
      readonly kind: 'unreadable';
      readonly reason: string;
    };

/**
 Which files of the diff a review read whole, and which it did not.
 */
export type ReviewCoverage = {
  /**
   The lines the diff file holds.
   */
  readonly diffLines: number;

  /**
   The lines of it the review says it read.
   */
  readonly linesRead: number;

  /**
   The files the review says it read whole.
   */
  readonly filesReadWhole: readonly string[];

  /**
   The files the review says it read in part or not at all, which a second review may read.
   */
  readonly filesNotReadWhole: readonly string[];
};

/**
 What reading a review's statement, session and diff answered.
 */
export type StatementReading =
  | {
    readonly kind: 'read';
    readonly statement: ReviewStatement;
    readonly diff: FilesInADiff;
    readonly comparison: CommandComparison;
  }
    | {
      readonly kind: 'unreadable';
      readonly reasons: readonly string[];
    };

/**
 What a record of what a review read says.
 */
export type ReadingAtAPath =
  | {
    readonly kind: 'read';
    readonly reading: ReviewReading;
  }
    | {
      readonly kind: 'unreadable';
    };

/**
 Reads one review file, or says what is at its path when it is not a regular file this reader reads.

 The descriptor is opened first, without waiting for a writer, and looked at second: a named pipe at a review file's path answers
 at once, and a file larger than a review file could be is refused rather than held in memory.

 @param path - the file to read

 @param limitBytes - the most bytes the file may hold

 @returns the file's text, or the fact that it is no file or cannot be read

 @throws the error a read gave for a regular file this reader should have been able to read

 @example
 ```ts
 const read = await textAtAPath({ path: '/reviews/a/review.md' });
 ```
 */
export async function textAtAPath({
  path,
  limitBytes,
}: {
  readonly path: string;
  readonly limitBytes?: number;
},): Promise<TextAtAPath> {
  /**
   What the read answered, which is one descriptor opened without waiting and looked at after it was opened.
   */
  const read = await readRegularFileAt({
    path,
    ...((limitBytes === undefined) ? {} : { limitBytes, }),
  },);
  if (read.kind === 'text') {
    return {
      kind: 'the text',
      text: read.text,
    };
  }
  if (read.kind === 'no file') {
    return { kind: 'no file', };
  }
  return {
    kind: 'unreadable',
    reason: read.reason,
  };
}

/**
 Reads what a review recorded it read, or says it cannot be read.

 @param file - the record's path

 @returns the record, or the answer that it does not fit

 @example
 ```ts
 const read = await readingAtAPath('/reviews/a/what-the-review-read.json');
 ```
 */
export async function readingAtAPath(file: string,): Promise<ReadingAtAPath> {
  /**
   What the file holds.
   */
  const shown = await textAtAPath({ path: file, },);
  if (shown.kind !== 'the text') {
    return { kind: 'unreadable', };
  }
  try {
    return {
      kind: 'read',
      reading: readingIn({
        shown: shown.text,
        file,
      },),
    };
  }
  catch (error) {
    // A record with no commit, ranges and source is a fact to report rather than a reason to stop the check.
    if ((error instanceof ReviewReadingError) || (error instanceof SyntaxError)) {
      return { kind: 'unreadable', };
    }
    throw error;
  }
}

/**
 Reads the coverage a statement and a diff together describe.

 @param statement - what the review says it read

 @param diff - the files the diff touches and its lines

 @returns the coverage

 @example
 ```ts
 const coverage = coverageIn({ statement, diff });
 ```
 */
export function coverageIn({
  statement,
  diff,
}: {
  readonly statement: ReviewStatement;
  readonly diff: FilesInADiff;
},): ReviewCoverage {
  return {
    diffLines: diff.lines,
    linesRead: statement.diff
      .linesRead,
    filesReadWhole: statement.files
      .filter(function whole(file,) {
        return file.read === 'whole';
      },)
      .map(function pathOf(file,) {
        return file.path;
      },),
    filesNotReadWhole: statement.files
      .filter(function partial(file,) {
        return file.read !== 'whole';
      },)
      .map(function pathOf(file,) {
        return file.path;
      },),
  };
}

/**
 Reads the lines that say what the review read, for a verdict's reasons.

 @param coverage - what the review says it read

 @returns one line about the lines, one about the files, and one naming the files not read whole when any were not

 @example
 ```ts
 const lines = coverageReasonsOf(coverage);
 ```
 */
export function coverageReasonsOf(coverage: ReviewCoverage,): readonly string[] {
  return [
    `read ${String(coverage.linesRead,)} of the diff's ${String(coverage.diffLines,)} lines`,
    `${String(coverage.filesReadWhole
      .length,)} of `
      + `${String(coverage.filesReadWhole
        .length
        + coverage.filesNotReadWhole
          .length,)} files read whole`,
    ...((coverage.filesNotReadWhole
      .length
      === 0)
      ? []
      : [`not read whole: ${coverage.filesNotReadWhole
        .join(', ')}`]),
  ];
}

/**
 Checks the files a statement names against the files a diff touches.

 @param statement - what the review says it read

 @param diff - the files the diff touches

 @returns one line per file the statement leaves out or names although the diff does not touch it

 @example
 ```ts
 const problems = fileProblemsIn({ statement, diff });
 ```
 */
export function fileProblemsIn({
  statement,
  diff,
}: {
  readonly statement: ReviewStatement;
  readonly diff: FilesInADiff;
},): readonly string[] {
  /**
   The paths the statement names.
   */
  const stated = new Set(statement.files
    .map(function pathOf(file,) {
      return file.path;
    },),);
  /**
   The files the diff touches that the statement does not name.
   */
  const missing = diff.files
    .filter(function unnamed(path,) {
      return !stated.has(path,);
    },);
  /**
   The files the statement names that the diff does not touch.
   */
  const extra = statement.files
    .map(function pathOf(file,) {
      return file.path;
    },)
    .filter(function untouched(path,) {
      return !diff.files
        .includes(path,);
    },);
  /**
   The missing files' line, when any file is missing.
   */
  const missingLine = (missing.length === 0) ? [] : [`the statement does not say how ${missing.join(', ')} was read`];
  /**
   The extra files' line, when the statement names any.
   */
  const extraLine = (extra.length === 0) ? [] : [`the statement names ${extra.join(', ')}, which the diff does not touch`];
  return [
    ...missingLine,
    ...extraLine,
  ];
}

/**
 Reads the statement, the diff and the session's comparison, so the verdict is one decision over all of them.

 @param review - the change the review read

 @param reviewsFolder - folder that holds one folder per review

 @param session - what the review's session records

 @returns the statement and what it was compared with, or the reasons it could not be read

 @example
 ```ts
 const reading = await readingOfAStatement({ review, reviewsFolder: '/reviews', session });
 ```
 */
export async function readingOfAStatement({
  review,
  reviewsFolder,
  session,
}: {
  readonly review: string;
  readonly reviewsFolder: string;
  readonly session: SessionReadings;
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
  try {
    /**
     The statement the review wrote.
     */
    const statement = statementInReview({
      shown: (shown.kind === 'the text') ? shown.text : '',
      file: reviewFile,
    },);
    /**
     The diff file's path, which the statement's files and line count are compared with.
     */
    const diffFile = diffFileOfAReview({
      reviewsFolder,
      review,
    },);
    /**
     What the diff file holds.
     */
    const diffShown = await textAtAPath({ path: diffFile, },);
    if (diffShown.kind !== 'the text') {
      return {
        kind: 'unreadable',
        reasons: [
          (diffShown.kind === 'no file')
            ? `${diffFile} is not there, so the files and lines the statement names cannot be compared`
            : `${diffShown.reason}, so the files and lines the statement names cannot be compared`,
        ],
      };
    }
    /**
     The files the diff touches.
     */
    const diff = filesInDiff({ diff: diffShown.text, },);
    return {
      kind: 'read',
      statement,
      diff,
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
    // A statement or a diff this reader cannot read is what the verdict reports; any other failure is the machine's.
    if ((error instanceof ReviewStatementError) || (error instanceof ReviewDiffError)) {
      return {
        kind: 'unreadable',
        reasons: [error.message,],
      };
    }
    throw error;
  }
}

/**
 What a statement's commit and the review's record say together.
 */
export type CommitReading =
  | {
    readonly kind: 'the same commit';
  }
    | {
      readonly kind: 'another commit';
      readonly problem: string;
    }
    | {
      readonly kind: 'cannot be compared';
      readonly uncompared: string;
    };

/**
 Reads what a statement's commit and the review's record say together: whether they name the same commit, another one, or no
 commit this reader can compare.

 @param recorded - what the review recorded it read

 @param stated - the commit the statement names

 @param readingFile - the record's path, for the message

 @returns which of the three it is, with the line a verdict prints

 @example
 ```ts
 const commit = commitReadingIn({ recorded, stated: 'abc123', readingFile: '/reviews/a/what-the-review-read.json' });
 ```
 */
export function commitReadingIn({
  recorded,
  stated,
  readingFile,
}: {
  readonly recorded: ReadingAtAPath;
  readonly stated: string;
  readonly readingFile: string;
},): CommitReading {
  if (recorded.kind === 'unreadable') {
    return {
      kind: 'cannot be compared',
      uncompared: `${readingFile} is not there or names no commit, so the commit the statement names cannot be compared`,
    };
  }
  return commitIsTheSame({
    stated,
    recorded: recorded.reading
      .commit,
  },)
    ? { kind: 'the same commit', }
    : {
      kind: 'another commit',
      problem: `the statement names commit ${stated} and the review recorded ${recorded.reading
        .commit}`,
    };
}
