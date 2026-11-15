#!/usr/bin/env node
/**
 `save-the-final-message-of-a-review-that-wrote-no-review-file`: writes the last assistant message of a review's newest finished run
 as that review's `review.md`, when the review wrote none, and answers what the review's own file is read as. See `README.md` for
 the options.

 A review worktree is kept until `review.md` exists, so a review whose run wrote no file is saved here rather than left unread. A
 review whose own file states a range it was not given, or states none, is answered as a review that did not cover its change and
 the command exits `1`: a review of a part of a change that answers "nothing found" is worse than no review, because its verdict
 is taken for the whole.

 @module
 */

import { argv, } from 'node:process';
import {
  type SavedReview,
  saveTheLastMessage,
} from './last-message-of-a-review.ts';
import {
  SAVE_REVIEW_USAGE,
  parseSaveReviewOptions,
} from './parse-save-review-options.ts';
import { usageWasAsked, } from './usage-of-a-command.ts';

// The word that asks for help goes to the command as an argument, and the command answers it before it reads an option
// or touches anything: mise appends a word its task's usage spec does not claim to the task's text, and `--help` is
// mise's own flag, so this is the only place it can land.
if (usageWasAsked({
  words: argv.slice(2,),
  usage: SAVE_REVIEW_USAGE,
},)) {
  process.exitCode = 0;
  process.exit(0,);
}

/**
 What this run of the command was asked for.
 */
const options = parseSaveReviewOptions(argv.slice(2,),);

/**
 What saving every review did.
 */
const saved: SavedReview[] = [];
/* oxlint-disable eslint/no-await-in-loop -- one review after another, each reading its own run's session file, so a failure names
   the review it is about. See doc/lint/sequential-awaits.md. */
for (const review of options.reviews) {
  saved.push(await saveTheLastMessage({
    runsFolder: options.runsFolder,
    reviewFolder: options.reviewFolder,
    review,
  },),);
}
/* oxlint-enable eslint/no-await-in-loop */
for (const one of saved) {
  process.stdout
    .write(`${one.line}\n`,);
}
process.exitCode = saved.every(function isReadable(one,) {
  return one.saved && (one.coverage
    ?.kind
    !== 'does-not-cover');
},)
  ? 0
  : 1;
