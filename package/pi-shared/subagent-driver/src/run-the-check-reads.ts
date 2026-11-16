/**
 Reads what a review's run left before the check reads the statement: the newest attempt, whether it is still going, and the summary
 and session that say what the review read and ran.

 @module
 */

import { endedOnItsOwnTerms, } from './end-reasons.ts';
import {
  type ReviewCheck,
  NO_REVIEWER,
} from './review-check-types.ts';
import {
  NO_FINISHED_RUN_OF_THAT_NAME,
  newestFinishedRunOf,
  runningAttemptOf,
} from './newest-finished-run-of-a-name.ts';
import {
  type SessionOfARun,
  sessionOfTheRun,
  summaryOfTheRun,
} from './session-of-a-finished-run.ts';
import { runNameOfAReview, } from './reviewer-of-a-change.ts';

/**
 What the check learns from a run before it reads the statement.
 */
export type RunReading = {
  /**
   The run's name.
   */
  readonly runName: string;

  /**
   The check to print when the newest attempt has not finished.
   */
  readonly stillRunning?: ReviewCheck;

  /**
   The newest finished attempt, when one finished.
   */
  readonly run?: string;

  /**
   What that attempt's summary says, when it wrote one.
   */
  readonly summary?: Awaited<ReturnType<typeof summaryOfTheRun>>;

  /**
   What that attempt's session records, when it can be read.
   */
  readonly session?: SessionOfARun;

  /**
   The reviewer the run started on.
   */
  readonly reviewer: {
    readonly provider: string;
    readonly model: string;
  };

  /**
   Why the run ended, when its summary names a reason.
   */
  readonly endReason?: string;

  /**
   Whether the run ended on its own terms.
   */
  readonly endedByItself: boolean;
};

/**
 Reads the newest attempt of a review's run and what it left.

 @param review - the change the review read

 @param reviewsFolder - folder that holds one folder per review

 @param runsFolder - folder the driver writes its runs into

 @param reviewFile - the review file's path

 @returns whether the review is still going, or the run, summary, session and reviewer of the attempt that finished

 @example
 ```ts
 const reading = await runTheCheckReads({ review: 'a', reviewsFolder: '/reviews', runsFolder: '/runs', reviewFile: '/reviews/a/review.md' });
 ```
 */
export async function runTheCheckReads({
  review,
  reviewsFolder: _reviewsFolder,
  runsFolder,
  reviewFile,
}: {
  readonly review: string;
  readonly reviewsFolder: string;
  readonly runsFolder: string;
  readonly reviewFile: string;
},): Promise<RunReading> {
  /**
   The run's name, which every attempt's folder starts with.
   */
  const runName = runNameOfAReview({ review, },);
  /**
   The newest attempt, when it has not finished.
   */
  const running = await runningAttemptOf({
    runsFolder,
    runName,
  },);
  if (running.run !== undefined) {
    return {
      runName,
      reviewer: NO_REVIEWER,
      endedByItself: false,
      stillRunning: {
        review,
        verdict: 'still running',
        reasons: [`${running.run} is the newest attempt of ${runName} and has not finished, so the review is still running and `
          + 'nothing is written under it'],
        reviewFile,
        run: running.run,
        endedByItself: false,
        builtPrograms: [],
        thin: false,
        reviewer: NO_REVIEWER,
      },
    };
  }
  /**
   The newest attempt that finished.
   */
  const newest = await newestFinishedRunOf({
    runsFolder,
    runName,
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
   What that run's session records, read even when the run was cut, so its model is known for the second start.
   */
  const session = (summary?.kind === 'read') ? await sessionOfTheRun({ sessionFile: summary.sessionFile, },) : undefined;
  /**
   Why that run ended, when its summary could be read.
   */
  const endReason = (summary?.kind === 'read') ? summary.endReason : undefined;
  return {
    runName,
    ...((run === undefined) ? {} : { run, }),
    ...((summary === undefined) ? {} : { summary, }),
    ...((session === undefined) ? {} : { session, }),
    reviewer: (session?.kind === 'read') ? session.session
      .reviewer : NO_REVIEWER,
    ...((endReason === undefined) ? {} : { endReason, }),
    endedByItself: endedOnItsOwnTerms(endReason,),
  };
}
