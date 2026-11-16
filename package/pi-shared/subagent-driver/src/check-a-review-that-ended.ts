#!/usr/bin/env node
/**
 `check-a-review-that-ended`: reads what an ended review left (its review file, its run folder and its session), says one verdict
 with its reasons, and gives a thin or incomplete first review one second review, on the other model of the pair. See `README.md` for
 the options.

 The check is bookkeeping of evidence: a review file that is there, a statement of what the review read and ran that matches the
 session, at least one command run through a program a build wrote, and a run that ended by itself. A review that read part of the
 diff and says so is complete and says which files it did not read whole; it is thin, and a second review answers it once.

 A second review is started by this command by itself, once, on the other reviewer of the pair the project's rules name. Where no Pi
 model serves that reviewer, the brief and the note saying the review is owed are written, and nothing is started. A second review
 that is still thin is reported and left for whoever coordinates, never started a third time.

 @module
 */

import {
  argv,
  env,
} from 'node:process';
import { setTimeout as wait, } from 'node:timers/promises';
import { checkoutFolderOf, } from './checkout-of-a-command.ts';
import { CHECK_LIMITS_LINE, } from './review-check-types.ts';
import {
  checkAReview,
  checkLineOf,
} from './checking-a-review.ts';
import {
  keepFailedOutput,
  reportOnStandardError,
} from './kept-start-failure.ts';
import {
  CHECK_REVIEW_USAGE,
  parseCheckReviewOptions,
} from './parse-check-review-options.ts';
import {
  otherReviewerOf,
  type ReviewerOfAChange,
} from './reviewer-of-a-change.ts';
import type { RestartPlan, } from './restart-brief.ts';
import {
  restartOfAReview,
  startRestarts,
} from './restarting-a-review.ts';
import {
  retryingStartEntry,
  startThroughTheRetryingStart,
} from './start-through-the-retrying-start.ts';
import { reportOnStandardOutput, } from './standard-output-of-a-command.ts';
import { usageWasAsked, } from './usage-of-a-command.ts';

// The word that asks for help goes to the command as an argument, and the command answers it before it reads an option
// or touches anything: mise appends a word its task's usage spec does not claim to the task's text, and `--help` is
// mise's own flag, so this is the only place it can land.
if (usageWasAsked({
  words: argv.slice(2,),
  usage: CHECK_REVIEW_USAGE,
},)) {
  process.exitCode = 0;
  process.exit(0,);
}

/**
 The top of the checkout this command runs in, which the rules file's default path is read from.
 */
const checkoutFolder = await checkoutFolderOf({ from: process.cwd(), },);

/**
 What this run of the command was asked for.
 */
const options = parseCheckReviewOptions({
  commandLine: argv.slice(2,),
  checkoutFolder,
},);

/**
 What this run has to report with its status: whether a review was left needing something it could not do, or a second review could
 not be written.
 */
const state = { unfinished: false, };

/**
 The second reviews this check wrote and whose reviewer a Pi run can serve, in the order their reviews were given.
 */
const startable: RestartPlan[] = [];

/* oxlint-disable eslint/no-await-in-loop -- one review after another, because each verdict is printed before the next review is
   read, so a review that needs a second one is read within minutes. See doc/lint/sequential-awaits.md. */
for (const review of options.reviews) {
  /**
   What the review's files and run say about it.
   */
  // oxlint-disable-next-line eslint/no-await-in-loop -- the one review this pass reads, before its line is printed
  const check = await checkAReview({
    review,
    reviewsFolder: options.reviewsFolder,
    runsFolder: options.runsFolder,
  },);
  process.stdout
    .write(`${checkLineOf({ check, },)}\n`,);
  if ((check.verdict === 'complete') && (!check.thin)) {
    continue;
  }
  state.unfinished = true;
  /**
   The other reviewer of the pair this review's first run did not start on.
   */
  const reviewer: ReviewerOfAChange = otherReviewerOf({
    provider: check.reviewer
      .provider,
    model: check.reviewer
      .model,
  },);
  /**
   What writing the second review answered.
   */
  // oxlint-disable-next-line eslint/no-await-in-loop -- the one second review of the review this pass read, before the next
  const restart = await restartOfAReview({
    check,
    reviewsFolder: options.reviewsFolder,
    runsFolder: options.runsFolder,
    rulesFile: options.rulesFile,
    focus: options.focus,
    reviewer,
  },);
  if (restart.kind === 'not needed') {
    continue;
  }
  if ((restart.kind === 'cannot') || (restart.kind === 'refused')) {
    process.stderr
      .write(`${restart.line}\n`,);
    continue;
  }
  process.stdout
    .write(`${restart.line}\n`,);
  if (restart.kind === 'written') {
    startable.push(restart.plan,);
  }
}
/* oxlint-enable eslint/no-await-in-loop */

if (startable.length > 0) {
  /**
   The retrying start this package ships beside this command.
   */
  const retryingStart = await retryingStartEntry();
  await startRestarts({
    plans: startable,
    runsFolder: options.runsFolder,
    checkoutFolder,
    seams: {
      start: async function start({
        name,
        words,
      },) {
        /**
         What the retrying start answered.
         */
        const answered = await startThroughTheRetryingStart({
          entry: retryingStart,
          words,
          environment: env,
          report: reportOnStandardOutput,
        },);
        if (answered.status === 0) {
          return answered;
        }
        /**
         Where the whole of what the failed start printed was kept.
         */
        const kept = await keepFailedOutput({
          runsFolder: options.runsFolder,
          name,
          output: answered.output,
          report: reportOnStandardError,
        },);
        return {
          ...answered,
          ...kept,
        };
      },
      wait,
      report: reportOnStandardOutput,
    },
  },);
}

process.stdout
  .write(`${CHECK_LIMITS_LINE}\n`,);
process.exitCode = state.unfinished ? 1 : 0;
