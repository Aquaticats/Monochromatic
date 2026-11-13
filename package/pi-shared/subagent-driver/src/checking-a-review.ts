/**
 The check of one ended review: is there a review file, does its statement section hold the shape it must, does the session show
 what the statement says it ran, did any command run a program built from the reviewed commit, and did the run end by itself or was
 it cut.

 The check reads files only. It starts no Pi run, asks no model, and its verdict is the same every time it reads the same run:
 `complete`, `ran nothing`, `statement missing or not matching its session`, `no review file`, or `cut`.

 The verdict is bookkeeping of evidence and coverage, never a judgement of quality: a review that says it read part of a diff is
 `complete` and says which files it did not read whole.

 @module
 */

import { join, } from 'node:path';
import { endReasonIn, } from './end-of-a-run.ts';
import { endedOnItsOwnTerms, } from './end-reasons.ts';
import {
  NO_FINISHED_RUN_OF_THAT_NAME,
  isThere,
  newestFinishedRunOf,
} from './newest-finished-run-of-a-name.ts';
import {
  type ReadingAtAPath,
  type ReviewCoverage,
  coverageIn,
  coverageReasonsOf,
  fileProblemsIn,
  readingAtAPath,
  readingOfAStatement,
} from './review-check-reading.ts';
import type { ReviewStatement, } from './review-statement-shape.ts';
import { commitIsTheSame, } from './review-statement.ts';
import {
  readingFileOfAReview,
  reviewFileOfAReview,
  runNameOfAReview,
} from './reviewer-of-a-change.ts';
import {
  type SessionOfARun,
  sessionOfTheRun,
} from './session-of-a-finished-run.ts';

/**
 What one review's check answered.
 */
export type ReviewVerdict =
  | 'complete'
    | 'ran nothing'
    | 'statement missing or not matching its session'
    | 'no review file'
    | 'cut';

/**
 One review's verdict and the reasons for it.
 */
export type ReviewCheck = {
  /**
   The change the review read.
   */
  readonly review: string;

  /**
   The verdict.
   */
  readonly verdict: ReviewVerdict;

  /**
   One line per fact that decided the verdict, and per claim the check could not confirm either way.
   */
  readonly reasons: readonly string[];

  /**
   The review file's path.
   */
  readonly reviewFile: string;

  /**
   The newest finished run of the review, when one finished.
   */
  readonly run?: string;

  /**
   Why that run ended, as its summary says.
   */
  readonly endReason?: string;

  /**
   Whether that run ended on its own terms, which is false for a run a cap, a wall clock, an abort or a model error ended.
   */
  readonly endedByItself: boolean;

  /**
   The statement the review wrote, when it wrote one the reader can read.
   */
  readonly statement?: ReviewStatement;

  /**
   What the statement says the review read, when the diff and the statement both read.
   */
  readonly coverage?: ReviewCoverage;

  /**
   The programs the statement names and the session confirms as ones the review's worktree built.
   */
  readonly builtPrograms: readonly string[];
};

/**
 The line one check's verdict prints, and the limits every verdict is read with.
 */
export const CHECK_LIMITS_LINE: string =
  'Compared: each statement command against the bash and process-start calls its session records, the exit code a bash result '
    + 'holds, the statement\'s files and line count against the diff file, and the commit against what the review recorded it '
    + 'read. Not visible: commands run inside a codemode call, a process start\'s exit code (its result records none), a script '
    + 'that builds a built program\'s path from a variable rather than holding it, and anything written after the run ended.';

/**
 Reads the commit problems a statement's commit and the review's record show.

 @param recorded - what the review recorded it read

 @param stated - the commit the statement names

 @param readingFile - the record's path, for the message

 @returns one line per problem, or one line saying the commit cannot be compared

 @example
 ```ts
 const problems = commitProblemsIn({ recorded, stated: 'abc123', readingFile: '/reviews/a/what-the-review-read.json' });
 ```
 */
function commitProblemsIn({
  recorded,
  stated,
  readingFile,
}: {
  readonly recorded: ReadingAtAPath;
  readonly stated: string;
  readonly readingFile: string;
},): readonly string[] {
  if (recorded.kind === 'unreadable') {
    return [`${readingFile} is not there or names no commit, so the commit the statement names cannot be compared`];
  }
  return commitIsTheSame({
    stated,
    recorded: recorded.reading
      .commit,
  },)
    ? []
    : [`the statement names commit ${stated} and the review recorded ${recorded.reading
      .commit}`];
}

/**
 Checks one ended review against the run and session it left.

 @param review - the change the review read

 @param reviewsFolder - folder that holds one folder per review

 @param runsFolder - folder the driver writes its runs into

 @returns the verdict and its reasons

 @throws when a file other than the review's own cannot be read, which is a machine's failure rather than the review's

 @example
 ```ts
 const check = await checkAReview({ review: 'a', reviewsFolder: '/reviews', runsFolder: '/runs' });
 ```
 */
export async function checkAReview({
  review,
  reviewsFolder,
  runsFolder,
}: {
  readonly review: string;
  readonly reviewsFolder: string;
  readonly runsFolder: string;
},): Promise<ReviewCheck> {
  /**
   The review file's path.
   */
  const reviewFile = reviewFileOfAReview({
    reviewsFolder,
    review,
  },);
  /**
   The run whose folder holds the session, which is the newest attempt of this review that finished.
   */
  const newest = await newestFinishedRunOf({
    runsFolder,
    runName: runNameOfAReview({ review, },),
  },);
  /**
   That run's folder, or nothing when no attempt finished.
   */
  const run = (newest === NO_FINISHED_RUN_OF_THAT_NAME) ? undefined : newest;
  /**
   Why that run ended, when one finished.
   */
  const endReason = (run === undefined)
    ? undefined
    : (await endReasonIn({ folder: join(
      runsFolder,
      run,
    ), },)).endReason;
  /**
   Whether the run ended on its own terms.
   */
  const endedByItself = endedOnItsOwnTerms(endReason,);
  if (!(await isThere(reviewFile,))) {
    /**
     What the run's ending says, when a run finished.
     */
    const ending = `${run} ended ${endReason ?? 'with no reason its summary names'}, `
      + `${endedByItself ? 'so it ended by itself' : 'so it was cut'}, and it wrote no review.md`;
    return {
      review,
      verdict: 'no review file',
      reasons: (run === undefined)
        ? [`no finished run of ${runNameOfAReview({ review, },)} is in ${runsFolder}`]
        : [ending],
      reviewFile,
      ...((run === undefined) ? {} : { run, }),
      ...((endReason === undefined) ? {} : { endReason, }),
      endedByItself,
      builtPrograms: [],
    };
  }
  if (run === undefined) {
    return {
      review,
      verdict: 'cut',
      reasons: [`no finished run of ${runNameOfAReview({ review, },)} is in ${runsFolder}, so no summary or session says what `
        + 'the review read or ran'],
      reviewFile,
      endedByItself,
      builtPrograms: [],
    };
  }
  if (!endedByItself) {
    return {
      review,
      verdict: 'cut',
      reasons: [`${run} ended ${endReason ?? 'without a reason its summary names'}, so it was cut, not ended by itself`],
      reviewFile,
      run,
      ...((endReason === undefined) ? {} : { endReason, }),
      endedByItself,
      builtPrograms: [],
    };
  }
  /**
   What the run's session records, or why it cannot be read.
   */
  const session = await sessionOfTheRun({
    runsFolder,
    run,
  },);
  if (session.kind === 'unreadable') {
    return {
      review,
      verdict: 'cut',
      reasons: [`${run} ended ${endReason ?? 'by itself'}, and its session cannot be read, so what it read and ran cannot be `
        + `compared: ${session.reason}`],
      reviewFile,
      run,
      ...((endReason === undefined) ? {} : { endReason, }),
      endedByItself,
      builtPrograms: [],
    };
  }
  /**
   The statement, the diff and the comparison with the session.
   */
  const reading = await readingOfAStatement({
    review,
    reviewsFolder,
    session: session.session,
  },);
  if (reading.kind === 'unreadable') {
    return {
      review,
      verdict: 'statement missing or not matching its session',
      reasons: [...reading.reasons,],
      reviewFile,
      run,
      ...((endReason === undefined) ? {} : { endReason, }),
      endedByItself,
      builtPrograms: [],
    };
  }
  /**
   What the review says it read.
   */
  const coverage = coverageIn({
    statement: reading.statement,
    diff: reading.diff,
  },);
  /**
   The line the statement's own line count earns, when it does not name the diff's.
   */
  const lineCountProblem = (reading.statement
    .diff
    .lines
    === reading.diff
      .lines)
    ? []
    : [`the statement says the diff holds ${String(reading.statement
      .diff
      .lines,)} lines and it holds ${
       String(reading.diff
         .lines,)}`];
  /**
   What the comparison found that does not fit.
   */
  const problems = [
    ...lineCountProblem,
    ...fileProblemsIn({
      statement: reading.statement,
      diff: reading.diff,
    },),
    ...reading.comparison
      .problems,
  ];
  /**
   The record of what the review read, which the statement's commit is compared with.
   */
  const readingFile = readingFileOfAReview({
    reviewsFolder,
    review,
  },);
  /**
   What the session cannot confirm either way.
   */
  const uncompared = [
    ...commitProblemsIn({
      recorded: await readingAtAPath(readingFile,),
      stated: reading.statement
        .commit,
      readingFile,
    },),
    ...reading.comparison
      .uncompared,
    ...((session.session
      .codemodeCalls
      === 0)
      ? []
      : [`${String(session.session
        .codemodeCalls,)} codemode call(s) are not read, so a command they ran cannot be compared`]),
  ];
  if (problems.length > 0) {
    return {
      review,
      verdict: 'statement missing or not matching its session',
      reasons: [
        ...problems,
        ...coverageReasonsOf(coverage,),
        ...uncompared
      ],
      reviewFile,
      run,
      ...((endReason === undefined) ? {} : { endReason, }),
      endedByItself,
      statement: reading.statement,
      coverage,
      builtPrograms: reading.comparison
        .builtPrograms,
    };
  }
  if (reading.comparison
    .builtPrograms
    .length
    === 0) {
    return {
      review,
      verdict: 'ran nothing',
      reasons: [
        `no command it lists ran a program built from commit ${reading.statement
          .commit}, and no command its session records `
          + 'does',
        ...coverageReasonsOf(coverage,),
        ...uncompared,
      ],
      reviewFile,
      run,
      ...((endReason === undefined) ? {} : { endReason, }),
      endedByItself,
      statement: reading.statement,
      coverage,
      builtPrograms: [],
    };
  }
  return {
    review,
    verdict: 'complete',
    reasons: [
      ...coverageReasonsOf(coverage,),
      `${String(reading.comparison
        .builtPrograms
        .length,)} command(s) ran a program built from commit `
        + `${reading.statement
          .commit}: ${reading.comparison
            .builtPrograms
            .join(', ')}`,
      `the run ended ${endReason ?? 'with no reason its summary names'}`,
      ...uncompared,
    ],
    reviewFile,
    run,
    ...((endReason === undefined) ? {} : { endReason, }),
    endedByItself,
    statement: reading.statement,
    coverage,
    builtPrograms: reading.comparison
      .builtPrograms,
  };
}

/**
 Reads the line one check prints: the run's name, its verdict, and its reasons.

 @param check - one review's check

 @returns the line

 @example
 ```ts
 const line = checkLineOf({ check });
 ```
 */
export function checkLineOf({ check, }: { readonly check: ReviewCheck },): string {
  /**
   The reasons, with the separator the line reads them with.
   */
  const reasons = (check.reasons
    .length
    === 0) ? '' : `; ${check.reasons
      .join('; ')}`;
  return `${runNameOfAReview({ review: check.review, },)}: ${check.verdict}${reasons}`;
}
