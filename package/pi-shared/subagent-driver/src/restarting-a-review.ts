/**
 Planning, writing and starting the restarts a check asks for: whether a review needs one at all, which number its next run
 carries, the refusal of any path that already holds somebody's text, and the one place the command starts anything.

 Nothing here starts a run on its own: `startRestarts` is called only when the caller asked, and a test hands it a fake launcher,
 so no test starts a run, a unit or a Pi.

 @module
 */

import {
  mkdir,
  readdir,
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
import type { ReviewCheck, } from './checking-a-review.ts';
import { errorCodeOf, } from './error-code-of.ts';
import {
  NO_FINISHED_RUN_OF_THAT_NAME,
  newestFinishedRunOf,
} from './newest-finished-run-of-a-name.ts';
import {
  NO_TEXT_AT_THAT_PATH,
  textAtAPath,
} from './review-check-reading.ts';
import {
  type RestartPlan,
  restartPlanOf,
} from './restart-brief.ts';
import {
  FIRST_RESTART_NUMBER,
  REVIEW_RESTART_SEGMENT,
  baseReviewNameOf,
  restartReviewNameOf,
  runWordOf,
} from './review-runs-of-a-change.ts';
import {
  REVIEW_FILE_NAME,
  briefFileOfAReview,
  reviewFileOfAReview,
  runNameOfAReview,
} from './reviewer-of-a-change.ts';
import { retryWordsOfAReview, } from './reviews-of-finished-branches.ts';
import { isDigitsOnly, } from './text-of-an-option.ts';

/**
 The code the operating system gives a path that is not there.
 */
const MISSING_PATH_CODE = 'ENOENT';

/**
 What planning and writing one restart answered, or why nothing was started.
 */
export type RestartOutcome =
  | {
    readonly kind: 'not needed';
  }
    | {
      readonly kind: 'already written';
      readonly line: string;
    }
    | {
      readonly kind: 'cannot';
      readonly line: string;
    }
    | {
      readonly kind: 'refused';
      readonly line: string;
    }
    | {
      readonly kind: 'written';
      readonly plan: RestartPlan;
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
 Says whether a restart has been written and has not ended, so another is not written under it.

 @param restartReview - the restart's review name

 @param reviewsFolder - folder that holds one folder per review

 @param runsFolder - folder the driver writes its runs into

 @returns whether its brief is there and neither its review file nor a finished run is

 @example
 ```ts
 const pending = await aRestartIsPending({ restartReview: 'a-restart-2', reviewsFolder: '/reviews', runsFolder: '/runs' });
 ```
 */
export async function aRestartIsPending({
  restartReview,
  reviewsFolder,
  runsFolder,
}: {
  readonly restartReview: string;
  readonly reviewsFolder: string;
  readonly runsFolder: string;
},): Promise<boolean> {
  /**
   The review file the restart would have written.
   */
  const reviewFile = reviewFileOfAReview({
    reviewsFolder,
    review: restartReview,
  },);
  if ((await textAtAPath(reviewFile,)) !== NO_TEXT_AT_THAT_PATH) {
    return false;
  }
  /**
   The brief the restart was written with.
   */
  const briefFile = briefFileOfAReview({
    reviewsFolder,
    review: restartReview,
  },);
  if ((await textAtAPath(briefFile,)) === NO_TEXT_AT_THAT_PATH) {
    return false;
  }
  /**
   The newest finished run of the restart.
   */
  const finished = await newestFinishedRunOf({
    runsFolder,
    runName: runNameOfAReview({ review: restartReview, },),
  },);
  return finished === NO_FINISHED_RUN_OF_THAT_NAME;
}

/**
 No refusal of a written target.
 */
const NO_REFUSAL: unique symbol = Symbol('written target holds its own text',);

/**
 Reads what one written target's path holds, and refuses it when it holds somebody else's text.

 @param target - one file the plan writes, and its text

 @param restartReview - the restart's review name, for the message

 @returns why the target refuses the plan, or nothing when it does not

 @example
 ```ts
 const refusal = await refusalOfATarget({ target, restartReview: 'a-restart-2' });
 ```
 */
async function refusalOfATarget({
  target,
  restartReview,
}: {
  readonly target: RestartPlan['texts'][number];
  readonly restartReview: string;
},): Promise<string | typeof NO_REFUSAL> {
  /**
   What is at the target's path now.
   */
  const there = await textAtAPath(target.path,);
  if ((there === NO_TEXT_AT_THAT_PATH) || (there === target.text)) {
    return NO_REFUSAL;
  }
  return `${restartReview}: ${target.path} is already there with other text, so it was not replaced; give this restart room by `
    + 'moving that file away';
}

/**
 Writes one restart's plan, refusing any file already holding other text.

 A plan whose files hold exactly its own text is the same restart written twice, which is left alone; a plan whose files hold
 anything else is refused whole, and nothing is written.

 @param plan - the plan to write

 @returns why the plan was refused, or nothing when every file was written or already held its text

 @example
 ```ts
 const written = await writeRestartPlan({ plan });
 ```
 */
export async function writeRestartPlan({ plan, }: { readonly plan: RestartPlan },): Promise<{
  readonly refused?: string
}> {
  /**
   The review file a restart that already ran would have written.
   */
  const reviewFile = join(
    dirname(plan.briefFile,),
    REVIEW_FILE_NAME,
  );
  if ((await textAtAPath(reviewFile,)) !== NO_TEXT_AT_THAT_PATH) {
    return {
      refused: `${plan.restartReview}: ${reviewFile} is already there, so this restart already ran and was not written again`,
    };
  }
  /**
   One refusal per target, in the plan's order.
   */
  const refusals = await Promise.all(plan.texts
    .map(async function refusalAt(target,) {
      return await refusalOfATarget({
        target,
        restartReview: plan.restartReview,
      },);
    },),);
  /**
   The first refusal, which is the one a reader needs.
   */
  const refusal = refusals.find(function isARefusal(one,) {
    return one !== NO_REFUSAL;
  },);
  if (refusal !== undefined) {
    return { refused: refusal, };
  }
  await Promise.all(plan.texts
    .map(async function writeOne(target,) {
    /**
     What is at the target's path now, which is its own text when an earlier write of it was interrupted.
     */
      const there = await textAtAPath(target.path,);
      if (there === target.text) {
        return;
      }
      await mkdir(
        dirname(target.path,),
        { recursive: true, },
      );
      await writeFile(
        target.path,
        target.text,
        'utf8',
      );
    },),);
  return {};
}

/**
 Plans and writes one restart of a review, when its verdict or its coverage asks for one.

 @param check - what the review's check answered

 @param reviewsFolder - folder that holds one folder per review

 @param runsFolder - folder the driver writes its runs into

 @param rulesFile - the rules file the brief names first

 @param focus - what the reviewer looks hardest at

 @returns what was written, or why nothing was

 @example
 ```ts
 const outcome = await restartOfAReview({ check, reviewsFolder: '/reviews', runsFolder: '/runs', rulesFile: '/repo/rules.md', focus: 'data loss' });
 ```
 */
export async function restartOfAReview({
  check,
  reviewsFolder,
  runsFolder,
  rulesFile,
  focus,
}: {
  readonly check: ReviewCheck;
  readonly reviewsFolder: string;
  readonly runsFolder: string;
  readonly rulesFile: string;
  readonly focus: string;
},): Promise<RestartOutcome> {
  /**
   The review the restart answers.
   */
  const review = baseReviewNameOf({ review: check.review, },);
  /**
   The runs of the review already written.
   */
  const numbers = await runsWrittenOf({
    reviewsFolder,
    review,
  },);
  /**
   Whether only the files the earlier review left unread whole are left, which a caller may start a restart for.
   */
  const onlyFilesUnread = (check.verdict === 'complete')
    && ((check.coverage
      ?.filesNotReadWhole
      .length
      ?? 0) > 0);
  if ((check.verdict === 'complete') && (!onlyFilesUnread)) {
    return { kind: 'not needed', };
  }
  /**
   The number the next run of this review carries.
   */
  const number = (numbers.length === 0) ? FIRST_RESTART_NUMBER : Math.max(...numbers,) + 1;
  /**
   The last run of the review already written, when any was.
   */
  const last = (numbers.length === 0)
    ? undefined
    : restartReviewNameOf({
      review,
      number: Math.max(...numbers,),
    },);
  if ((last !== undefined) && (await aRestartIsPending({
    restartReview: last,
    reviewsFolder,
    runsFolder,
  },))) {
    return {
      kind: 'already written',
      line: `${last}: the ${runWordOf(Math.max(...numbers,),)} run of review ${review} is already written at `
        + `${briefFileOfAReview({
          reviewsFolder,
          review: last,
        },)}, and it has not ended, so no later run was written`,
    };
  }
  /**
   The restart, planned from what the earlier run read.
   */
  const planned = await restartPlanOf({
    check,
    number,
    reviewsFolder,
    runsFolder,
    rulesFile,
    focus,
  },);
  if (planned.plan === undefined) {
    return {
      kind: 'cannot',
      line: planned.cannot ?? `${review}: no restart could be planned`,
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
  return {
    kind: 'written',
    plan: planned.plan,
    line: planned.plan
      .line,
  };
}

/**
 What starting the restarts a check asked for needs.
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
 Starts each planned restart through the retrying start, one at a time.

 This is the only place the command starts anything, and the command calls it only when the caller asked with `--start`.

 @param plans - the restarts to start

 @param runsFolder - folder the driver writes its runs into

 @param checkoutFolder - the top of the checkout the worktrees are made in

 @param seams - the answers the command asks, which a test replaces

 @returns one line per start, in the order the plans were given

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
   One start per plan, each resolving when its run has ended.
   */
  const starts: {
    readonly plan: RestartPlan;
    readonly start: Promise<StartedEntry>
  }[] = [];
  /* oxlint-disable eslint/no-await-in-loop -- one restart after another, because the stagger is what keeps their slots in the
     order the reviews were given. See doc/lint/sequential-awaits.md. */
  for (const plan of plans) {
    starts.push({
      plan,
      start: seams.start({
        name: plan.entry
          .name,
        words: retryWordsOfAReview({
          review: plan.restartReview,
          commit: plan.commit,
          briefFile: plan.briefFile,
          runsFolder,
          checkoutFolder,
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
    readonly started: StartedEntry
  }[] = await Promise.all(starts.map(async function answeredStart(entry: {
    readonly plan: RestartPlan;
    readonly start: Promise<StartedEntry>;
  },) {
    return {
      plan: entry.plan,
      started: await entry.start,
    };
  },),);
  return answered.map(function lineFor(entry: {
    readonly plan: RestartPlan;
    readonly started: StartedEntry;
  },) {
    /**
     The line the start ended with.
     */
    const line = entryLineOf({
      name: entry.plan
        .entry
        .name,
      started: entry.started,
    },);
    seams.report?.(line,);
    return line;
  },);
}
