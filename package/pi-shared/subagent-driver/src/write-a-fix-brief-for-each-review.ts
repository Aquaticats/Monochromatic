#!/usr/bin/env node
/**
 `write-a-fix-brief-for-each-review`: writes one brief for each review whose defects a fix agent answers, from the rules file the
 brief names first, and, when a batch list is named, the list that starts those agents. See `README.md` for the options.

 A review whose review file or diff is not there gets no brief: a fix agent sent to read a file that does not exist spends its
 run on nothing, so that review is said in one line and the rest go on.

 @module
 */

import {
  access,
  writeFile,
} from 'node:fs/promises';
import { join, } from 'node:path';
import { argv, } from 'node:process';
import type { BatchEntry, } from './batch-list.ts';
import { checkoutFolderOf, } from './checkout-of-a-command.ts';
import { errorCodeOf, } from './error-code-of.ts';
import {
  branchOfAFixFor,
  briefOfAFix,
  changeNameOfAFixFor,
} from './fix-brief.ts';
import { parseFixBriefOptions, } from './parse-fix-brief-options.ts';
import {
  diffFileOfAReview,
  reviewFileOfAReview,
} from './reviewer-of-a-change.ts';
import { rulesFileIn, } from './rules-every-pi-brief-carries.ts';

/**
 The code the operating system gives a path that is not there.
 */
const MISSING_PATH_CODE = 'ENOENT';

/**
 The top of the checkout this command runs in, which the briefs' paths are read from.
 */
const checkoutFolder = await checkoutFolderOf({ from: process.cwd(), },);

/**
 What this run of the command was asked for.
 */
const options = parseFixBriefOptions({
  commandLine: argv.slice(2,),
  checkoutFolder,
},);

/**
 The rules file every brief names first.
 */
const rulesFile = rulesFileIn({ checkoutFolder, },);

/**
 Says whether a file is there.

 @param path - the file to look for

 @returns whether it is there

 @example
 ```ts
 const there = await isThere('/reviews/a/review.md',);
 ```
 */
async function isThere(path: string,): Promise<boolean> {
  try {
    await access(path,);
    return true;
  }
  catch (error) {
    // A file that is not there is exactly what this asks about.
    if (errorCodeOf(error,) === MISSING_PATH_CODE) {
      return false;
    }
    throw error;
  }
}

/**
 The batch entries the briefs make, in the order the reviews were given.
 */
const entries: BatchEntry[] = [];
/**
 The lines to print.
 */
const lines: string[] = [];
/**
 How many reviews got no brief because their own files are not there, counted as its work goes.
 */
const counts = { withoutAReview: 0, };
/* oxlint-disable eslint/no-await-in-loop -- one review after another, each read before its brief is written, so a missing file
   names the review it belongs to. See doc/lint/sequential-awaits.md. */
for (const review of options.reviews) {
  /**
   The run and branch the fix lives on, named after the change the review read.
   */
  const fixName = changeNameOfAFixFor({ review, },);
  /**
   The files the brief sends the agent to.
   */
  const wanted: readonly string[] = [
    reviewFileOfAReview({
      reviewsFolder: options.reviewsFolder,
      review,
    },),
    diffFileOfAReview({
      reviewsFolder: options.reviewsFolder,
      review,
    },),
  ];
  /**
   The files among them that are not there.
   */
  const missing: string[] = [];
  for (const path of wanted) {
    if (!(await isThere(path,))) {
      missing.push(path,);
    }
  }
  if (missing.length > 0) {
    lines.push(`${fixName}: ${missing.join(' and ',)} is not there, so no brief was written`,);
    counts.withoutAReview += 1;
    continue;
  }
  /**
   The branch the fix's work goes on.
   */
  const branch = branchOfAFixFor({
    review,
    branchPrefix: options.branchPrefix,
  },);
  /**
   The file the brief is written to, which the batch list's entry reads.
   */
  const promptFile = join(
    options.runsFolder,
    `${fixName}-prompt.md`,
  );
  await writeFile(
    promptFile,
    briefOfAFix({
      review,
      branch,
      base: options.base,
      packageFolder: options.packageFolder,
      reviewsFolder: options.reviewsFolder,
      checkoutFolder,
      rulesFile,
    },),
    'utf8',
  );
  entries.push({
    name: fixName,
    branch,
    base: options.base,
    prompt: promptFile,
    wall: options.wallSeconds,
  },);
  lines.push(`${fixName}: brief at ${promptFile}; branch ${branch} at ${options.base}`,);
}
/* oxlint-enable eslint/no-await-in-loop */
if (options.batchFile !== undefined) {
  await writeFile(
    options.batchFile,
    `${JSON.stringify(
      entries,
      undefined,
      2
    )}\n`,
    'utf8',
  );
  lines.push(`${String(entries.length,)} entries in ${options.batchFile}`,);
}
for (const line of lines) {
  process.stdout
    .write(`${line}\n`,);
}
process.exitCode = (counts.withoutAReview === 0)
  ? 0
  : 1;
