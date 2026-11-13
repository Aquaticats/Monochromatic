/**
 Reads the command line of `check-a-review-that-ended`: which reviews to check, where their files and runs live, the rules file a
 restart's brief names first, and whether to start the restarts this check writes.

 @module
 */

import { homedir, } from 'node:os';
import { join, } from 'node:path';
import { optionIn, } from './option-in.ts';
import { requiredText, } from './required-option-values.ts';
import { FOCUS_OF_A_CHANGE_REVIEW, } from './review-brief.ts';
import {
  DEFAULT_REVIEWS_FOLDER_IN_HOME,
  DEFAULT_RUNS_FOLDER_IN_HOME,
} from './reviewer-of-a-change.ts';
import { rulesFileIn, } from './rules-every-pi-brief-carries.ts';

/**
 The command's command line, for the error message.
 */
export const CHECK_REVIEW_USAGE: string =
  'check-a-review-that-ended --review=<name> [--review=<name>]... [--reviews-folder=<folder>] [--runs-folder=<folder>] '
    + '[--rules-file=<file>] [--focus=<text>] [--start]';

/**
 The option that names one review to check, which may be given once per review.
 */
const REVIEW_OPTION = '--review=';

/**
 The option that names the folder that holds one folder per review.
 */
const REVIEWS_FOLDER_OPTION = '--reviews-folder=';

/**
 The option that names the folder the runs live in.
 */
const RUNS_FOLDER_OPTION = '--runs-folder=';

/**
 The option that names the rules file a restart's brief sends the reviewer to first.
 */
const RULES_FILE_OPTION = '--rules-file=';

/**
 The option that names what a restart's reviewer looks hardest at.
 */
const FOCUS_OPTION = '--focus=';

/**
 The word that asks the command to start the restarts it writes, which it never does without.
 */
export const START_RESTARTS_FLAG = '--start';

/**
 A command line the check cannot use.
 */
export class CheckReviewOptionsError extends Error {
  /**
   The name printed with the message.
   */
  override readonly name = 'CheckReviewOptionsError';
}

/**
 Builds the error this command answers a refused option with.

 @param message - what is wrong with the command line

 @returns the error the command exits with

 @example
 ```ts
 throw checkReviewOptionsErrorOf('--review is required.',);
 ```
 */
function checkReviewOptionsErrorOf(message: string,): Error {
  return new CheckReviewOptionsError(message,);
}

/**
 What the check was asked for.
 */
export type CheckReviewOptions = {
  /**
   The reviews to check, in the order given.
   */
  readonly reviews: readonly string[];

  /**
   Folder that holds one folder per review.
   */
  readonly reviewsFolder: string;

  /**
   Folder the runs live in.
   */
  readonly runsFolder: string;

  /**
   The rules file a restart's brief names first.
   */
  readonly rulesFile: string;

  /**
   What a restart's reviewer looks hardest at.
   */
  readonly focus: string;

  /**
   Whether to start the restarts this check writes, which it never does without.
   */
  readonly start: boolean;
};

/**
 Reads the command line.

 @param commandLine - arguments after the program name

 @param checkoutFolder - the top of the checkout, which the rules file's default path is read from

 @returns what the check was asked for

 @throws CheckReviewOptionsError when no review is named, or one is empty

 @example
 ```ts
 const options = parseCheckReviewOptions({ commandLine: ['--review=a', '--start'], checkoutFolder: '/repo' });
 ```
 */
export function parseCheckReviewOptions({
  commandLine,
  checkoutFolder,
}: {
  readonly commandLine: readonly string[];
  readonly checkoutFolder: string;
}): CheckReviewOptions {
  /**
   The reviews to check.
   */
  const reviews = commandLine.filter(function namesAReview(argument,) {
    return argument.startsWith(REVIEW_OPTION,);
  },)
    .map(function reviewOf(argument,) {
      return requiredText({
        value: argument.slice(REVIEW_OPTION.length,),
        name: 'review',
        usage: CHECK_REVIEW_USAGE,
        errorOf: checkReviewOptionsErrorOf,
      },);
    },);
  if (reviews.length === 0) {
    throw checkReviewOptionsErrorOf(`--review is required at least once.\nUsage: ${CHECK_REVIEW_USAGE}`,);
  }
  /**
   The folder that holds one folder per review.
   */
  const reviewsFolder = optionIn({
    commandLine,
    option: REVIEWS_FOLDER_OPTION,
  },);
  /**
   The folder the runs live in.
   */
  const runsFolder = optionIn({
    commandLine,
    option: RUNS_FOLDER_OPTION,
  },);
  /**
   The rules file's path.
   */
  const rulesFile = optionIn({
    commandLine,
    option: RULES_FILE_OPTION,
  },);
  /**
   What a restart's reviewer looks hardest at.
   */
  const focus = optionIn({
    commandLine,
    option: FOCUS_OPTION,
  },);
  return {
    reviews,
    reviewsFolder: (reviewsFolder === '')
      ? join(
        homedir(),
        ...DEFAULT_REVIEWS_FOLDER_IN_HOME,
      )
      : reviewsFolder,
    runsFolder: (runsFolder === '')
      ? join(
        homedir(),
        ...DEFAULT_RUNS_FOLDER_IN_HOME,
      )
      : runsFolder,
    rulesFile: (rulesFile === '') ? rulesFileIn({ checkoutFolder, },) : rulesFile,
    focus: (focus === '') ? FOCUS_OF_A_CHANGE_REVIEW : focus,
    start: commandLine.includes(START_RESTARTS_FLAG,),
  };
}
