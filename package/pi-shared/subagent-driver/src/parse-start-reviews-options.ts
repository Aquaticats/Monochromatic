/**
 Reads the command line of `start-reviews-of-finished-branches`: which branches to review, the integration branch their changes
 are read against, the folder whose changes are reviewed, and where the reviews and their runs live.

 @module
 */

import { homedir, } from 'node:os';
import { join, } from 'node:path';
import { optionIn, } from './option-in.ts';
import { requiredText, } from './required-option-values.ts';
import {
  DEFAULT_REVIEWS_FOLDER_IN_HOME,
  DEFAULT_RUNS_FOLDER_IN_HOME,
} from './reviewer-of-a-change.ts';

/**
 The command's command line, for the error message.
 */
export const START_REVIEWS_USAGE: string =
  'start-reviews-of-finished-branches --branch=<branch> [--branch=<branch>]... --integration=<branch> --source=<folder> '
    + '[--reviews-folder=<folder>] [--runs-folder=<folder>]';

/**
 The option that names one branch to review, which may be given once per branch.
 */
const BRANCH_OPTION = '--branch=';

/**
 The option that names the integration branch a branch's change is read against.
 */
const INTEGRATION_OPTION = '--integration=';

/**
 The option that names the folder whose changes are reviewed.
 */
const SOURCE_OPTION = '--source=';

/**
 The option that names the folder that holds one folder per review.
 */
const REVIEWS_FOLDER_OPTION = '--reviews-folder=';

/**
 The option that names the folder the runs live in.
 */
const RUNS_FOLDER_OPTION = '--runs-folder=';

/**
 A command line the review starter cannot use.
 */
export class StartReviewsOptionsError extends Error {
  /**
   The name printed with the message.
   */
  override readonly name = 'StartReviewsOptionsError';
}

/**
 Builds the error this command answers a refused option with.

 @param message - what is wrong with the command line

 @returns the error the command exits with

 @example
 ```ts
 throw startReviewsOptionsErrorOf('--integration is required.',);
 ```
 */
function startReviewsOptionsErrorOf(message: string,): Error {
  return new StartReviewsOptionsError(message,);
}

/**
 What the review starter was asked for.
 */
export type StartReviewsOptions = {
  /**
   The branches a review is wanted for, in the order given.
   */
  readonly branches: readonly string[];

  /**
   The integration branch a branch's change is read against.
   */
  readonly integration: string;

  /**
   The folder whose changes are reviewed.
   */
  readonly source: string;

  /**
   Folder that holds one folder per review.
   */
  readonly reviewsFolder: string;

  /**
   Folder the runs live in.
   */
  readonly runsFolder: string;

  /**
   The top of the checkout the worktrees are made in.
   */
  readonly checkoutFolder: string;
};

/**
 Reads the command line.

 @param commandLine - arguments after the program name

 @param checkoutFolder - the top of the checkout the command's git questions are about

 @returns what the review starter was asked for

 @throws StartReviewsOptionsError when no branch, integration or source is named

 @example
 ```ts
 const options = parseStartReviewsOptions(['--branch=fix/a', '--integration=main', '--source=package/a/src',], '/repo',);
 ```
 */
export function parseStartReviewsOptions({
  commandLine,
  checkoutFolder,
}: {
  readonly commandLine: readonly string[];
  readonly checkoutFolder: string;
}): StartReviewsOptions {
  /**
   The branches a review is wanted for.
   */
  const branches = commandLine.filter(function namesABranch(argument,) {
    return argument.startsWith(BRANCH_OPTION,);
  },)
    .map(function branchOf(argument,) {
      return requiredText({
        value: argument.slice(BRANCH_OPTION.length,),
        name: 'branch',
        usage: START_REVIEWS_USAGE,
        errorOf: startReviewsOptionsErrorOf,
      },);
    },);
  if (branches.length === 0) {
    throw startReviewsOptionsErrorOf(`--branch is required at least once.\nUsage: ${START_REVIEWS_USAGE}`,);
  }
  /**
   The integration branch a branch's change is read against.
   */
  const integration = requiredText({
    value: optionIn({
      commandLine,
      option: INTEGRATION_OPTION,
    },),
    name: 'integration',
    usage: START_REVIEWS_USAGE,
    errorOf: startReviewsOptionsErrorOf,
  },);
  /**
   The folder whose changes are reviewed.
   */
  const source = requiredText({
    value: optionIn({
      commandLine,
      option: SOURCE_OPTION,
    },),
    name: 'source',
    usage: START_REVIEWS_USAGE,
    errorOf: startReviewsOptionsErrorOf,
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
  return {
    branches,
    integration,
    source,
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
    checkoutFolder,
  };
}
