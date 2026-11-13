#!/usr/bin/env node
/**
 `check-a-review-that-ended`: reads what an ended review left (its review file, its run folder and its session), says one verdict
 with its reasons, and writes the entry that starts a review again when it needs one. See `README.md` for the options.

 The check is bookkeeping of evidence: a review file that is there, a statement of what the review read and ran that matches the
 session, at least one command run through a program built from the reviewed commit, and a run that ended by itself. A review that
 read part of the diff and says so is complete and says which files it did not read whole.

 It writes the batch entry a launcher takes for a review that ran nothing, wrote no review file or was cut, and, only when the
 caller asks with `--start`, starts it through the retrying start. It never writes over a brief, a diff or a review file.

 @module
 */

import {
  argv,
  env,
} from 'node:process';
import { setTimeout as wait, } from 'node:timers/promises';
import { checkoutFolderOf, } from './checkout-of-a-command.ts';
import {
  CHECK_LIMITS_LINE,
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
 What this run has to report with its status: whether a review was left needing something it could not do, or a restart could
 not be written.
 */
const state = { unfinished: false, };

/**
 The restarts this check wrote and the caller asked to start.
 */
const startable: RestartPlan[] = [];

/* oxlint-disable eslint/no-await-in-loop -- one review after another, because each verdict is printed before the next review is
   read, so a review that needs a restart is read within minutes. See doc/lint/sequential-awaits.md. */
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
  if (check.verdict !== 'complete') {
    state.unfinished = true;
  }
  /**
   Whether the review read every file it was given whole, which needs a second review only when the caller starts one.
   */
  const partial = (check.verdict === 'complete')
    && ((check.coverage
      ?.filesNotReadWhole
      .length
      ?? 0) > 0);
  if ((check.verdict === 'complete') && (!partial)) {
    continue;
  }
  if (partial && (!options.start)) {
    continue;
  }
  /**
   What writing the restart answered.
   */
  // oxlint-disable-next-line eslint/no-await-in-loop -- the one restart of the review this pass read, before the next review
  const restart = await restartOfAReview({
    check,
    reviewsFolder: options.reviewsFolder,
    runsFolder: options.runsFolder,
    rulesFile: options.rulesFile,
    focus: options.focus,
  },);
  if (restart.kind === 'not needed') {
    continue;
  }
  if (restart.kind === 'already written') {
    process.stdout
      .write(`${restart.line}\n`,);
    state.unfinished = true;
    continue;
  }
  if ((restart.kind === 'cannot') || (restart.kind === 'refused')) {
    process.stderr
      .write(`${restart.line}\n`,);
    state.unfinished = true;
    continue;
  }
  process.stdout
    .write(`${restart.line}\n`,);
  startable.push(restart.plan,);
}
/* oxlint-enable eslint/no-await-in-loop */

if (options.start && (startable.length > 0)) {
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
