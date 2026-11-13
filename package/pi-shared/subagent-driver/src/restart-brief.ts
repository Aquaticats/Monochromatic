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
import type { ReviewCheck, } from './checking-a-review.ts';
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
  baseReviewNameOf,
  restartReviewNameOf,
  runWordOf,
} from './review-runs-of-a-change.ts';
import {
  REVIEWING_MODEL,
  REVIEWING_PROVIDER,
  branchOfAReview,
  briefFileOfAReview,
  diffFileOfAReview,
  readingFileOfAReview,
  reproductionsFolderOfAReview,
  reviewFileOfAReview,
  runNameOfAReview,
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
   The entry a launcher starts.
   */
  readonly entry: BatchEntry;

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
 Reads the sentence a restart's brief opens with about what the earlier run's statement says.

 @param check - what the earlier run's check answered

 @returns the sentence

 @example
 ```ts
 const stated = statementSentenceOf({ check });
 ```
 */
function statementSentenceOf({ check, }: { readonly check: ReviewCheck },): string {
  if (check.statement === undefined) {
    return 'It wrote no statement of what it read and ran.';
  }
  /**
   What the earlier run's statement says it read.
   */
  const {coverage} = check;
  /**
   The lines of the diff it read.
   */
  const linesRead = coverage?.linesRead ?? 0;
  /**
   The lines the diff holds.
   */
  const diffLines = coverage?.diffLines ?? 0;
  /**
   The files it read whole.
   */
  const whole = coverage?.filesReadWhole
    .length
    ?? 0;
  /**
   The files it did not read whole.
   */
  const notWhole = coverage?.filesNotReadWhole ?? [];
  /**
   The sentence about the files it did not read whole, when it left any.
   */
  const notWholeNote = (notWhole.length === 0) ? '' : `; it did not read whole: ${notWhole.join(', ')}`;
  return `Its statement says it read ${String(linesRead,)} of the diff's ${String(diffLines,)} lines, and `
    + `${String(whole,)} of ${String(whole + notWhole.length,)} files whole${notWholeNote}. `
    + `It ran ${String(check.builtPrograms
      .length,)} command(s) through a program built from commit `
    + `${check.statement
      .commit}.`;
}

/**
 Reads the sentence about where the earlier run's own notes are.

 @param review - the review the restart answers

 @param reviewsFolder - folder that holds one folder per review

 @returns the sentence

 @example
 ```ts
 const notes = await notesSentenceOf({ review: 'a', reviewsFolder: '/reviews' });
 ```
 */
async function notesSentenceOf({
  review,
  reviewsFolder,
}: {
  readonly review: string;
  readonly reviewsFolder: string;
},): Promise<string> {
  /**
   The earlier run's own review file.
   */
  const reviewFile = reviewFileOfAReview({
    reviewsFolder,
    review,
  },);
  /**
   What is at the earlier run's review file's path.
   */
  const notes = await textAtAPath({ path: reviewFile, },);
  if (notes.kind === 'no file') {
    return 'It wrote no review file, so its findings were not saved.';
  }
  if (notes.kind === 'not a file this reader reads') {
    return `Its review file is at \`${reviewFile}\` and cannot be read: ${notes.reason}.`;
  }
  return `Its own review file is at \`${reviewFile}\`; read it before you begin, and do not repeat a finding it makes unless `
    + `what you read shows that finding is wrong. Its reproduction scripts are under `
    + `\`${reproductionsFolderOfAReview({
      reviewsFolder,
      review,
    },)}\`.`;
}

/**
 Reads the paragraph a restart's brief opens with, which says what the earlier run did and did not do.

 @param check - what the earlier run's check answered

 @param number - which run of the review the restart is

 @param reviewsFolder - folder that holds one folder per review

 @returns the paragraph, ending with a blank line

 @example
 ```ts
 const earlier = await earlierRunTextOf({ check, number: 2, reviewsFolder: '/reviews' });
 ```
 */
export async function earlierRunTextOf({
  check,
  number,
  reviewsFolder,
}: {
  readonly check: ReviewCheck;
  readonly number: number;
  readonly reviewsFolder: string;
},): Promise<string> {
  /**
   The review the restart answers.
   */
  const baseReview = baseReviewNameOf({ review: check.review, },);
  /**
   The run whose work the restart continues.
   */
  const run = runNameOfAReview({ review: check.review, },);
  /**
   What the earlier run's ending says.
   */
  const ending = (check.run === undefined)
    ? `No finished run of ${run} is in the runs folder, so the earlier run left no summary.`
    : `${run} ended ${check.endReason ?? 'with no reason its summary names'}, `
      + `${check.endedByItself ? 'so it ended by itself' : 'so it was cut rather than ended by itself'}.`;
  return `This is the ${runWordOf(number,)} run of the review \`${baseReview}\`. ${ending} `
    + `Its check answered \`${check.verdict}\`. `
    + `${statementSentenceOf({ check, },)} `
    + `${await notesSentenceOf({
      review: check.review,
      reviewsFolder,
    },)}\n\n`;
}

/**
 Says whether the earlier review read every file of its diff whole.

 @param check - what the earlier run's check answered

 @returns whether its reading was complete and left files unread whole

 @example
 ```ts
 const partial = leftFilesUnread({ check });
 ```
 */
function leftFilesUnread({ check, }: { readonly check: ReviewCheck },): boolean {
  return (check.verdict === 'complete')
    && ((check.coverage
      ?.filesNotReadWhole
      .length
      ?? 0) > 0);
}

/**
 Reads the files a restart's reviewer should read: the files the earlier review did not read whole, when its reading was
 complete, and the whole diff when the earlier run left no complete reading.

 @param check - what the earlier run's check answered

 @param diff - the earlier review's diff

 @returns the files to read, from the top of the checkout

 @example
 ```ts
 const files = filesForARestart({ check, diff: await readFile('change.diff', 'utf8') });
 ```
 */
export function filesForARestart({
  check,
  diff,
}: {
  readonly check: ReviewCheck;
  readonly diff: string;
},): readonly string[] {
  if (leftFilesUnread({ check, },)) {
    return check.coverage
      ?.filesNotReadWhole
      ?? [];
  }
  return filesInDiff({ diff, },)
    .files;
}

/**
 Reads the batch list that starts one restart.

 @param entry - the entry the launcher takes

 @returns the list's text

 @example
 ```ts
 const shown = batchListOfOne({ entry });
 ```
 */
export function batchListOfOne({ entry, }: { readonly entry: BatchEntry },): string {
  return `${JSON.stringify(
    [entry,],
    undefined,
    2
  )}\n`;
}

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
}: {
  readonly check: ReviewCheck;
  readonly number: number;
  readonly reviewsFolder: string;
  readonly runsFolder: string;
  readonly rulesFile: string;
  readonly focus: string;
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
   The entry the launcher starts.
   */
  const entry: BatchEntry = {
    name: runNameOfAReview({ review: restartReview, },),
    branch: branchOfAReview({ review: restartReview, },),
    base: reading.commit,
    prompt: briefFile,
    cpu: BATCH_DEFAULT_CPU_PERCENT,
    memory: BATCH_DEFAULT_MEMORY,
    tasks: BATCH_DEFAULT_TASK_COUNT,
    wall: REVIEW_WALL_SECONDS,
    turnCap: REVIEW_TURN_CAP,
    provider: REVIEWING_PROVIDER,
    model: REVIEWING_MODEL,
  };
  return {
    plan: {
      review: check.review,
      restartReview,
      number,
      briefFile,
      diffFile,
      commit: reading.commit,
      files,
      batchFile,
      entry,
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
            },
            undefined,
            2
          )}\n`,
        },
        {
          path: briefFile,
          text: briefOfAReview({
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
          },),
        },
        {
          path: batchFile,
          text: batchListOfOne({ entry, },),
        },
      ],
      line: `${restartReview}: the ${runWordOf(number,)} run of review ${review}, written at ${briefFile}; branch `
        + `${branchOfAReview({ review: restartReview, },)} at ${reading.commit}; its entry is in ${batchFile}`,
    },
  };
}
