/**
 The batch entry one review of a range is started as: the run named after the change, on the branch and from the brief the
 package's writer made, on the provider and model every review runs on.

 It is read beside the start of an entry of a queue because it is the one shape of entry whose run name, branch and props are
 derived rather than queued whole.

 @module
 */

import {
  BATCH_DEFAULT_CPU_PERCENT,
  BATCH_DEFAULT_DEADLINE_SECONDS,
  BATCH_DEFAULT_MEMORY,
  BATCH_DEFAULT_TASK_COUNT,
  type BatchEntry,
} from './batch-list.ts';
import {
  type ReviewOfARangeEntry,
  changeOfAReviewRun,
} from './entry-of-a-queue.ts';
import { REVIEW_WALL_SECONDS, } from './reviews-of-finished-branches.ts';
import {
  REVIEWING_MODEL,
  REVIEWING_PROVIDER,
  branchOfAReview,
  briefFileOfAReview,
  runNameOfAReview,
} from './reviewer-of-a-change.ts';

/**
 Reads the batch entry one review of a range is started as: the run named after the change, on the branch and from the brief the
 package's writer made, on the provider and model every review runs on.

 @param review - the review of a range

 @param reviewsFolder - folder that holds one folder per review

 @returns the batch entry to write

 @example
 ```ts
 const entry = reviewRunEntryOf({ review: { name: 'review-a', reviewOfRange: 'main...a', commit: 'a', source: 'src' }, reviewsFolder: '/reviews' });
 ```
 */
export function reviewRunEntryOf({
  review,
  reviewsFolder,
}: {
  readonly review: ReviewOfARangeEntry;
  readonly reviewsFolder: string;
},): BatchEntry {
  /**
   The change under review.
   */
  const change = changeOfAReviewRun({ run: review.name, },);
  return {
    name: runNameOfAReview({ review: change, },),
    branch: branchOfAReview({ review: change, },),
    base: review.commit,
    prompt: briefFileOfAReview({
      reviewsFolder,
      review: change,
    },),
    cpu: BATCH_DEFAULT_CPU_PERCENT,
    memory: BATCH_DEFAULT_MEMORY,
    tasks: BATCH_DEFAULT_TASK_COUNT,
    wall: REVIEW_WALL_SECONDS,
    provider: REVIEWING_PROVIDER,
    model: REVIEWING_MODEL,
  };
}
