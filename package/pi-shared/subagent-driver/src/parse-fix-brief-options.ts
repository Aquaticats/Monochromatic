/**
 Reads the command line of `write-a-fix-brief-for-each-review`: which reviews found the defects, the namespace a fix's branch
 lives under, the package it changes, and where the briefs and their batch list go. The commit a fix starts at is not an option:
 each review records the commit it read, and a fix starts there.

 @module
 */

import { homedir, } from 'node:os';
import { join, } from 'node:path';
import { FIX_BRIEF_WALL_SECONDS, } from './fix-brief.ts';
import { optionIn, } from './option-in.ts';
import {
  requiredPositiveNumber,
  requiredText,
} from './required-option-values.ts';
import {
  DEFAULT_REVIEWS_FOLDER_IN_HOME,
  DEFAULT_RUNS_FOLDER_IN_HOME,
} from './reviewer-of-a-change.ts';

/**
 The command's command line, for the error message.
 */
export const FIX_BRIEFS_USAGE: string =
  'write-a-fix-brief-for-each-review --review=<name> --name=<run name> [--review=<name> --name=<run name>]... '
    + '--branch-prefix=<prefix> --package=<folder> [--reviews-folder=<folder>] [--runs-folder=<folder>] '
    + '[--batch-file=<file>] [--wall-seconds=<n>]';

/**
 The option that names the run a fix gets, which is given once per review and in the same order.
 */
const NAME_OPTION = '--name=';

/**
 The option that names one review whose defects a fix answers, which may be given once per review.
 */
const REVIEW_OPTION = '--review=';

/**
 The option that names the namespace a fix branch lives under.
 */
const BRANCH_PREFIX_OPTION = '--branch-prefix=';

/**
 The option that names the package a fix changes.
 */
const PACKAGE_OPTION = '--package=';

/**
 The option that names the folder that holds one folder per review.
 */
const REVIEWS_FOLDER_OPTION = '--reviews-folder=';

/**
 The option that names the folder the runs live in.
 */
const RUNS_FOLDER_OPTION = '--runs-folder=';

/**
 The option that names the batch list the briefs are written into.
 */
const BATCH_FILE_OPTION = '--batch-file=';

/**
 The option that names how long a fix's run may last.
 */
const WALL_SECONDS_OPTION = '--wall-seconds=';

/**
 A command line the fix brief writer cannot use.
 */
export class FixBriefOptionsError extends Error {
  /**
   The name printed with the message.
   */
  override readonly name = 'FixBriefOptionsError';
}

/**
 Builds the error this command answers a refused option with.

 @param message - what is wrong with the command line

 @returns the error the command exits with

 @example
 ```ts
 throw fixBriefOptionsErrorOf('--branch-prefix is required.',);
 ```
 */
function fixBriefOptionsErrorOf(message: string,): Error {
  return new FixBriefOptionsError(message,);
}

/**
 What the fix brief writer was asked for.
 */
export type FixBriefOptions = {
  /**
   The reviews whose defects the fixes answer, in the order given.
   */
  readonly reviews: readonly string[];

  /**
   The name each fix run gets, in the same order as the reviews.

   A name of its own is what keeps a fix out of the worktree and the brief of the run it answers: a batch entry named after a run
   that exists restarts inside that run's worktree, on that run's branch, and its brief would then disagree with the branch its
   work goes on.
   */
  readonly names: readonly string[];

  /**
   The namespace a fix branch lives under.
   */
  readonly branchPrefix: string;

  /**
   The package a fix changes, from the top of the checkout.
   */
  readonly packageFolder: string;

  /**
   Folder that holds one folder per review.
   */
  readonly reviewsFolder: string;

  /**
   Folder the runs live in, where the briefs are written.
   */
  readonly runsFolder: string;

  /**
   The batch list the briefs are written into, when the caller named one.
   */
  readonly batchFile?: string;

  /**
   How long a fix's run may last.
   */
  readonly wallSeconds: number;

  /**
   The top of the checkout the fixes start in.
   */
  readonly checkoutFolder: string;
};

/**
 Reads the command line.

 @param commandLine - arguments after the program name

 @param checkoutFolder - the top of the checkout the fixes start in

 @returns what the fix brief writer was asked for

 @throws FixBriefOptionsError when no review, branch prefix or package is named, or a limit cannot be read

 @example
 ```ts
 const options = parseFixBriefOptions(['--review=a', '--branch-prefix=fix/', '--package=package/a',], '/repo',);
 ```
 */
export function parseFixBriefOptions({
  commandLine,
  checkoutFolder,
}: {
  readonly commandLine: readonly string[];
  readonly checkoutFolder: string;
}): FixBriefOptions {
  /**
   The reviews whose defects the fixes answer.
   */
  const reviews = commandLine.filter(function namesAReview(argument,) {
    return argument.startsWith(REVIEW_OPTION,);
  },)
    .map(function reviewOf(argument,) {
      return requiredText({
        value: argument.slice(REVIEW_OPTION.length,),
        name: 'review',
        usage: FIX_BRIEFS_USAGE,
        errorOf: fixBriefOptionsErrorOf,
      },);
    },);
  if (reviews.length === 0) {
    throw fixBriefOptionsErrorOf(`--review is required at least once.\nUsage: ${FIX_BRIEFS_USAGE}`,);
  }
  /**
   The names the fix runs get, in the order given.
   */
  const names = commandLine.filter(function namesAFix(argument,) {
    return argument.startsWith(NAME_OPTION,);
  },)
    .map(function nameOf(argument,) {
      return requiredText({
        value: argument.slice(NAME_OPTION.length,),
        name: 'name',
        usage: FIX_BRIEFS_USAGE,
        errorOf: fixBriefOptionsErrorOf,
      },);
    },);
  if (names.length !== reviews.length) {
    throw fixBriefOptionsErrorOf(
      `--name is required once per --review, and ${String(reviews.length,)} reviews got ${String(names.length,)} names.\n`
        + `Usage: ${FIX_BRIEFS_USAGE}`,
    );
  }
  /**
   The first name that is taken twice, or that names a review rather than a fix.
   */
  const clash = names.find(function isTaken(
    name,
    at,
  ) {
    return (names.indexOf(name,) !== at) || reviews.includes(name,);
  },);
  if (clash !== undefined) {
    throw fixBriefOptionsErrorOf(
      `--name=${clash} is already a name in this command line: a fix run needs a name of its own, different from every other `
        + `fix and from the reviews it answers.\nUsage: ${FIX_BRIEFS_USAGE}`,
    );
  }
  /**
   The namespace a fix branch lives under.
   */
  const branchPrefix = requiredText({
    value: optionIn({
      commandLine,
      option: BRANCH_PREFIX_OPTION,
    },),
    name: 'branch-prefix',
    usage: FIX_BRIEFS_USAGE,
    errorOf: fixBriefOptionsErrorOf,
  },);
  /**
   The package a fix changes.
   */
  const packageFolder = requiredText({
    value: optionIn({
      commandLine,
      option: PACKAGE_OPTION,
    },),
    name: 'package',
    usage: FIX_BRIEFS_USAGE,
    errorOf: fixBriefOptionsErrorOf,
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
   The batch list the briefs are written into.
   */
  const batchFile = optionIn({
    commandLine,
    option: BATCH_FILE_OPTION,
  },);
  /**
   How long a fix's run may last.
   */
  const wallSeconds = optionIn({
    commandLine,
    option: WALL_SECONDS_OPTION,
  },);
  return {
    reviews,
    names,
    branchPrefix,
    packageFolder,
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
    ...((batchFile === '')
      ? {}
      : { batchFile, }),
    wallSeconds: (wallSeconds === '')
      ? FIX_BRIEF_WALL_SECONDS
      : requiredPositiveNumber({
        value: wallSeconds,
        name: 'wall-seconds',
        usage: FIX_BRIEFS_USAGE,
        errorOf: fixBriefOptionsErrorOf,
      },),
    checkoutFolder,
  };
}
