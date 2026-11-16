/**
 What a restart reads and the paragraph that tells its reviewer what the earlier run did: the files the earlier review did not read
 whole, the diff of those files, the brief that opens with the earlier run's facts, and the batch entry a launcher takes.

 The brief is the review brief the package already writes, with one extra paragraph in front, so a restart is briefed the way every
 review is and its reader is told what the first run left. Nothing here writes a file: `restarting-a-review.ts` writes a plan once
 it has checked that nothing at those paths holds somebody's text.

 @module
 */

import { join, } from 'node:path';
import {
  BATCH_DEFAULT_CPU_PERCENT,
  BATCH_DEFAULT_MEMORY,
  BATCH_DEFAULT_TASK_COUNT,
  type BatchEntry,
} from './batch-list.ts';
import { checkLineOf, } from './review-check-line.ts';
import type { ReviewCheck, } from './review-check-types.ts';
import {
  readingAtAPath,
  textAtAPath,
} from './review-check-reading.ts';
import {
  diffOfOnlyFiles,
  filesInDiff,
} from './review-diff.ts';
import { briefOfAReview, } from './review-brief.ts';
import {
  batchListOfOne,
  earlierRunTextOf,
  filesForARestart,
  leftFilesUnread,
  owedNoteTextOf,
} from './restart-brief-text.ts';
import {
  baseReviewNameOf,
  restartReviewNameOf,
  runWordOf,
} from './review-runs-of-a-change.ts';
import {
  type ReviewerOfAChange,
  branchOfAReview,
  briefFileOfAReview,
  diffFileOfAReview,
  readingFileOfAReview,
  reproductionsFolderOfAReview,
  reviewFileOfAReview,
  runNameOfAReview,
  reviewerIsReachableByPi,
} from './reviewer-of-a-change.ts';
import {
  REVIEW_TURN_CAP,
  REVIEW_WALL_SECONDS,
} from './reviews-of-finished-branches.ts';

/**
 One restart's plan: the files it writes, the entry a launcher takes, and the line to print.
 */
export type RestartPlan = {
  /**
   The review the restart reads.
   */
  readonly review: string;

  /**
   Folder that holds one folder per review, so a caller needs no path arithmetic to find one.
   */
  readonly reviewsFolder: string;

  /**
   The review the restart is, which reads the change under a name of its own.
   */
  readonly restartReview: string;

  /**
   Which run of the review this restart is, counting the review itself as the first.
   */
  readonly number: number;

  /**
   The brief the restart's reviewer reads.
   */
  readonly briefFile: string;

  /**
   The diff the restart's reviewer reads.
   */
  readonly diffFile: string;

  /**
   The commit the restart's worktree starts at.
   */
  readonly commit: string;

  /**
   The files the restart's reviewer reads, from the top of the checkout.
   */
  readonly files: readonly string[];

  /**
   The batch list a launcher takes, holding this one entry.
   */
  readonly batchFile: string;

  /**
   The entry a launcher starts, when a Pi model serves the reviewer this second review is owed to.
   */
  readonly entry?: BatchEntry;

  /**
   The reviewer this second review is for: the other of the pair the first review did not run on.
   */
  readonly reviewer: ReviewerOfAChange;

  /**
   The note that says the second review is owed to a harness Pi cannot start, when there is one.
   */
  readonly owedNoteFile?: string;

  /**
   The text of each file the plan writes, by path.
   */
  readonly texts: readonly {
    readonly path: string;
    readonly text: string;
  }[];

  /**
   The line to print when the plan is written.
   */
  readonly line: string;
};

/**
 Builds one restart's plan without writing it.

 @param check - what the earlier run's check answered

 @param number - which run of the review the restart is

 @param reviewsFolder - folder that holds one folder per review

 @param runsFolder - folder the driver writes its runs into

 @param rulesFile - the rules file the brief names first

 @param focus - what the reviewer looks hardest at

 @returns the plan, or why it cannot be planned

 @example
 ```ts
 const planned = await restartPlanOf({ check, number: 2, reviewsFolder: '/reviews', runsFolder: '/runs', rulesFile: '/repo/rules.md', focus: 'data loss' });
 ```
 */
export async function restartPlanOf({
  check,
  number,
  reviewsFolder,
  runsFolder,
  rulesFile,
  focus,
  reviewer,
}: {
  readonly check: ReviewCheck;
  readonly number: number;
  readonly reviewsFolder: string;
  readonly runsFolder: string;
  readonly rulesFile: string;
  readonly focus: string;
  readonly reviewer: ReviewerOfAChange;
},): Promise<{
  readonly plan?: RestartPlan;
  readonly cannot?: string
}> {
  /**
   The review the restart answers.
   */
  const review = baseReviewNameOf({ review: check.review, },);
  /**
   The review this restart is.
   */
  const restartReview = restartReviewNameOf({
    review,
    number,
  },);
  /**
   The record of what the earlier run read.
   */
  const readingFile = readingFileOfAReview({
    reviewsFolder,
    review: check.review,
  },);
  /**
   What the earlier run read, or why it cannot be read.
   */
  const read = await readingAtAPath(readingFile,);
  if (read.kind === 'unreadable') {
    return {
      cannot: `${restartReview}: ${readingFile} names no commit, ranges and source folder, so where a restart would start `
        + 'cannot be read',
    };
  }
  /**
   The record itself, which says the commit and the source folder the restart reads.
   */
  const { reading, } = read;
  /**
   The diff the earlier review read.
   */
  const earlierDiffFile = diffFileOfAReview({
    reviewsFolder,
    review: check.review,
  },);
  /**
   What that diff holds.
   */
  const earlierDiff = await textAtAPath({ path: earlierDiffFile, },);
  if (earlierDiff.kind !== 'the text') {
    return {
      cannot: `${restartReview}: ${earlierDiffFile} is `
        + `${(earlierDiff.kind === 'no file') ? 'not there' : `no file this reader reads (${earlierDiff.reason})`}, so what a `
        + 'restart would read cannot be read',
    };
  }
  /**
   The diff's own text.
   */
  const earlierDiffText = earlierDiff.text;
  /**
   The files the restart reads.
   */
  const files = filesForARestart({
    check,
    diff: earlierDiffText,
  },);
  /**
   The diff the restart reads, which is only those files' part when the earlier review left files unread whole.
   */
  const diff = leftFilesUnread({ check, },)
    ? diffOfOnlyFiles({
      diff: earlierDiffText,
      files,
    },)
    : earlierDiffText;
  /**
   The brief's path.
   */
  const briefFile = briefFileOfAReview({
    reviewsFolder,
    review: restartReview,
  },);
  /**
   The diff's path.
   */
  const diffFile = diffFileOfAReview({
    reviewsFolder,
    review: restartReview,
  },);
  /**
   The restart's own record of what it read, so a fix of its findings starts where it read.
   */
  const restartReadingFile = readingFileOfAReview({
    reviewsFolder,
    review: restartReview,
  },);
  /**
   The batch list the launcher takes.
   */
  const batchFile = join(
    reviewsFolder,
    review,
    `restart-${String(number,)}-batch.json`,
  );
  /**
   Whether a Pi run can be started on the reviewer this second review is owed to.
   */
  const reachable = reviewerIsReachableByPi(reviewer,);
  /**
   The entry the launcher starts, when Pi serves the reviewer.
   */
  const entry = reachable ? {
    name: runNameOfAReview({ review: restartReview, },),
    branch: branchOfAReview({ review: restartReview, },),
    base: reading.commit,
    prompt: briefFile,
    cpu: BATCH_DEFAULT_CPU_PERCENT,
    memory: BATCH_DEFAULT_MEMORY,
    tasks: BATCH_DEFAULT_TASK_COUNT,
    wall: REVIEW_WALL_SECONDS,
    turnCap: REVIEW_TURN_CAP,
    provider: reviewer.provider,
    model: reviewer.model,
  } : undefined;
  /**
   The note that a harness with the reviewer must take the second review, when no Pi model serves it.
   */
  const owedNoteFile = reachable
    ? undefined
    : join(
      reviewsFolder,
      review,
      `restart-${String(number,)}-second-review-owed.md`,
    );
  /**
   The brief's text, which opens with what the earlier run did and did not do.
   */
  const brief = briefOfAReview({
    review: restartReview,
    commit: reading.commit,
    source: reading.source,
    files,
    focus,
    reviewsFolder,
    runsFolder,
    rulesFile,
    earlier: await earlierRunTextOf({
      check,
      number,
      reviewsFolder,
    },),
  },);
  return {
    plan: {
      review: check.review,
      reviewsFolder,
      restartReview,
      number,
      briefFile,
      diffFile,
      commit: reading.commit,
      files,
      batchFile,
      ...((entry === undefined) ? {} : { entry, }),
      reviewer,
      ...((owedNoteFile === undefined) ? {} : { owedNoteFile, }),
      texts: [
        {
          path: diffFile,
          text: diff,
        },
        {
          path: restartReadingFile,
          text: `${JSON.stringify(
            {
              ...reading,
              review: restartReview,
              restartOf: review,
              files,
              reviewer: {
                name: reviewer.name,
                provider: reviewer.provider,
                model: reviewer.model,
              },
            },
            undefined,
            2
          )}\n`,
        },
        {
          path: briefFile,
          text: brief,
        },
        ...((entry === undefined) ? [] : [{
          path: batchFile,
          text: batchListOfOne({ entry, },),
        },]),
        ...((owedNoteFile === undefined) ? [] : [{
          path: owedNoteFile,
          text: owedNoteTextOf({
            check,
            restartReview,
            reviewer,
            briefFile,
            commit: reading.commit,
          },),
        },]),
      ],
      line: reachable
        ? `${restartReview}: the ${runWordOf(number,)} run of review ${review}, on ${reviewer.name}, written at ${briefFile}; `
          + `branch ${branchOfAReview({ review: restartReview, },)} at ${reading.commit}; its entry is in ${batchFile}`
        : `${restartReview}: a second review of ${review} is owed on ${reviewer.name}, which no Pi model on this machine is; `
          + `its brief is at ${briefFile} and the note for a harness that has it is at ${owedNoteFile}; nothing was started`,
    },
  };
}
