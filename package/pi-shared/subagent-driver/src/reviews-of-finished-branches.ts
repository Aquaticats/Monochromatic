/**
 Starts one review per finished branch: each review's brief is written first, then its run is started through the retrying start
 on the qwen plan's review pool, and a review that ended without its review file has its last message saved as that file.

 A branch the integration already merged, a branch whose change leaves nothing to read under the reviewed source folder, and a
 change already reviewed are each said in one line and the rest go on: the scratch starter stopped at the first of them, and two
 reviews of round 231 were lost that way with no line anyone read.

 A review whose file is already written is judged like one this call finished, by the same answer the command that saves a review
 acts on: one that covers its change is said to be already reviewed, and one that does not covers nothing. A review that does not
 cover its change, a file its reviewer was killed before finishing included, is named in its own line, counted in the status, and
 not started again: the caller starts it, because a review's branch is fixed by its name and the launcher adds an existing branch
 as it is.

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
import type { SavedReview, } from './last-message-of-a-review.ts';
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
 The longest a review of a finished branch has used, in turns, read on 2026-11-11 from the `turns` of every review run's
 `summary.json` under the coordinator's runs folder: 226 review runs held one, 172 of them ended on their own terms, and those
 used 3 to 156 turns with a median of 51 and 147, 153 and 156 at the top. The 28 reviews measured earlier that day ended at 16 to
 37 turns, which is why a cap of 47 cut two reviews off mid-reading: they were not the long ones.
 */
export const LONGEST_MEASURED_REVIEW_TURNS = 156;

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
 What saving one review answered, which is the part of `SavedReview` this module reads.
 */
export type SavedReviewAnswer = Pick<SavedReview, 'line' | 'covers'>;

/**
 What starting the reviews answered: the lines every step printed, the status each start ended with, and how many reviews do not
 cover the change they were given.
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

  /**
   How many of the reviews it judged do not cover the change they were given, whether this call started them or found them
   already written, which is what the command that starts them answers `1` for as well as a start that did not end `0`.
   */
  readonly reviewsWithoutCoverage: number;
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
   Whether a review of that change already wrote its review file, which is then judged rather than started.
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
   Shows one line as it happens, so a review that starts, fails or ends is read within minutes rather than after the last one: on
   2026-11-11 four reviews were lost while the command printed nothing until every review had ended.
   */
  readonly report?: (line: string) => void;

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
   Saves a review that ended without its review file, answering the line to print and whether the review covers its change.
   */
  readonly save: (request: { readonly review: string }) => Promise<SavedReviewAnswer>;

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
   Records one line, and shows it as it happens when the caller asked to see it.

   @param line - the line to record
   */
  function note(line: string,): void {
    lines.push(line,);
    seams.report?.(line,);
  }
  /**
   One start per review, each resolving when its run has ended, kept beside the review it reads.
   */
  const starts: {
    readonly review: string;
    readonly start: Promise<StartOfOneReview>
  }[] = [];
  /**
   Whether each review this call judges covers the change it was given: one for every review already written, and one for every
   start it made.
   */
  const coverage: boolean[] = [];
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
      // A review that is already written is judged like one this call finished: a file that does not cover its change is not a
      // review of it, so the caller is told which review and why, and the starter's status counts it. It is not started again
      // here: a review's branch is fixed by its name and the launcher adds an existing branch as it is, so starting over, after
      // the placeholder or the folder is put aside, is the caller's decision. See README.md.
      /**
       What saving that review answered: its line, and whether it covers the change it was given.
       */
      // oxlint-disable-next-line eslint/no-await-in-loop -- one branch after another, and this review's own file is read before the next branch is looked at; see doc/lint/sequential-awaits.md
      const alreadyWritten = await seams.save({ review, },);
      coverage.push(alreadyWritten.covers,);
      note(alreadyWritten.covers
        ? `${run}: already reviewed`
        : alreadyWritten.line,);
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
    note(briefed.line,);
    if (!briefed.written) {
      continue;
    }
    note(`${runNameOfAReview({ review, },)}: started`,);
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
      note(entryLineOf({
        name: run,
        started: start,
        ...((start.keptAt === undefined) ? {} : { keptAt: start.keptAt, }),
      },),);
    }
    /**
     What saving that review answered: its line, and whether the review covers the change it was given.
     */
    // oxlint-disable-next-line eslint/no-await-in-loop -- one review after another, because a saved file is read from that review's own run; see doc/lint/sequential-awaits.md
    const saved = await seams.save({ review, },);
    note(saved.line,);
    coverage.push(saved.covers,);
  }
  /**
   How many of the reviews started do not cover the change they were given.
   */
  const reviewsWithoutCoverage = coverage.filter(function doesNotCover(covers,) {
    return !covers;
  },)
    .length;
  if ((statuses.length > 0) || (reviewsWithoutCoverage > 0)) {
    note(statusSummaryOf({
      statuses,
      reviewsWithoutCoverage,
    },),);
  }
  return {
    lines,
    statuses,
    reviewsWithoutCoverage,
  };
}

/**
 Writes the last line a starter prints, so that a caller reads which starts ended how and whether every review covers the change it
 was given.

 A call that started nothing still prints it when a review it judged does not cover its change, because that answer is the one its
 caller acts on.

 @param statuses - the status each start ended with, in the order the starts were made

 @param reviewsWithoutCoverage - how many of the reviews it judged do not cover the change they were given

 @returns the line

 @example
 ```ts
 const line = statusSummaryOf({ statuses: ['0',], reviewsWithoutCoverage: 0, },);
 ```
 */
function statusSummaryOf({
  statuses,
  reviewsWithoutCoverage,
}: {
  readonly statuses: readonly string[];
  readonly reviewsWithoutCoverage: number;
}): string {
  /**
   What the line says about coverage.
   */
  const coverageWords = coverageWordsOf({ reviewsWithoutCoverage, },);
  return (statuses.length === 0)
    ? `no review was started${coverageWords}`
    : `reviews ended with statuses ${statuses.join(' ',)}${coverageWords}`;
}

/**
 Writes the words that follow the statuses a starter prints, so that the last line a caller reads says whether every review covers
 the change it was given.

 @param reviewsWithoutCoverage - how many of the reviews started do not cover the change they were given

 @returns the words, beginning with the comma that joins them to the statuses

 @example
 ```ts
 const words = coverageWordsOf({ reviewsWithoutCoverage: 1, },);
 ```
 */
function coverageWordsOf({ reviewsWithoutCoverage }: { readonly reviewsWithoutCoverage: number },): string {
  if (reviewsWithoutCoverage === 0) {
    return ', and every review covers its change';
  }
  return `, and ${String(reviewsWithoutCoverage,)} `
    + `${(reviewsWithoutCoverage === 1) ? 'does' : 'do'} not cover ${(reviewsWithoutCoverage === 1) ? 'its' : 'their'} change`;
}

/**
 Reads the status a command that started reviews answers with: `0` only when every start ended `0` and every review covers the
 change it was given.

 @param started - what starting the reviews answered

 @returns the status the command exits with

 @example
 ```ts
 const status = statusOfStartedReviews({ started, },);
 ```
 */
export function statusOfStartedReviews({ started, }: { readonly started: StartedReviews },): number {
  /**
   Whether every start ended `0` and every review covers its change.
   */
  const everyStartEndedWell = started.statuses
    .every(function endedWell(status,) {
      return status === '0';
    },);
  return (everyStartEndedWell && (started.reviewsWithoutCoverage === 0)) ? 0 : 1;
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
