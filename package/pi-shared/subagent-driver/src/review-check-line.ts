/**
 Reads the one line that states a review's verdict and its reasons.

 @module
 */

import type { ReviewCheck, } from './review-check-types.ts';
import { runNameOfAReview, } from './reviewer-of-a-change.ts';

/**
 Reads the line one check prints: the run's name, its verdict, and its reasons.

 @param check - one review's check

 @returns the line

 @example
 ```ts
 const line = checkLineOf({ check });
 ```
 */
export function checkLineOf({ check, }: { readonly check: ReviewCheck },): string {
  /**
   The reasons, with the separator the line reads them with.
   */
  const reasons = (check.reasons
    .length
    === 0) ? '' : `; ${check.reasons
      .join('; ')}`;
  return `${runNameOfAReview({ review: check.review, },)}: ${check.verdict}${reasons}`;
}
