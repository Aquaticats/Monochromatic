#!/usr/bin/env node
/**
 `write-a-brief-for-a-review`: writes what one review of a change reads, which are the diff of its ranges under the reviewed source
 folder, the brief that names its files, and the folder its reproductions go in. See `README.md` for the options.

 It makes no worktree and starts nothing: `start-reviews-of-finished-branches` writes this brief for each finished branch and then
 starts the run, whose worktree begins at the commit this brief names.

 @module
 */

import { argv, } from 'node:process';
import { checkoutFolderOf, } from './checkout-of-a-command.ts';
import { gitRunnerOf, } from './git-command.ts';
import {
  REVIEW_BRIEF_USAGE,
  parseReviewBriefOptions,
} from './parse-review-brief-options.ts';
import { runCommandWithChildProcess, } from './run-command.ts';
import { usageWasAsked, } from './usage-of-a-command.ts';
import { writeReviewBrief, } from './writing-a-review-brief.ts';

// The word that asks for help goes to the command as an argument, and the command answers it before it reads an option
// or touches anything: mise appends a word its task's usage spec does not claim to the task's text, and `--help` is
// mise's own flag, so this is the only place it can land.
if (usageWasAsked({
  words: argv.slice(2,),
  usage: REVIEW_BRIEF_USAGE,
},)) {
  process.exitCode = 0;
  process.exit(0,);
}

/**
 The top of the checkout this command runs in, which every path it writes is read from.
 */
const checkoutFolder = await checkoutFolderOf({ from: process.cwd(), },);

/**
 What this run of the command was asked for.
 */
const options = parseReviewBriefOptions({
  commandLine: argv.slice(2,),
  checkoutFolder,
},);

/**
 What writing the brief answered.
 */
const briefed = await writeReviewBrief({
  git: gitRunnerOf(runCommandWithChildProcess,),
  checkoutFolder,
  review: options.review,
  commit: options.commit,
  ranges: options.ranges,
  source: options.source,
  focus: options.focus,
  reviewsFolder: options.reviewsFolder,
  runsFolder: options.runsFolder,
  rulesFile: options.rulesFile,
},);

process.stdout
  .write(`${briefed.line}\n`,);
process.exitCode = briefed.written
  ? 0
  : 1;
