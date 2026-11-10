/**
 A round of Kimi K3.1 Pro reviews: each review's worktree, diff and brief are prepared, the review is started outside the
 Neuralwatt slots, and a review the provider rate-limited is started again on Neuralwatt with the same session.

 The reviews run side by side, started a little apart, which is how a round of them finishes in the time one takes.

 @module
 */

import { caughtValueText, } from '@monochromatic-dev/caught-value-core';
import {
  KIMI_REVIEW_MODEL,
  KIMI_REVIEW_PROVIDER,
  type KimiReview,
} from './kimi-reviews.ts';
import {
  type NO_SUMMARY,
  type PreparedReview,
  type ReviewFallback,
  fallbackAfterARateLimit
} from './prepare-a-kimi-review.ts';
import { runFolderPathOf, } from './pi-runs-folders.ts';

/**
 What one attempt at a review ended with.
 */
export type AttemptEnding = {
  /**
   The status its queue exited with.
   */
  readonly status: number;

  /**
   The last line it printed.
   */
  readonly output: string;
};

/**
 Starts one attempt at a prepared review.
 */
export type StartAReviewAttempt = (request: {
  /**
   The review to start.
   */
  readonly prepared: PreparedReview;

  /**
   What the attempt runs on.
   */
  readonly provider: string;
  readonly model: string;

  /**
   The number of this attempt, which names its run folder.
   */
  readonly run: number;

  /**
   How long this attempt may last.
   */
  readonly wallSeconds: number;

  /**
   The session file this attempt continues, when the attempt before it left one.
   */
  readonly session?: string;
}) => Promise<AttemptEnding>;

/**
 Reads what one attempt's summary says, for the fallback decision.
 */
export type SummaryTextOfARun = (request: {
  readonly runFolder: string;
}) => Promise<string | typeof NO_SUMMARY>;

/**
 Prepares one review.
 */
export type PrepareAReview = (review: KimiReview) => Promise<PreparedReview>;

/**
 What a round of reviews is started with.
 */
export type RoundOfReviewsRequest = {
  /**
   The reviews to start.
   */
  readonly reviews: readonly KimiReview[];

  /**
   How a review is prepared.
   */
  readonly prepare: PrepareAReview;

  /**
   How one attempt is started.
   */
  readonly startAttempt: StartAReviewAttempt;

  /**
   How an attempt's summary is read.
   */
  readonly summaryTextOf: SummaryTextOfARun;

  /**
   How time passes between two reviews, which a test replaces.
   */
  readonly wait: (milliseconds: number) => Promise<void>;

  /**
   How far apart two reviews are started.
   */
  readonly staggerMilliseconds: number;

  /**
   The folder the driver writes one run folder per run into.
   */
  readonly runsFolder: string;

  /**
   How long an attempt on Synthetic may last.
   */
  readonly wallSeconds: number;

  /**
   How long the attempt on Neuralwatt's Kimi pool may last, which is longer because that pool runs slower.
   */
  readonly fallbackWallSeconds: number;
};

/**
 Reads the last line an attempt printed.

 @param output - what the attempt printed

 @returns the last line that holds anything

 @example
 ```ts
 const line = lastLineOf({ output: 'a\nb\n', });
 ```
 */
function lastLineOf({
  output,
}: {
  readonly output: string;
},): string {
  return output.split('\n',)
    .filter(function holdsSomething(line,) {
      return line.trim() !== '';
    },)
    .at(-1,)
    ?.trim()
    ?? '';
}

/**
 Describes what one attempt answered.

 @param prepared - the review the attempt ran

 @param run - the attempt's number

 @param ending - what the attempt ended with

 @returns the line to print

 @example
 ```ts
 const line = attemptLineOf({ prepared, run: 1, ending: { status: 0, output: '', }, });
 ```
 */
function attemptLineOf({
  prepared,
  run,
  ending,
}: {
  readonly prepared: PreparedReview;
  readonly run: number;
  readonly ending: AttemptEnding;
},): string {
  /**
   The last thing the attempt printed, which says what happened when it did not end clean.
   */
  const last = lastLineOf({ output: ending.output, },);
  return `${prepared.runName}-${String(run,)}: status ${String(ending.status,)}${((ending.status === 0) || (last === '')) ? '' : ` ${last}`}`;
}

/**
 What preparing one review gave: the review, or what went wrong.
 */
type PreparedOrProblem = {
  /**
   The review as prepared, when it could be.
   */
  readonly prepared?: PreparedReview;

  /**
   What went wrong, when it did.
   */
  readonly problem?: string;
};

/**
 Prepares one review, saying what went wrong rather than throwing it.

 @param review - the review to prepare

 @param request - how a review is prepared

 @returns the review, or what went wrong

 @example
 ```ts
 const made = await preparedOrProblemOf({ review, request });
 ```
 */
async function preparedOrProblemOf({
  review,
  request,
}: {
  readonly review: KimiReview;
  readonly request: RoundOfReviewsRequest;
},): Promise<PreparedOrProblem> {
  try {
    return { prepared: await request.prepare(review,), };
  }
  catch (error) {
    return { problem: `kimi-review-${review.name}: not started, the brief or worktree could not be made (${caughtValueText(error,)})`, };
  }
}

/**
 Starts one review, and starts it again on Neuralwatt when the provider rate-limited it.

 @param review - the review to start

 @param request - how a review is prepared and started, and with what

 @returns the lines about this review

 @example
 ```ts
 const lines = await runOneReview({ review, request });
 ```
 */
async function runOneReview({
  review,
  request,
}: {
  readonly review: KimiReview;
  readonly request: RoundOfReviewsRequest;
},): Promise<readonly string[]> {
  /**
   The review as prepared, or why it could not be.
   */
  const made = await preparedOrProblemOf({
    review,
    request,
  },);
  if (made.prepared === undefined) {
    return [(made.problem ?? `kimi-review-${review.name}: not started`),];
  }
  /**
   The prepared review, which everything after this starts.
   */
  const {prepared} = made;
  /**
   What the attempt on Synthetic ended with.
   */
  const first = await request.startAttempt({
    prepared,
    provider: KIMI_REVIEW_PROVIDER,
    model: KIMI_REVIEW_MODEL,
    run: 1,
    wallSeconds: request.wallSeconds,
  },);
  /**
   What that attempt left for the next one.
   */
  const next: ReviewFallback = fallbackAfterARateLimit({
    summaryText: await request.summaryTextOf({
      runFolder: runFolderPathOf({
        runsFolder: request.runsFolder,
        name: prepared.runName,
        run: 1,
      },),
    },),
  },);
  if (!next.resume) {
    return [
      `${prepared.runName}: ${String(prepared.filesRead,)} files, worktree ready at ${review.commit}`,
      attemptLineOf({
        prepared,
        run: 1,
        ending: first,
      },),
    ];
  }
  /**
   What the attempt on Neuralwatt's Kimi pool ended with.
   */
  const second = await request.startAttempt({
    prepared,
    provider: next.provider,
    model: next.model,
    run: 2,
    wallSeconds: request.fallbackWallSeconds,
    session: next.session,
  },);
  return [
    `${prepared.runName}: ${String(prepared.filesRead,)} files, worktree ready at ${review.commit}`,
    attemptLineOf({
      prepared,
      run: 1,
      ending: first,
    },),
    `${prepared.runName}: Synthetic rate-limited the review, so its session went on to ${next.provider}/${next.model}`,
    attemptLineOf({
      prepared,
      run: 2,
      ending: second,
    },),
  ];
}

/**
 Starts a round of reviews, a little apart, and answers with what became of each.

 @param request - the reviews, how one is prepared and started, and how far apart they start

 @returns one block of lines per review, in the list's order

 @example
 ```ts
 const lines = await startARoundOfKimiReviews({ reviews, prepare, startAttempt, summaryTextOf, wait, staggerMilliseconds: 0, runsFolder: '/runs', wallSeconds: 5_400, fallbackWallSeconds: 10_800, });
 ```
 */
export async function startARoundOfKimiReviews(
  request: RoundOfReviewsRequest,
): Promise<readonly string[]> {
  /**
   The round as the caller asked for it.
   */
  const {
    reviews,
    prepare,
    startAttempt,
    summaryTextOf,
    wait,
    staggerMilliseconds,
    runsFolder,
    wallSeconds,
    fallbackWallSeconds,
  } = request;
  /**
   One review per entry, each going while the ones after it are started.
   */
  const round: Promise<readonly string[]>[] = [];
  for (const review of reviews) {
    round.push(runOneReview({
      review,
      request: {
        reviews,
        prepare,
        startAttempt,
        summaryTextOf,
        wait,
        staggerMilliseconds,
        runsFolder,
        wallSeconds,
        fallbackWallSeconds,
      },
    },),);
    // oxlint-disable-next-line eslint/no-await-in-loop -- the reviews are started a little apart so a round's units do not all begin at once; see doc/lint/sequential-awaits.md
    await wait(staggerMilliseconds,);
  }
  return (await Promise.all(round,)).flat();
}
