/**
 Planning, writing and starting the second review a thin one is owed: whether a review needs one at all, which number its next run
 carries, the refusal of any path that already holds somebody's text, and the one place a start happens.

 A review is answered once: the first review of a change gets one second review, on the other model of the pair, and a second review
 that is still thin is reported and left for whoever coordinates, never started a third time. A plan written and not started is
 started by a later check, and a plan a kill cut short is repaired rather than left; a start already taken by another check is not
 taken again.

 Nothing here starts a run by itself: the command starts what this module plans, through `startRestarts`, which takes the start
 marker first so two checks cannot both start one restart.

 @module
 */

import {
  mkdir,
  readdir,
  rm,
  writeFile,
} from 'node:fs/promises';
import {
  dirname,
  join,
} from 'node:path';
import {
  BATCH_STAGGER_MILLISECONDS,
  type StartedEntry,
  entryLineOf,
} from './batch-list.ts';
import { checkLineOf, } from './checking-a-review.ts';
import type { ReviewCheck, } from './review-check-types.ts';
import { errorCodeOf, } from './error-code-of.ts';
import {
  NO_FINISHED_RUN_OF_THAT_NAME,
  attemptsOfARun,
  newestFinishedRunOf,
} from './newest-finished-run-of-a-name.ts';
import { textAtAPath, } from './review-check-reading.ts';
import {
  type RestartPlan,
  restartPlanOf,
} from './restart-brief.ts';
import { writeRestartPlan, } from './writing-a-restart-plan.ts';
import {
  FIRST_RESTART_NUMBER,
  NOT_A_RESTART,
  REVIEW_RESTART_SEGMENT,
  baseReviewNameOf,
  restartNumberIn,
  restartReviewNameOf,
  runWordOf,
} from './review-runs-of-a-change.ts';
import {
  type ReviewerOfAChange,
  briefFileOfAReview,
  reviewFileOfAReview,
  runNameOfAReview,
} from './reviewer-of-a-change.ts';
import { retryWordsOfAReview, } from './reviews-of-finished-branches.ts';
import {
  type StartOwnership,
  takeTheStart,
} from './start-mark-of-a-restart.ts';
import { isDigitsOnly, } from './text-of-an-option.ts';

/**
 The code the operating system gives a path that is not there.
 */
const MISSING_PATH_CODE = 'ENOENT';

/**
 What planning and writing one second review answered, or why nothing was started.
 */
export type RestartOutcome =
  | {
    readonly kind: 'not needed';
  }
    | {
      readonly kind: 'left thin';
      readonly line: string;
    }
    | {
      readonly kind: 'already written';
      readonly line: string;
    }
    | {
      readonly kind: 'owed';
      readonly plan: RestartPlan;
      readonly line: string;
    }
    | {
      readonly kind: 'written';
      readonly plan: RestartPlan;
      readonly line: string;
    }
    | {
      readonly kind: 'cannot';
      readonly line: string;
    }
    | {
      readonly kind: 'refused';
      readonly line: string;
    };

/**
 Reads a folder's entries, or nothing when the folder is not there.

 @param folder - the folder to list

 @returns its entries' names, or none for a folder that is not there

 @example
 ```ts
 const entries = await entriesOrNothingIn('/reviews');
 ```
 */
async function entriesOrNothingIn(folder: string,): Promise<readonly string[]> {
  try {
    return await readdir(folder,);
  }
  catch (error) {
    // A folder that is not there holds no restart yet, which is a fact rather than a failure.
    if (errorCodeOf(error,) === MISSING_PATH_CODE) {
      return [];
    }
    throw error;
  }
}

/**
 Reads the numbers of the runs of one review already written.

 @param reviewsFolder - folder that holds one folder per review

 @param review - the review's own name

 @returns each run's number, as its folder names it

 @example
 ```ts
 const numbers = await runsWrittenOf({ reviewsFolder: '/reviews', review: 'a' });
 ```
 */
async function runsWrittenOf({
  reviewsFolder,
  review,
}: {
  readonly reviewsFolder: string;
  readonly review: string;
},): Promise<readonly number[]> {
  /**
   The words every restart of this review's name starts with.
   */
  const prefix = `${review}${REVIEW_RESTART_SEGMENT}`;
  return (await entriesOrNothingIn(reviewsFolder,)).flatMap(function numberOf(entry,) {
    if (!entry.startsWith(prefix,)) {
      return [];
    }
    /**
     The text after that prefix, which is the restart's number when the entry is one.
     */
    const written = entry.slice(prefix.length,);
    return isDigitsOnly(written,) ? [Number(written,),] : [];
  },);
}

/**
 Plans and writes the second review a review is owed, when its verdict or its coverage asks for one.

 @param check - what the review's check answered

 @param reviewsFolder - folder that holds one folder per review

 @param runsFolder - folder the driver writes its runs into

 @param rulesFile - the rules file the brief names first

 @param focus - what the second reviewer looks hardest at

 @param reviewer - the other of the pair of reviewers a change's review runs between

 @returns what was written, or why nothing was

 @example
 ```ts
 const outcome = await restartOfAReview({ check, reviewsFolder: '/reviews', runsFolder: '/runs', rulesFile: '/repo/rules.md', focus: 'data loss', reviewer: REVIEWING_PEER });
 ```
 */
export async function restartOfAReview({
  check,
  reviewsFolder,
  runsFolder,
  rulesFile,
  focus,
  reviewer,
}: {
  readonly check: ReviewCheck;
  readonly reviewsFolder: string;
  readonly runsFolder: string;
  readonly rulesFile: string;
  readonly focus: string;
  readonly reviewer: ReviewerOfAChange;
},): Promise<RestartOutcome> {
  /**
   Whether this review needs another run at all: a complete review that read every file whole and ran a build program does not.
   */
  if ((check.verdict === 'complete') && (!check.thin)) {
    return { kind: 'not needed', };
  }
  /**
   Which run of the review this check is.
   */
  const asARun = restartNumberIn({ review: check.review, },);
  if (asARun !== NOT_A_RESTART) {
    // Once only: a second review that is still thin is reported and left, never started a third time.
    return {
      kind: 'left thin',
      line: `${check.review}: the ${runWordOf(asARun,)} run of ${baseReviewNameOf({
        review: check.review,
      },)} is still thin or incomplete, and a second review is not started a third time; left for whoever coordinates: ${
         checkLineOf({ check, },)}`,
    };
  }
  /**
   The review this check reads.
   */
  const {review} = check;
  /**
   The second reviews already written for it.
   */
  const numbers = await runsWrittenOf({
    reviewsFolder,
    review,
  },);
  /**
   The number the next run carries: the last one already written, so a plan written and not started is started rather than
   duplicated; or the first restart's number.
   */
  const number = (numbers.length === 0) ? FIRST_RESTART_NUMBER : Math.max(...numbers,);
  if (numbers.length > 0) {
    /**
     The last second review written.
     */
    const last = restartReviewNameOf({
      review,
      number,
    },);
    /**
     Its review file, when it wrote one.
     */
    const lastReviewFile = reviewFileOfAReview({
      reviewsFolder,
      review: last,
    },);
    if ((await textAtAPath({ path: lastReviewFile, },)).kind !== 'no file') {
      return {
        kind: 'already written',
        line: `${last} holds the second review; check it with --review=${last}, and no review is started a third time`,
      };
    }
    /**
     The runs of that second review, finished or not.
     */
    const attempts = await attemptsOfARun({
      runsFolder,
      runName: runNameOfAReview({ review: last, },),
    },);
    if (attempts.length > 0) {
      return {
        kind: 'already written',
        line: `${last}: the second review has been started (${attempts[0]
          ?.folder
          ?? last}) and wrote no review file yet; check it `
          + `with --review=${last}, and no review is started a third time`,
      };
    }
  }
  /**
   The second review, planned from what the first read.
   */
  const planned = await restartPlanOf({
    check,
    number,
    reviewsFolder,
    runsFolder,
    rulesFile,
    focus,
    reviewer,
  },);
  if (planned.plan === undefined) {
    return {
      kind: 'cannot',
      line: planned.cannot ?? `${review}: no second review could be planned`,
    };
  }
  /**
   What writing the plan answered.
   */
  const written = await writeRestartPlan({ plan: planned.plan, },);
  if (written.refused !== undefined) {
    return {
      kind: 'refused',
      line: written.refused,
    };
  }
  return (planned.plan
    .entry
    === undefined)
    ? {
      kind: 'owed',
      plan: planned.plan,
      line: planned.plan
        .line,
    }
    : {
      kind: 'written',
      plan: planned.plan,
      line: planned.plan
        .line,
    };
}

/**
 What starting the second reviews a check asked for needs.
 */
export type RestartStartingSeams = {
  /**
   Starts one entry through the retrying start, which a test replaces with a fake launcher.
   */
  readonly start: (request: {
    readonly name: string;
    readonly words: readonly string[];
  }) => Promise<StartedEntry>;

  /**
   How the command passes time between one start and the next.
   */
  readonly wait: (milliseconds: number) => Promise<void>;

  /**
   Shows one line as it happens.
   */
  readonly report?: (line: string) => void;
};

/**
 Starts each planned second review through the retrying start, one at a time, after taking its start marker.

 This is the only place a review is started, and the command calls it for every plan it wrote whose reviewer Pi can serve. A test
 hands it a fake launcher, so no test starts a run, a unit or a Pi.

 @param plans - the second reviews to start

 @param runsFolder - folder the driver writes its runs into

 @param checkoutFolder - the top of the checkout the worktrees are made in

 @param seams - the answers the command asks, which a test replaces

 @returns one line per plan: what its start answered, or who had already taken it

 @example
 ```ts
 const lines = await startRestarts({ plans, runsFolder: '/runs', checkoutFolder: '/repo', seams });
 ```
 */
export async function startRestarts({
  plans,
  runsFolder,
  checkoutFolder,
  seams,
}: {
  readonly plans: readonly RestartPlan[];
  readonly runsFolder: string;
  readonly checkoutFolder: string;
  readonly seams: RestartStartingSeams;
},): Promise<readonly string[]> {
  /**
   One start per plan, each resolving when its run has ended, or the line that says another check had it.
   */
  const starts: {
    readonly plan: RestartPlan;
    readonly start?: Promise<StartedEntry>;
    readonly line?: string
  }[] = [];
  /* oxlint-disable eslint/no-await-in-loop -- one second review after another, because the stagger is what keeps their slots in
     the order the reviews were given, and each start marker is taken before the next is planned. See
     doc/lint/sequential-awaits.md. */
  for (const plan of plans) {
    /**
     Whether this process took the start, or the line that says who has it.
     */
    const ownership: StartOwnership = await takeTheStart({
      reviewsFolder: plan.reviewsFolder,
      review: plan.review,
      number: plan.number,
    },);
    if (ownership.kind === 'somebody else') {
      starts.push({
        plan,
        line: `${plan.restartReview}: not started, because ${ownership.line}`,
      },);
      continue;
    }
    /**
     The entry the plan starts, which a plan with no entry never reaches here.
     */
    const {entry} = plan;
    starts.push({
      plan,
      start: seams.start({
        name: entry?.name ?? runNameOfAReview({ review: plan.restartReview, },),
        words: retryWordsOfAReview({
          review: plan.restartReview,
          commit: plan.commit,
          briefFile: plan.briefFile,
          runsFolder,
          checkoutFolder,
          ...((entry === undefined) ? {} : {
            provider: entry.provider ?? '',
            model: entry.model ?? '',
          }),
        },),
      },),
    },);
    await seams.wait(BATCH_STAGGER_MILLISECONDS,);
  }
  /* oxlint-enable eslint/no-await-in-loop */
  /**
   Each plan with what its start answered, in the plans' order.
   */
  const answered: {
    readonly plan: RestartPlan;
    readonly started?: StartedEntry;
    readonly line?: string
  }[] = await Promise.all(starts.map(async function answeredStart(entry: {
    readonly plan: RestartPlan;
    readonly start?: Promise<StartedEntry>;
    readonly line?: string;
  },) {
    if (entry.start === undefined) {
      return {
        plan: entry.plan,
        ...((entry.line === undefined) ? {} : { line: entry.line, }),
      };
    }
    return {
      plan: entry.plan,
      started: await entry.start,
    };
  },),);
  return answered.map(function lineFor(entry: {
    readonly plan: RestartPlan;
    readonly started?: StartedEntry;
    readonly line?: string;
  },) {
    /**
     The line the start ended with, or the line that says why it did not start.
     */
    const line = (entry.started === undefined)
      ? (entry.line ?? `${entry.plan
        .restartReview}: not started`)
      : entryLineOf({
        name: entry.plan
          .entry
          ?.name
          ?? runNameOfAReview({ review: entry.plan
            .restartReview, },),
        started: entry.started,
      },);
    seams.report?.(line,);
    return line;
  },);
}
