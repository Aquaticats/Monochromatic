/**
 The names one review's runs carry: which run of a review a folder is, the review a run answers, and the words a run is named by
 in prose.

 A restart is the review's own name with `-restart-<number>` on the end, so a restart never shares a run, a worktree or a branch
 with the run it answers, and a review started again from a restart's own name reads back to the one change it is about.

 @module
 */

import { isDigitsOnly, } from './text-of-an-option.ts';

/**
 The text a restart's review name gains before its number.
 */
export const REVIEW_RESTART_SEGMENT = '-restart-';

/**
 The number the first run of a review carries, which is one because the review itself is that run.
 */
export const FIRST_RUN_NUMBER: number = 1;

/**
 The number a review's first restart carries, which is two because the review itself is the first run.
 */
export const FIRST_RESTART_NUMBER: number = FIRST_RUN_NUMBER + 1;

/**
 The words the runs of a review are named by, from the first run on.
 */
const RUN_WORDS: readonly string[] = [
  'first',
  'second',
  'third',
  'fourth',
  'fifth',
];

/**
 A review name that carries no restart number.
 */
const NOT_A_RESTART: unique symbol = Symbol('review name holds no restart number',);

/**
 Reads the number a review's own name ends with, when it is a restart's number.

 @param review - the review's name, as a folder names it

 @returns the restart's number, or {@link NOT_A_RESTART} when the name is no restart's

 @example
 ```ts
 const number = restartNumberIn({ review: 'a-restart-3' });
 ```
 */
export function restartNumberIn({
  review,
}: {
  readonly review: string;
},): number | typeof NOT_A_RESTART {
  /**
   Where the name's last restart segment starts.
   */
  const at = review.lastIndexOf(REVIEW_RESTART_SEGMENT,);
  if (at === (-1)) {
    return NOT_A_RESTART;
  }
  /**
   The text after that segment, which is the number when the name is a restart's.
   */
  const written = review.slice(at + REVIEW_RESTART_SEGMENT.length,);
  return isDigitsOnly(written,) ? Number(written,) : NOT_A_RESTART;
}

/**
 Reads the review a restart answers: the name with every restart's own suffix taken off.

 @param review - the review's or restart's name

 @returns the review's own name

 @example
 ```ts
 const base = baseReviewNameOf({ review: 'a-restart-2-restart-3' });
 ```
 */
export function baseReviewNameOf({ review, }: { readonly review: string },): string {
  /**
   Where the name's last restart segment starts, or minus one when it carries none.
   */
  const at = review.lastIndexOf(REVIEW_RESTART_SEGMENT,);
  if (at === (-1)) {
    return review;
  }
  if (restartNumberIn({ review, },) === NOT_A_RESTART) {
    return review;
  }
  return baseReviewNameOf({ review: review.slice(
    0,
    at,
  ), },);
}

/**
 Reads the name of one run of a review.

 @param review - the review's own name

 @param number - which run of it, counting the review itself as the first

 @returns the run's review name

 @example
 ```ts
 const name = restartReviewNameOf({ review: 'a', number: 2 });
 ```
 */
export function restartReviewNameOf({
  review,
  number,
}: {
  readonly review: string;
  readonly number: number;
},): string {
  return `${baseReviewNameOf({ review, },)}${REVIEW_RESTART_SEGMENT}${String(number,)}`;
}

/**
 Reads the words a run of a review is named by.

 @param number - which run of it, counting the review itself as the first

 @returns the words, such as `third`, or the run's number when no word is written for it

 @example
 ```ts
 const word = runWordOf(3);
 ```
 */
export function runWordOf(number: number,): string {
  return RUN_WORDS[number - FIRST_RUN_NUMBER] ?? `run ${String(number,)}`;
}
