/**
 The check of one ended review: is there a review file, does its statement say what it read and ran, does the session show what the
 statement says it ran, did any command run a program a build wrote or a build task, and did the run end by itself or was it cut.

 The check reads files only. It starts no Pi run, asks no model, and its verdict is the same every time it reads the same run:
 `complete`, `ran nothing`, `still running`, `statement missing or not matching its session`, `no review file`, or `cut`.

 The verdict is bookkeeping of evidence and coverage, never a judgement of quality: a review that says it read part of a diff is
 `complete` and says which files it did not read whole. A review whose newest attempt has not finished is `still running`, and no
 second review is written under it.

 @module
 */

import { join, } from 'node:path';
import { isThere, } from './newest-finished-run-of-a-name.ts';
import {
  type ReviewCheck,
  type ReviewVerdict,
  CHECK_LIMITS_LINE,
  NO_REVIEWER,
} from './review-check-types.ts';
import {
  type ReadingAtAPath,
  type ReviewCoverage,
  commitReadingIn,
  coverageIn,
  coverageReasonsOf,
  fileProblemsIn,
  readingAtAPath,
} from './review-check-reading.ts';
import { readingOfAStatement, } from './reading-a-review.ts';
import { runTheCheckReads, } from './run-the-check-reads.ts';
import type { ReviewStatement, } from './review-statement-shape.ts';
import {
  readingFileOfAReview,
  reviewFileOfAReview,
  runNameOfAReview,
} from './reviewer-of-a-change.ts';

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
   What the run left before the statement is read: whether it is still going, and the run, summary and session that answer.
   */
  const runReading = await runTheCheckReads({
    review,
    reviewsFolder,
    runsFolder,
    reviewFile,
  },);
  if (runReading.stillRunning !== undefined) {
    return runReading.stillRunning;
  }
  /**
   The run's name.
   */
  const {runName} = runReading;
  /**
   That run's folder, or nothing when no attempt finished.
   */
  const {run} = runReading;
  /**
   What that run's summary says.
   */
  const {summary} = runReading;
  /**
   What that run's session records.
   */
  const {session} = runReading;
  /**
   The reviewer the run started on.
   */
  const {reviewer} = runReading;
  /**
   Why that run ended, when its summary could be read.
   */
  const {endReason} = runReading;
  /**
   Whether the run ended on its own terms.
   */
  const {endedByItself} = runReading;
  if (!(await isThere(reviewFile,))) {
    return {
      review,
      verdict: 'no review file',
      reasons: (run === undefined)
        ? [`no finished run of ${runName} is in ${runsFolder}`]
        : [`${run} ended ${endReason ?? 'with no reason its summary names'}, `
          + `${endedByItself ? 'so it ended by itself' : 'so it was cut'}, and it wrote no review.md`],
      reviewFile,
      ...((run === undefined) ? {} : { run, }),
      ...((endReason === undefined) ? {} : { endReason, }),
      endedByItself,
      builtPrograms: [],
      thin: false,
      reviewer,
    };
  }
  if (run === undefined) {
    return {
      review,
      verdict: 'cut',
      reasons: [`no finished run of ${runName} is in ${runsFolder}, so no summary or session says what the review read or ran`],
      reviewFile,
      endedByItself,
      builtPrograms: [],
      thin: false,
      reviewer,
    };
  }
  if (summary?.kind === 'unreadable') {
    return {
      review,
      verdict: 'cut',
      reasons: [`${run} left a summary that cannot be read, so what the review read and ran cannot be compared: ${
         summary.reason}`],
      reviewFile,
      run,
      endedByItself,
      builtPrograms: [],
      thin: false,
      reviewer,
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
      thin: false,
      reviewer,
    };
  }
  if (session?.kind !== 'read') {
    return {
      review,
      verdict: 'cut',
      reasons: [`${run} ended ${endReason ?? 'by itself'}, and its session cannot be read, so what it read and ran cannot be `
        + `compared: ${session?.reason ?? 'no session file'}`],
      reviewFile,
      run,
      ...((endReason === undefined) ? {} : { endReason, }),
      endedByItself,
      builtPrograms: [],
      thin: false,
      reviewer,
    };
  }
  /**
   The record of what the review read, which the statement's commit is compared with and a prose statement's commit falls back to.
   */
  const reading = await readingAtAPath(readingFileOfAReview({
    reviewsFolder,
    review,
  },),);
  /**
   The statement, the diff and the comparison with the session.
   */
  const statementReading = await readingOfAStatement({
    review,
    reviewsFolder,
    session: session.session,
    recordedCommit: (reading.kind === 'read') ? reading.reading
      .commit : '',
  },);
  if (statementReading.kind === 'unreadable') {
    return {
      review,
      verdict: 'statement missing or not matching its session',
      reasons: [...statementReading.reasons,],
      reviewFile,
      run,
      ...((endReason === undefined) ? {} : { endReason, }),
      endedByItself,
      builtPrograms: [],
      thin: false,
      reviewer,
    };
  }
  /**
   What the review says it read of the files its brief lists.
   */
  const coverage = coverageIn({
    statement: statementReading.statement,
    diff: statementReading.diff,
    briefFiles: statementReading.briefFiles,
  },);
  /**
   The line the statement's own line count earns, when it does not name the diff's.
   */
  const lineCountProblem = (statementReading.statement
    .diff
    .lines
    === statementReading.diff
      .lines)
    ? []
    : [`the statement says the diff holds ${String(statementReading.statement
      .diff
      .lines,)} lines and it holds ${
       String(statementReading.diff
         .lines,)}`];
  /**
   The record of what the review read, which the statement's commit is compared with.
   */
  const readingFile = readingFileOfAReview({
    reviewsFolder,
    review,
  },);
  /**
   What the statement's commit and the review's record say together.
   */
  const commit = commitReadingIn({
    recorded: reading,
    stated: statementReading.statement
      .commit,
    readingFile,
  },);
  /**
   What the comparison found that does not fit.
   */
  const problems = [
    ...lineCountProblem,
    ...((commit.kind === 'another commit') ? [commit.problem,] : []),
    ...fileProblemsIn({
      statement: statementReading.statement,
      briefFiles: statementReading.briefFiles,
    },),
    ...statementReading.comparison
      .problems,
  ];
  /**
   What the session cannot confirm either way.
   */
  const uncompared = [
    ...((commit.kind === 'cannot be compared') ? [commit.uncompared,] : []),
    ...statementReading.comparison
      .uncompared,
    ...((session.session
      .codemodeCalls
      === 0)
      ? []
      : [`${String(session.session
        .codemodeCalls,)} codemode call(s) are not read, so a command they ran cannot be compared`]),
  ];
  /**
   Whether the review showed a run at all: a program a build wrote, or a build task.
   */
  const ranSomething = (statementReading.comparison
    .builtPrograms
    .length
    > 0)
    || statementReading.comparison
      .buildCommandRan;
  /**
   Whether the review is thin: a brief file not read whole, unnamed files not read whole, or no run shown.
   */
  const thin = (coverage.filesNotReadWhole
    .length
    > 0) || coverage.unnamedNotWhole
    || (!ranSomething);
  /**
   The reasons every verdict from here names.
   */
  const reasons = [
    ...coverageReasonsOf(coverage,),
    ...uncompared,
  ];
  if (problems.length > 0) {
    return {
      review,
      verdict: 'statement missing or not matching its session',
      reasons: [
        ...problems,
        ...reasons,
      ],
      reviewFile,
      run,
      ...((endReason === undefined) ? {} : { endReason, }),
      endedByItself,
      statement: statementReading.statement,
      coverage,
      builtPrograms: statementReading.comparison
        .builtPrograms,
      thin,
      reviewer,
    };
  }
  if (!ranSomething) {
    return {
      review,
      verdict: 'ran nothing',
      reasons: [
        'thin: no command it lists ran a program a build wrote and no command ran a build task, and no command its session '
          + 'records does',
        ...reasons,
      ],
      reviewFile,
      run,
      ...((endReason === undefined) ? {} : { endReason, }),
      endedByItself,
      statement: statementReading.statement,
      coverage,
      builtPrograms: [],
      thin: true,
      reviewer,
    };
  }
  return {
    review,
    verdict: 'complete',
    reasons: [
      ...((statementReading.comparison
        .builtPrograms
        .length
        > 0)
        ? [`${String(statementReading.comparison
          .builtPrograms
          .length,)} command(s) ran a program a build wrote in this review's `
          + `worktree or its own folder, which the statement names for commit ${statementReading.statement
            .commit}: ${
           statementReading.comparison
             .builtPrograms
             .join(', ')}`]
        : [`${String(statementReading.comparison
          .buildTaskCommands
          .length,)} command(s) ran a build task: `
          + `${statementReading.comparison
            .buildTaskCommands
            .join(', ')}; no program's path is named, so the build itself is the `
          + 'statement\'s claim']),
      ...reasons,
      `the run ended ${endReason ?? 'with no reason its summary names'}`,
      ...((thin)
        ? ['thin: the brief files it did not read whole are read by a second review']
        : []),
    ],
    reviewFile,
    run,
    ...((endReason === undefined) ? {} : { endReason, }),
    endedByItself,
    statement: statementReading.statement,
    coverage,
    builtPrograms: statementReading.comparison
      .builtPrograms,
    thin,
    reviewer,
  };
}
