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

import { endedOnItsOwnTerms, } from './end-reasons.ts';
import {
  NO_FINISHED_RUN_OF_THAT_NAME,
  isThere,
  newestFinishedRunOf,
} from './newest-finished-run-of-a-name.ts';
import {
  type ReviewCheck,
  type ReviewVerdict,
  CHECK_LIMITS_LINE,
} from './review-check-types.ts';
import {
  type ReadingAtAPath,
  type ReviewCoverage,
  coverageIn,
  coverageReasonsOf,
  fileProblemsIn,
  readingAtAPath,
  readingOfAStatement,
  commitReadingIn
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
  summaryOfTheRun,
} from './session-of-a-finished-run.ts';

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
   What that run's summary says, read through one descriptor opened without waiting.
   */
  const summary = (run === undefined) ? undefined : await summaryOfTheRun({
    runsFolder,
    run,
  },);
  /**
   Why that run ended, when its summary could be read.
   */
  const endReason = (summary?.kind === 'read') ? summary.endReason : undefined;
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
      thin: false,
      reviewer: {
        provider: '',
        model: '',
      },
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
      thin: false,
      reviewer: {
        provider: '',
        model: '',
      },
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
      reviewer: {
        provider: '',
        model: '',
      },
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
      reviewer: {
        provider: '',
        model: '',
      },
    };
  }
  /**
   What the run's session records, or why it cannot be read. The summary was read above, so it names the session file.
   */
  const session = await sessionOfTheRun({ sessionFile: (summary?.kind === 'read') ? summary.sessionFile : '', },);
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
      thin: false,
      reviewer: {
        provider: '',
        model: '',
      },
    };
  }
  /**
   The reviewer the run started on, which a second start picks another of.
   */
  const {reviewer} = session.session;
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
      thin: false,
      reviewer,
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
    recorded: await readingAtAPath(readingFile,),
    stated: reading.statement
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
      statement: reading.statement,
      diff: reading.diff,
    },),
    ...reading.comparison
      .problems,
  ];
  /**
   What the session cannot confirm either way.
   */
  const uncompared = [
    ...((commit.kind === 'cannot be compared') ? [commit.uncompared,] : []),
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
      thin: coverage.filesNotReadWhole
        .length
        > 0,
      reviewer,
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
        `thin: no command it lists ran a program built from commit ${reading.statement
          .commit}, and no command its session records `
          + 'does',
        ...((reading.comparison
          .buildCommandRan)
          ? ['the session records a build command, but no command ran a program it wrote']
          : ['the session records no build command either']),
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
      thin: true,
      reviewer,
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
      ...((reading.comparison
        .buildCommandRan)
        ? ['the session records a build command, so a program from that build is what ran']
        : ['the session records no build command, so the build itself is the statement\'s own claim']),
      ...((coverage.filesNotReadWhole
        .length
        === 0)
        ? []
        : ['thin: it did not read every file it was given whole, which a second review reads']),
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
    thin: coverage.filesNotReadWhole
      .length
      > 0,
    reviewer,
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
