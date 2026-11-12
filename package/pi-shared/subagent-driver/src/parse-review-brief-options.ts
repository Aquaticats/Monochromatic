/**
 Reads the command line of `write-a-brief-for-a-review`: which change, the commit under review, what changed, the folder whose
 changes are reviewed, and where the review's own files go.

 @module
 */

import { homedir, } from 'node:os';
import { join, } from 'node:path';
import { optionIn, } from './option-in.ts';
import {
  requiredText,
} from './required-option-values.ts';
import { FOCUS_OF_A_CHANGE_REVIEW, } from './review-brief.ts';
import {
  DEFAULT_REVIEWS_FOLDER_IN_HOME,
  DEFAULT_RUNS_FOLDER_IN_HOME,
} from './reviewer-of-a-change.ts';
import { rulesFileIn, } from './rules-every-pi-brief-carries.ts';

/**
 The command's command line, for the error message.
 */
export const REVIEW_BRIEF_USAGE: string =
  'write-a-brief-for-a-review --review=<change> --commit=<commit> --range=<a>...<b> [--range=<a>...<b>]... --source=<folder> '
    + '[--focus=<text>] [--reviews-folder=<folder>] [--runs-folder=<folder>] [--rules-file=<file>]';

/**
 The option that names the change the review reads.
 */
const REVIEW_OPTION = '--review=';

/**
 The option that names the commit under review.
 */
const COMMIT_OPTION = '--commit=';

/**
 The option that names what changed, which may be given once per merge a branch was taken in by.
 */
const RANGE_OPTION = '--range=';

/**
 The option that names the folder whose changes are reviewed.
 */
const SOURCE_OPTION = '--source=';

/**
 The option that names what the reviewer looks hardest at.
 */
const FOCUS_OPTION = '--focus=';

/**
 The option that names the folder that holds one folder per review.
 */
const REVIEWS_FOLDER_OPTION = '--reviews-folder=';

/**
 The option that names the folder the runs live in.
 */
const RUNS_FOLDER_OPTION = '--runs-folder=';

/**
 The option that names the rules file the brief sends the reviewer to first.
 */
const RULES_FILE_OPTION = '--rules-file=';

/**
 A command line the brief writer cannot use.
 */
export class ReviewBriefOptionsError extends Error {
  /**
   The name printed with the message.
   */
  override readonly name = 'ReviewBriefOptionsError';
}

/**
 Builds the error this command answers a refused option with.

 @param message - what is wrong with the command line

 @returns the error the command exits with

 @example
 ```ts
 throw reviewBriefOptionsErrorOf('--review is required.',);
 ```
 */
function reviewBriefOptionsErrorOf(message: string,): Error {
  return new ReviewBriefOptionsError(message,);
}

/**
 Reads one repeatable option's values, each refused when it is empty.

 @param commandLine - arguments after the program name

 @param option - the option, with its equals sign

 @param name - the option's name, without the leading dashes

 @returns the values, in the order they were written

 @throws ReviewBriefOptionsError when one of them is empty

 @example
 ```ts
 const ranges = valuesOf({ commandLine: ['--range=a...b',], option: '--range=', name: 'range', },);
 ```
 */
function valuesOf({
  commandLine,
  option,
  name,
}: {
  readonly commandLine: readonly string[];
  readonly option: string;
  readonly name: string;
}): readonly string[] {
  return commandLine.filter(function startsWithOption(argument,) {
    return argument.startsWith(option,);
  },)
    .map(function valueOf(argument,) {
      return requiredText({
        value: argument.slice(option.length,),
        name,
        usage: REVIEW_BRIEF_USAGE,
        errorOf: reviewBriefOptionsErrorOf,
      },);
    },);
}

/**
 What the brief writer was asked for.
 */
export type ReviewBriefOptions = {
  /**
   The change the review reads.
   */
  readonly review: string;

  /**
   The commit under review.
   */
  readonly commit: string;

  /**
   What changed, each as `git diff` takes one.
   */
  readonly ranges: readonly string[];

  /**
   The folder whose changes are reviewed.
   */
  readonly source: string;

  /**
   What the reviewer looks hardest at.
   */
  readonly focus: string;

  /**
   Folder that holds one folder per review.
   */
  readonly reviewsFolder: string;

  /**
   Folder the runs live in.
   */
  readonly runsFolder: string;

  /**
   The rules file's path.
   */
  readonly rulesFile: string;
};

/**
 Reads the command line.

 @param commandLine - arguments after the program name

 @param checkoutFolder - the top of the checkout, which the rules file's default path is read from

 @returns what the brief writer was asked for

 @throws ReviewBriefOptionsError when a required option is missing, or a range is not given

 @example
 ```ts
 const options = parseReviewBriefOptions(['--review=a', '--commit=abc123', '--range=main...fix/a', '--source=package/a/src',], checkoutFolder);
 ```
 */
export function parseReviewBriefOptions({
  commandLine,
  checkoutFolder,
}: {
  readonly commandLine: readonly string[];
  readonly checkoutFolder: string;
}): ReviewBriefOptions {
  /**
   The change the review reads.
   */
  const review = requiredText({
    value: optionIn({
      commandLine,
      option: REVIEW_OPTION,
    },),
    name: 'review',
    usage: REVIEW_BRIEF_USAGE,
    errorOf: reviewBriefOptionsErrorOf,
  },);
  /**
   The commit under review.
   */
  const commit = requiredText({
    value: optionIn({
      commandLine,
      option: COMMIT_OPTION,
    },),
    name: 'commit',
    usage: REVIEW_BRIEF_USAGE,
    errorOf: reviewBriefOptionsErrorOf,
  },);
  /**
   What changed.
   */
  const ranges = valuesOf({
    commandLine,
    option: RANGE_OPTION,
    name: 'range',
  },);
  if (ranges.length === 0) {
    throw reviewBriefOptionsErrorOf(`--range is required at least once.\nUsage: ${REVIEW_BRIEF_USAGE}`,);
  }
  /**
   The folder whose changes are reviewed.
   */
  const source = requiredText({
    value: optionIn({
      commandLine,
      option: SOURCE_OPTION,
    },),
    name: 'source',
    usage: REVIEW_BRIEF_USAGE,
    errorOf: reviewBriefOptionsErrorOf,
  },);
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
   What the reviewer looks hardest at.
   */
  const focus = optionIn({
    commandLine,
    option: FOCUS_OPTION,
  },);
  return {
    review,
    commit,
    ranges,
    source,
    focus: (focus === '') ? FOCUS_OF_A_CHANGE_REVIEW : focus,
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
  };
}
