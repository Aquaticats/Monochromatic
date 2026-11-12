#!/usr/bin/env node
/**
 `start-reviews-of-finished-branches`: writes a review's brief for each finished branch named, starts one review of each through
 the retrying start on the review pool, and saves the last message of a review that ended without writing its review file. See
 `README.md` for the options.

 A branch that was already reviewed, one whose change leaves nothing under the reviewed source folder, and one whose brief could
 not be written are each said in one line, and the rest go on.

 @module
 */

import { access, } from 'node:fs/promises';
import {
  argv,
  env,
} from 'node:process';
import { setTimeout as wait, } from 'node:timers/promises';
import { checkoutFolderOf, } from './checkout-of-a-command.ts';
import { errorCodeOf, } from './error-code-of.ts';
import { gitRunnerOf, } from './git-command.ts';
import {
  keepFailedOutput,
  reportOnStandardError,
} from './kept-start-failure.ts';
import { saveTheLastMessage, } from './last-message-of-a-review.ts';
import { outputOfGit, } from './material-of-a-review.ts';
import { parseStartReviewsOptions, } from './parse-start-reviews-options.ts';
import { FOCUS_OF_A_CHANGE_REVIEW, } from './review-brief.ts';
import {
  briefFileOfAReview,
  reviewFileOfAReview,
  runNameOfAReview,
} from './reviewer-of-a-change.ts';
import { startReviews, } from './reviews-of-finished-branches.ts';
import { rulesFileIn, } from './rules-every-pi-brief-carries.ts';
import { runCommandWithChildProcess, } from './run-command.ts';
import {
  retryingStartEntry,
  startThroughTheRetryingStart,
} from './start-through-the-retrying-start.ts';
import { writeReviewBrief, } from './writing-a-review-brief.ts';

/**
 The code the operating system gives a path that is not there.
 */
const MISSING_PATH_CODE = 'ENOENT';

/**
 The top of the checkout this command runs in, which every git question and worktree is about.
 */
const checkoutFolder = await checkoutFolderOf({ from: process.cwd(), },);

/**
 What this run of the command was asked for.
 */
const options = parseStartReviewsOptions({
  commandLine: argv.slice(2,),
  checkoutFolder,
},);

/**
 How this command asks git a question.
 */
const git = gitRunnerOf(runCommandWithChildProcess,);

/**
 The retrying start this package ships beside this command.
 */
const retryingStart = await retryingStartEntry();

/**
 Says whether a review already wrote its review file.

 @param review - the change the review reads

 @returns whether the file is there

 @example
 ```ts
 const written = await reviewIsWritten({ review: 'a', },);
 ```
 */
async function reviewIsWritten({ review, }: { readonly review: string },): Promise<boolean> {
  try {
    await access(reviewFileOfAReview({
      reviewsFolder: options.reviewsFolder,
      review,
    },),);
    return true;
  }
  catch (error) {
    // A review file that is not there is exactly what this asks about.
    if (errorCodeOf(error,) === MISSING_PATH_CODE) {
      return false;
    }
    throw error;
  }
}

/**
 Starts one review per branch and answers every line it printed.
 */
const started = await startReviews({
  branches: options.branches,
  integration: options.integration,
  runsFolder: options.runsFolder,
  checkoutFolder,
  briefFileOf: function briefFileOf({ review, },) {
    return briefFileOfAReview({
      reviewsFolder: options.reviewsFolder,
      review,
    },);
  },
  seams: {
    reviewIsWritten,
    branchTip: async function branchTip({ branch, },) {
      return (await outputOfGit({
        git,
        checkoutFolder,
        words: [
          'rev-parse',
          branch,
        ],
      },)).trim();
    },
    integrationMerges: async function integrationMerges() {
      return (await outputOfGit({
        git,
        checkoutFolder,
        words: [
          'log',
          '--first-parent',
          '--merges',
          '--format=%H %P',
          options.integration,
        ],
      },)).split('\n',)
        .filter(function isLine(line,) {
          return line !== '';
        },);
    },
    brief: async function brief({
      review,
      commit,
      range,
    },) {
      return await writeReviewBrief({
        git,
        checkoutFolder,
        review,
        commit,
        ranges: [range,],
        source: options.source,
        focus: FOCUS_OF_A_CHANGE_REVIEW,
        reviewsFolder: options.reviewsFolder,
        runsFolder: options.runsFolder,
        rulesFile: rulesFileIn({ checkoutFolder, },),
      },);
    },
    start: async function start({
      review,
      words,
    },) {
      /**
       What the retrying start answered for this review.
       */
      const answered = await startThroughTheRetryingStart({
        entry: retryingStart,
        words,
        environment: env,
      },);
      if (answered.status === 0) {
        return answered;
      }
      /**
       Where the whole of what the failed start printed was kept.
       */
      const kept = await keepFailedOutput({
        runsFolder: options.runsFolder,
        name: runNameOfAReview({ review, },),
        output: answered.output,
        report: reportOnStandardError,
      },);
      return {
        ...answered,
        ...kept,
      };
    },
    save: async function save({ review, },) {
      return await saveTheLastMessage({
        runsFolder: options.runsFolder,
        reviewFolder: options.reviewsFolder,
        review,
      },);
    },
    wait,
  },
},);

for (const line of started.lines) {
  process.stdout
    .write(`${line}\n`,);
}
process.exitCode = started.statuses
  .every(function endedWell(status,) {
    return status === '0';
  },)
  ? 0
  : 1;
