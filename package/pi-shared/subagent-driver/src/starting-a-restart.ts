/**
 Starting the second reviews a check wrote: the start marker that lets one check start one review, the seam a test replaces with a
 fake launcher, and the plan that names why a diff cannot be read instead of raising it.

 @module
 */

import { rm, } from 'node:fs/promises';
import {
  BATCH_STAGGER_MILLISECONDS,
  type StartedEntry,
  entryLineOf,
  startedOrFailed,
} from './batch-list.ts';
import type { ReviewCheck, } from './review-check-types.ts';
import {
  type RestartPlan,
  restartPlanOf,
} from './restart-brief.ts';
import { ReviewDiffError, } from './review-diff.ts';
import {
  type ReviewerOfAChange,
  runNameOfAReview,
} from './reviewer-of-a-change.ts';
import { retryWordsOfAReview, } from './reviews-of-finished-branches.ts';
import {
  type StartOwnership,
  takeTheStart,
} from './start-mark-of-a-restart.ts';

/**
 Plans one second review, and answers the line that says why its diff cannot be read rather than raising it.

 @param request - what the plan needs, as `restartPlanOf` reads it

 @returns the planned review, or why no plan could be read

 @example
 ```ts
 const planned = await restartPlanOrCannot({ check, number: 2, reviewsFolder: '/reviews', runsFolder: '/runs', rulesFile: '/repo/rules.md', focus: 'data loss', reviewer });
 ```
 */
export async function restartPlanOrCannot(request: {
  readonly check: ReviewCheck;
  readonly number: number;
  readonly reviewsFolder: string;
  readonly runsFolder: string;
  readonly rulesFile: string;
  readonly focus: string;
  readonly reviewer: ReviewerOfAChange;
},): Promise<{
  readonly kind: 'planned';
  readonly planned: Awaited<ReturnType<typeof restartPlanOf>>
} | {
  readonly kind: 'cannot';
  readonly line: string
}> {
  try {
    return {
      kind: 'planned',
      planned: await restartPlanOf(request,),
    };
  }
  catch (error) {
    // A diff this reader cannot read is one review's own file; it is named, and the next review is still checked.
    if (error instanceof ReviewDiffError) {
      return {
        kind: 'cannot',
        line: `${request.check
          .review}: no second review was planned, because ${error.message}`,
      };
    }
    throw error;
  }
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
    readonly marker?: string;
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
      marker: ownership.file,
      start: startedOrFailed({
        start: seams.start,
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
    readonly marker?: string;
    readonly start?: Promise<StartedEntry>;
    readonly line?: string;
  },) {
    if (entry.start === undefined) {
      return {
        plan: entry.plan,
        ...((entry.line === undefined) ? {} : { line: entry.line, }),
      };
    }
    /**
     What the start answered.
     */
    const started = await entry.start;
    // A start that rejected answered no status at all, so it launched nothing: the marker this process took is given back, and a
    // later check starts the second review once.
    if ((started.status === undefined) && (entry.marker !== undefined)) {
      await rm(
        entry.marker,
        { force: true, },
      );
    }
    return {
      plan: entry.plan,
      started,
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
