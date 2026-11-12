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
  mkdir,
  readFile,
  readdir,
  writeFile,
} from 'node:fs/promises';
import {
  basename,
  join,
} from 'node:path';
import { argv, } from 'node:process';
import type { BatchEntry, } from './batch-list.ts';
import { checkoutFolderOf, } from './checkout-of-a-command.ts';
import { errorCodeOf, } from './error-code-of.ts';
import {
  branchOfAFixFor,
  briefOfAFix,
} from './fix-brief.ts';
import {
  type ReviewReading,
  ReviewReadingError,
  readingIn,
} from './material-of-a-review.ts';
import {
  FIX_BRIEFS_USAGE,
  parseFixBriefOptions,
} from './parse-fix-brief-options.ts';
import {
  diffFileOfAReview,
  readingFileOfAReview,
  reviewFileOfAReview,
} from './reviewer-of-a-change.ts';
import { rulesFileIn, } from './rules-every-pi-brief-carries.ts';
import { usageWasAsked, } from './usage-of-a-command.ts';

// The word that asks for help goes to the command as an argument, and the command answers it before it reads an option
// or touches anything: mise appends a word its task's usage spec does not claim to the task's text, and `--help` is
// mise's own flag, so this is the only place it can land.
if (usageWasAsked({
  words: argv.slice(2,),
  usage: FIX_BRIEFS_USAGE,
},)) {
  process.exitCode = 0;
  process.exit(0,);
}

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
 Reads what one review recorded it read.

 @param review - the review whose defects the fix answers

 @param reviewsFolder - folder that holds one folder per review

 @returns what the review read, or nothing when it recorded nothing readable

 @example
 ```ts
 const reading = await readingOfAReview({ review: 'a-round-1', reviewsFolder: '/reviews', },);
 ```
 */
async function readingOfAReview({
  review,
  reviewsFolder,
}: {
  readonly review: string;
  readonly reviewsFolder: string;
}): Promise<{ readonly reading?: ReviewReading }> {
  /**
   The file that records what the review read.
   */
  const file = readingFileOfAReview({
    reviewsFolder,
    review,
  },);
  try {
    return { reading: readingIn({
      shown: await readFile(
        file,
        'utf8',
      ),
      file,
    },), };
  }
  catch (error) {
    // A review that recorded nothing, or recorded something no reader can follow, gives a fix no commit to start at; the
    // caller says so in one line and the other reviews go on.
    if ((errorCodeOf(error,) === MISSING_PATH_CODE) || (error instanceof ReviewReadingError)
      || (error instanceof SyntaxError)) {
      return {};
    }
    throw error;
  }
}

/**
 What is already at the path one plan's brief is written to.
 */
type BriefAtThePath = {
  /**
   Whether a file is there at all.
   */
  readonly there: boolean;

  /**
   Whether it holds exactly the text this command would write, so writing it would say nothing new.
   */
  readonly same: boolean;
};

/**
 Reads what is already at the path one plan's brief is written to.

 @param plan - the fix's plan

 @returns whether a file is there, and whether it holds the text this command would write

 @example
 ```ts
 const there = await briefAlreadyThere(plan);
 ```
 */
async function briefAlreadyThere(plan: FixPlan,): Promise<BriefAtThePath> {
  try {
    return {
      there: true,
      same: (await readFile(
        plan.promptFile,
        'utf8',
      )) === plan.brief,
    };
  }
  catch (error) {
    // A brief that is not there is nothing to keep; anything else is a folder this account cannot read, which the caller must see.
    if (errorCodeOf(error,) === MISSING_PATH_CODE) {
      return {
        there: false,
        same: false,
      };
    }
    throw error;
  }
}

/**
 Reads what the runs folder already holds of each plan's name, and each brief that says something else.

 A brief is the file this command writes, and a file at that path with other text is somebody's work: it is refused rather than
 replaced. A name the folder already holds is refused too, because a batch entry named after a run that exists restarts inside
 that run's worktree, on that run's branch.

 @param plans - the fixes to write

 @param runsFolder - the folder the runs and their briefs live in

 @returns the plans that can be written, and one line per refusal naming what holds the name or the file

 @example
 ```ts
 const checks = await checkedAgainstTheRunsFolder({ plans, runsFolder: '/runs', },);
 ```
 */
async function checkedAgainstTheRunsFolder({
  plans,
  runsFolder,
}: {
  readonly plans: readonly FixPlan[];
  readonly runsFolder: string;
},): Promise<RunsFolderCheck> {
  /**
   What the runs folder holds, which is one entry per run, brief or failure log.
   */
  const held = await readdir(runsFolder,);
  /**
   The plans that can be written, and the refusals.
   */
  const checked: PlansAndRefusals = {
    writable: [],
    refused: [],
  };
  for (const plan of plans) {
    /**
     The brief's own file name, which is one of the entries of this name.
     */
    const briefName = basename(plan.promptFile,);
    /**
     What is at that path now, which is the file this command would write.
     */
    // oxlint-disable-next-line eslint/no-await-in-loop -- one fix after another, each brief read before the next, so a refusal names the file it belongs to. See doc/lint/sequential-awaits.md.
    const brief = await briefAlreadyThere(plan,);
    /**
     The file this plan's brief would replace with other text, or empty text when nothing is in the way.
     */
    const briefInTheWay = (brief.there && (!brief.same)) ? plan.promptFile : '';
    /**
     What else the folder holds of this name: a run of it, a failure log of it, or another brief of it.
     */
    const others = held.filter(function isOfThisName(entry,) {
      return entry.startsWith(`${plan.name}-`,) && (entry !== briefName);
    },);
    /**
     The first entry of this name that is not the brief, or empty text when the name is free.
     */
    const nameHolder = (others.length === 0) ? '' : join(
      runsFolder,
      others[0] ?? '',
    );
    /**
     Why this plan cannot be written, or empty text when it can.
     */
    const refusal = (briefInTheWay === '')
      ? ((nameHolder === '')
        ? ''
        : `${plan.name}: ${nameHolder} is already there, so this name is taken; a fix run needs a name no run, brief or `
          + 'failure log holds')
      : `${plan.name}: ${briefInTheWay} is already there with other text, so it was not replaced; give this fix a name whose `
        + 'brief does not exist yet';
    if (refusal === '') {
      checked.writable
        .push(plan,);
    }
    else {
      checked.refused
        .push(refusal,);
    }
  }
  return checked;
}

/**
 What one review's fix needs, once every file it reads is there.
 */
type FixPlan = {
  /**
   The review whose defects the fix answers.
   */
  readonly review: string;

  /**
   The run name the fix gets, which the caller gave and which nothing else holds.
   */
  readonly name: string;

  /**
   The branch the fix's work goes on.
   */
  readonly branch: string;

  /**
   The commit the review read, which the fix starts at.
   */
  readonly base: string;

  /**
   What the review read, in the review's own words.
   */
  readonly subject: string;

  /**
   The file the brief is written to, which the batch list's entry reads.
   */
  readonly promptFile: string;

  /**
   The brief's text.
   */
  readonly brief: string;
};

/**
 The plans that can be written, and one line per plan that cannot.
 */
type PlansAndRefusals = {
  /**
   The plans whose name nothing in the runs folder holds, and whose brief is not there or is the same text.
   */
  readonly writable: FixPlan[];

  /**
   One line per name already taken or brief that says something else, naming what holds it.
   */
  readonly refused: string[];
};

/**
 What a fix needs of the runs folder before anything is written.
 */
type RunsFolderCheck = {
  /**
   The plans whose name nothing in the runs folder holds, and whose brief is not there or is the same text.
   */
  readonly writable: readonly FixPlan[];

  /**
   One line per name already taken, a brief that says something else, naming what holds it.
   */
  readonly refused: readonly string[];
};

/**
 The lines to print.
 */
const lines: string[] = [];
/**
 How many reviews got no brief, counted as its work goes.
 */
const counts = { withoutAReview: 0, };
/**
 The plans whose brief can be written, in the order the reviews were given.
 */
const plans: FixPlan[] = [];
/* oxlint-disable eslint/no-await-in-loop -- one review after another, each read before its brief is built, so a missing file
   names the review it belongs to. See doc/lint/sequential-awaits.md. */
/**
 The reviews whose defects the fixes answer, in the order given.
 */
const { reviews, } = options;
for (const [at, review,] of reviews.entries()) {
  /**
   The run the fix lives on, which the caller named: a name of the review's own would put the fix inside the review's worktree.
   */
  const fixName = options.names[at] ?? '';
  /**
   What the review recorded it read, which is where the fix starts.
   */
  const { reading, } = await readingOfAReview({
    review,
    reviewsFolder: options.reviewsFolder,
  },);
  /**
   The file that records what the review read, which a fix starts at.
   */
  const readingFile = readingFileOfAReview({
    reviewsFolder: options.reviewsFolder,
    review,
  },);
  /**
   The files the brief sends the agent to, with the record of what it read.
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
    readingFile,
  ];
  /**
   The files among them that are not there.
   */
  const missing: string[] = [];
  for (const path of wanted) {
    // oxlint-disable-next-line eslint/no-await-in-loop -- the handful of files one review names, each looked for before the next
    if (!(await isThere(path,))) {
      missing.push(path,);
    }
  }
  if (missing.length > 0) {
    lines.push(`${fixName}: ${missing.join(' and ',)} is not there, so no brief was written`,);
    counts.withoutAReview += 1;
    continue;
  }
  if (reading === undefined) {
    lines.push(`${fixName}: ${readingFile} names no commit, ranges and source folder, so where a fix would start cannot be read`,);
    counts.withoutAReview += 1;
    continue;
  }
  /**
   The branch the fix's work goes on, which a repeated segment of the prefix refuses here rather than on the branch list.
   */
  const branch = branchOfAFixFor({
    name: fixName,
    branchPrefix: options.branchPrefix,
  },);
  /**
   What the review read, in its own words, which the brief names so a fix answers the change that was reviewed.
   */
  const subject = `${reading.ranges
    .join(' and ',)} at commit ${reading.commit}`;
  plans.push({
    review,
    name: fixName,
    branch,
    base: reading.commit,
    subject,
    promptFile: join(
      options.runsFolder,
      `${fixName}-prompt.md`,
    ),
    brief: briefOfAFix({
      name: fixName,
      review,
      branch,
      base: reading.commit,
      subject,
      packageFolder: options.packageFolder,
      reviewsFolder: options.reviewsFolder,
      checkoutFolder,
      rulesFile,
    },),
  },);
}
/* oxlint-enable eslint/no-await-in-loop */
// A runs folder the caller names and nothing made yet is made here rather than refused: the briefs go into it, and a command that
// fails on a missing folder says nothing about the reviews it was given.
await mkdir(
  options.runsFolder,
  { recursive: true, },
);
/**
 What the runs folder says about the plans, before any of them is written.
 */
const checks = await checkedAgainstTheRunsFolder({
  plans,
  runsFolder: options.runsFolder,
},);
/**
 How many fixes this run refused, which is what its status reports along with the reviews that gave it nothing to write.
 */
const refusedCount = checks.refused
  .length;
for (const refused of checks.refused) {
  // A refusal is not a line about work done: it goes where a caller reads failures, with the status that says one happened.
  process.stderr
    .write(`${refused}\n`,);
}
/**
 The entries the batch list holds, which are the fixes that can be started.
 */
const entries: BatchEntry[] = [];
for (const plan of checks.writable) {
  // What is already there with this text is the same brief, so it is not written twice and nothing is discarded.
  /**
   What is at this brief's path now.
   */
  // oxlint-disable-next-line eslint/no-await-in-loop -- one brief written before the next is read, so a failure names its fix
  const there = await briefAlreadyThere(plan,);
  if (!there.same) {
    // oxlint-disable-next-line eslint/no-await-in-loop -- the brief of one fix, written before the next is read
    await writeFile(
      plan.promptFile,
      plan.brief,
      'utf8',
    );
  }
  entries.push({
    name: plan.name,
    branch: plan.branch,
    base: plan.base,
    prompt: plan.promptFile,
    wall: options.wallSeconds,
  },);
  lines.push(`${plan.name}: brief at ${plan.promptFile}; branch ${plan.branch} at ${plan.base}, what the review read: ${plan.subject}`,);
}
// A refused run wrote a list holding no entry of its own, which replaced whatever list the caller named with nothing: the list is
// written only when every fix was accepted.
if ((options.batchFile !== undefined) && (refusedCount > 0)) {
  lines.push(`${options.batchFile} was not written, because a fix was refused`,);
}
else if (options.batchFile !== undefined) {
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
process.exitCode = ((counts.withoutAReview === 0) && (refusedCount === 0))
  ? 0
  : 1;
