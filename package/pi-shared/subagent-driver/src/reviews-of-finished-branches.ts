/**
 Starts one review per finished branch: each review's brief is written first, then its run is started through the retrying start
 on the qwen plan's review pool, and a review that ended without its review file has its last message saved as that file.

 A branch the integration already merged, a branch whose change leaves no code under the reviewed source folder, and a change
 already reviewed are each said in one line and the rest go on: the scratch starter stopped at the first of them, and two
 reviews of round 231 were lost that way with no line anyone read.

 @module
 */

import {
  BATCH_DEFAULT_CPU_PERCENT,
  BATCH_DEFAULT_MEMORY,
  BATCH_DEFAULT_TASK_COUNT,
  BATCH_RETRY_PERIOD_SECONDS,
  BATCH_STAGGER_MILLISECONDS,
  entryLineOf,
  type StartedEntry,
} from './batch-list.ts';
import { rangeOfABranch, } from './material-of-a-review.ts';
import {
  branchOfAReview,
  changeNameOfBranch,
  REVIEWING_MODEL,
  REVIEWING_PROVIDER,
  runNameOfAReview,
} from './reviewer-of-a-change.ts';
import { TURN_CAP_MARGIN, } from './turn-cap-of-finished-runs.ts';

/**
 Seconds a review's start keeps asking for a slot, which is the scratch starter's own six hours: a review takes one slot of the
 review pool, and a pool that is full clears as the runs on it end, so the wait is bounded by the work ahead rather than by a
 guess about the pool.
 */
export const REVIEW_START_DEADLINE_SECONDS = 21_600;

/**
 Seconds a review may last, which is what the scratch starter gave one: a review reads a whole change in full, builds it and
 runs its reproductions, so it lasts longer than the fix it reviews.
 */
export const REVIEW_WALL_SECONDS = 14_400;

/**
 The longest a review of a finished branch ran, in turns, among the 28 measured on 2026-11-11 while the Kimi flex pool read 22:
 they ended at turn 16 to 37. A review gets a quarter above the longest of them, the same margin the worktree launcher sizes a
 cap with, because a review whose cap is reached loses its findings.
 */
export const LONGEST_MEASURED_REVIEW_TURNS = 37;

/**
 How many model turns a review may start.
 */
export const REVIEW_TURN_CAP: number = Math.ceil(LONGEST_MEASURED_REVIEW_TURNS * TURN_CAP_MARGIN);

/**
 One start with the review it read, kept together so no lookup can come up short.
 */
type StartWithItsReview = {
  /**
   The change the start's run reads.
   */
  readonly review: string;

  /**
   The start, which resolves when its run has ended.
   */
  readonly start: Promise<StartOfOneReview>;
};

/**
 What starting the reviews answered: the lines every step printed, and the status each start ended with.
 */
export type StartedReviews = {
  /**
   Every line, in the order the steps printed them.
   */
  readonly lines: readonly string[];

  /**
   The status each start ended with, or `none` when it could not be started at all.
   */
  readonly statuses: readonly string[];
};

/**
 What writing one review's brief answered.
 */
export type BriefOfOneReview = {
  /**
   Whether the brief was written, which is false when the change leaves nothing to review.
   */
  readonly written: boolean;

  /**
   The line to print either way.
   */
  readonly line: string;
};

/**
 What starting one review's run answered: the retrying start's status, or why it could not be started, and where the whole of what
 a failed start printed was kept.
 */
export type StartOfOneReview = StartedEntry & {
  /**
   The file a failed start's whole output was kept in, when it was kept.
   */
  readonly keptAt?: string;
};

/**
 The seams the starter asks, so a test can answer them without a unit, a worktree or a process.
 */
export type ReviewStartingSeams = {
  /**
   Whether a review of that change already wrote its review file.
   */
  readonly reviewIsWritten: (request: { readonly review: string }) => Promise<boolean>;

  /**
   The branch's tip commit, which the review's worktree starts at.
   */
  readonly branchTip: (request: { readonly branch: string }) => Promise<string>;

  /**
   The integration branch's own first-parent merges, each as `<merge> <parent>...`.
   */
  readonly integrationMerges: () => Promise<readonly string[]>;

  /**
   Writes one review's brief and the diff it reads.
   */
  readonly brief: (request: {
    readonly review: string;
    readonly commit: string;
    readonly range: string;
  }) => Promise<BriefOfOneReview>;

  /**
   Starts one review's run through the retrying start, with the words that command reads.
   */
  readonly start: (request: {
    readonly review: string;
    readonly words: readonly string[];
  }) => Promise<StartOfOneReview>;

  /**
   Saves a review that ended without its review file, answering the line to print.
   */
  readonly save: (request: { readonly review: string }) => Promise<{ readonly line: string }>;

  /**
   How the starter passes time between one start and the next.
   */
  readonly wait: (milliseconds: number) => Promise<void>;
};

/**
 Reads the words the retrying start is given for one review: the pool gate's own words after the separator, which are the
 worktree launcher's.

 The review's worktree starts at the commit under review, on a branch of its own, and its run is named after the change alone, so
 no name says which model reads it.

 @param review - the change the review reads

 @param commit - the commit under review, which the worktree starts at

 @param briefFile - the brief the reviewer reads

 @param runsFolder - folder the driver writes its runs into

 @param checkoutFolder - the top of the checkout the worktree is made in

 @returns the retrying start's words

 @example
 ```ts
 const words = retryWordsOfAReview({ review: 'a', commit: 'abc123', briefFile: '/reviews/a/brief.md', runsFolder: '/runs', checkoutFolder: '/repo', },);
 ```
 */
export function retryWordsOfAReview({
  review,
  commit,
  briefFile,
  runsFolder,
  checkoutFolder,
}: {
  readonly review: string;
  readonly commit: string;
  readonly briefFile: string;
  readonly runsFolder: string;
  readonly checkoutFolder: string;
}): readonly string[] {
  return [
    `--deadline-seconds=${String(REVIEW_START_DEADLINE_SECONDS,)}`,
    `--period-seconds=${String(BATCH_RETRY_PERIOD_SECONDS,)}`,
    '--',
    `--checkout-folder=${checkoutFolder}`,
    `--runs-folder=${runsFolder}`,
    `--name=${runNameOfAReview({ review, },)}`,
    `--branch=${branchOfAReview({ review, },)}`,
    `--base=${commit}`,
    `--prompt=${briefFile}`,
    `--cpu=${String(BATCH_DEFAULT_CPU_PERCENT,)}`,
    `--memory=${BATCH_DEFAULT_MEMORY}`,
    `--tasks=${String(BATCH_DEFAULT_TASK_COUNT,)}`,
    `--wall=${String(REVIEW_WALL_SECONDS,)}`,
    `--turn-cap=${String(REVIEW_TURN_CAP,)}`,
    `--provider=${REVIEWING_PROVIDER}`,
    `--model=${REVIEWING_MODEL}`,
  ];
}

/**
 Starts one review per branch, one after another, and answers the line each step printed.

 @param branches - the branches a review is wanted for

 @param integration - the integration branch a branch's change is read against

 @param runsFolder - folder the driver writes its runs into

 @param checkoutFolder - the top of the checkout the worktrees are made in

 @param briefFileOf - the brief file's path for one review

 @param seams - the answers the starter asks, which a test replaces

 @returns one line per review and per start, then the statuses every start ended with

 @example
 ```ts
 const lines = await startReviews({ branches, integration, runsFolder, checkoutFolder, briefFileOf, seams, },);
 ```
 */
export async function startReviews({
  branches,
  integration,
  runsFolder,
  checkoutFolder,
  briefFileOf,
  seams,
}: {
  readonly branches: readonly string[];
  readonly integration: string;
  readonly runsFolder: string;
  readonly checkoutFolder: string;
  readonly briefFileOf: (request: { readonly review: string }) => string;
  readonly seams: ReviewStartingSeams;
}): Promise<StartedReviews> {
  /**
   The lines every step printed, in the order they happened.
   */
  const lines: string[] = [];
  /**
   One start per review, each resolving when its run has ended, kept beside the review it reads.
   */
  const starts: {
    readonly review: string;
    readonly start: Promise<StartOfOneReview>
  }[] = [];
  /* oxlint-disable eslint/no-await-in-loop -- one branch after another, because each brief and start is written and made before
     the next, and the stagger is what keeps their slots in the order given. See doc/lint/sequential-awaits.md. */
  for (const branch of branches) {
    /**
     The change this branch is reviewed as.
     */
    const review = changeNameOfBranch({ branch, },);
    /**
     The run that reads it.
     */
    const run = runNameOfAReview({ review, },);
    if (await seams.reviewIsWritten({ review, },)) {
      lines.push(`${run}: already reviewed`,);
      continue;
    }
    /**
     The commit the review's worktree starts at.
     */
    const commit = await seams.branchTip({ branch, },);
    /**
     The range that names what the branch changed.
     */
    const range = rangeOfABranch({
      branch,
      tip: commit,
      integration,
      merges: await seams.integrationMerges(),
    },);
    /**
     What writing the brief answered.
     */
    const briefed = await seams.brief({
      review,
      commit,
      range,
    },);
    lines.push(briefed.line,);
    if (!briefed.written) {
      continue;
    }
    starts.push({
      review,
      start: seams.start({
        review,
        words: retryWordsOfAReview({
          review,
          commit,
          briefFile: briefFileOf({ review, },),
          runsFolder,
          checkoutFolder,
        },),
      },),
    },);
    await seams.wait(BATCH_STAGGER_MILLISECONDS,);
  }
  /* oxlint-enable eslint/no-await-in-loop */
  /**
   Each start with the review it read, once its run has ended, in the order the reviews were started.
   */
  const settled = await Promise.all(starts.map(async function settledStart(entry,) {
    return {
      review: entry.review,
      start: await entry.start,
    };
  },),);
  /**
   The status each start ended with, or `none` when it could not be started at all.
   */
  const statuses = settled.map(function statusOf(entry: { readonly start: StartOfOneReview },) {
    return statusTextOf(entry.start,);
  },);
  for (const {
    review,
    start,
  } of settled) {
    /**
     The run that read that review.
     */
    const run = runNameOfAReview({ review, },);
    if (start.status !== 0) {
      lines.push(entryLineOf({
        name: run,
        started: start,
        ...((start.keptAt === undefined) ? {} : { keptAt: start.keptAt, }),
      },),);
    }
    // oxlint-disable-next-line eslint/no-await-in-loop -- one review after another, because a saved file is read from that review's own run; see doc/lint/sequential-awaits.md
    lines.push((await seams.save({ review, },)).line,);
  }
  if (statuses.length > 0) {
    lines.push(`reviews ended with statuses ${statuses.join(' ',)}`,);
  }
  return {
    lines,
    statuses,
  };
}

/**
 Reads the status to print for one start, naming a start that answered none.

 @param start - what starting one review answered

 @returns the status as text, or `none` when the retrying start answered no status

 @example
 ```ts
 const shown = statusTextOf({ status: 0, output: '', },);
 ```
 */
function statusTextOf(start: StartOfOneReview,): string {
  return ((start.status === undefined) ? 'none' : String(start.status,));
}
