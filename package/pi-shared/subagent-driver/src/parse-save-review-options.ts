/**
 Reads the command line of `save-the-final-message-of-a-review-that-wrote-no-review-file`: which reviews to save, and the folders
 they are read from and written to.

 @module
 */

import { homedir, } from 'node:os';
import { join, } from 'node:path';
import { optionIn, } from './option-in.ts';
import { requiredText, } from './required-option-values.ts';

/**
 The command's command line, for the error message.
 */
export const SAVE_REVIEW_USAGE =
  'save-the-final-message-of-a-review-that-wrote-no-review-file --review=<name> [--review=<name>]... [--runs-folder=<folder>] [--review-folder=<folder>]';

/**
 The option that names one review.
 */
const REVIEW_OPTION = '--review=';

/**
 The option that names the folder the runs live in.
 */
const RUNS_FOLDER_OPTION = '--runs-folder=';

/**
 The option that names the folder that holds one folder per review.
 */
const REVIEW_FOLDER_OPTION = '--review-folder=';

/**
 The parts of the coordinator's runs folder below the home directory of whoever runs this.
 */
const DEFAULT_RUNS_FOLDER_IN_HOME: readonly string[] = [
  'temp',
  'agent',
  'pi-runs',
];

/**
 The parts of the coordinator's reviews folder below the home directory of whoever runs this.
 */
const DEFAULT_REVIEW_FOLDER_IN_HOME: readonly string[] = [
  'temp',
  'agent',
  'kimi-reviews',
];

/**
 A command line the command cannot use.
 */
export class SaveReviewOptionsError extends Error {
  /**
   The name printed with the message.
   */
  override readonly name = 'SaveReviewOptionsError';
}

/**
 Builds the error this command answers a refused option with.

 @param message - what is wrong with the command line

 @returns the error the command exits with

 @example
 ```ts
 throw saveReviewOptionsErrorOf('--review is required.',);
 ```
 */
function saveReviewOptionsErrorOf(message: string,): Error {
  return new SaveReviewOptionsError(message,);
}

/**
 Reads the command line.

 @param commandLine - arguments after the program name

 @returns the reviews to save, and the folders they are read from and written to

 @throws SaveReviewOptionsError when no review is named

 @example
 ```ts
 const options = parseSaveReviewOptions(['--review=a-round-1',],);
 ```
 */
export function parseSaveReviewOptions(commandLine: readonly string[],): {
  readonly reviews: readonly string[];
  readonly runsFolder: string;
  readonly reviewFolder: string;
} {
  /**
   The reviews named, in the order they were written.
   */
  const reviews = commandLine.filter(function namesAReview(word,) {
    return word.startsWith(REVIEW_OPTION,);
  },)
    .map(function reviewOf(word,) {
      return requiredText({
        value: word.slice(REVIEW_OPTION.length,),
        name: 'review',
        usage: SAVE_REVIEW_USAGE,
        errorOf: saveReviewOptionsErrorOf,
      },);
    },);
  if (reviews.length === 0) {
    throw new SaveReviewOptionsError(`--review is required at least once.\nUsage: ${SAVE_REVIEW_USAGE}`,);
  }
  /**
   The folder the runs live in.
   */
  const runsFolder = optionIn({
    commandLine,
    option: RUNS_FOLDER_OPTION,
  },);
  /**
   The folder that holds one folder per review.
   */
  const reviewFolder = optionIn({
    commandLine,
    option: REVIEW_FOLDER_OPTION,
  },);
  return {
    reviews,
    runsFolder: (runsFolder === '')
      ? join(
        homedir(),
        ...DEFAULT_RUNS_FOLDER_IN_HOME,
      )
      : runsFolder,
    reviewFolder: (reviewFolder === '')
      ? join(
        homedir(),
        ...DEFAULT_REVIEW_FOLDER_IN_HOME,
      )
      : reviewFolder,
  };
}
