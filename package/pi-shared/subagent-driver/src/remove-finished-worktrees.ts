#!/usr/bin/env node
/**
 `remove-finished-worktrees`: removes the worktree of every finished Pi run, and says why it left each other one alone. Without
 `--apply` it only lists what it would do. See `README.md` for the options.

 A run's worktree is removed only when its unit has ended, the run finished its work, no brief is waiting for a slot, nothing of
 the run works in it, and it holds nothing uncommitted. Nothing is ever removed by force, and every branch is kept.

 @module
 */

import { readFile, } from 'node:fs/promises';
import { homedir, } from 'node:os';
import { join, } from 'node:path';
import {
  argv,
  cwd,
  env,
  stdout,
} from 'node:process';
import {
  briefQueuedAfter,
  decideWorktree,
  folderSomeProcessWorksIn,
  ownFoldersOf,
  pausedNamesIn,
  readFileOrNothing,
  runNameOfWorktree,
  runWorktreesIn,
  worktreesIn,
} from './finished-worktrees.ts';
import {
  NO_FINISHED_RUN_OF_THAT_NAME,
  SUMMARY_FILE_NAME,
  attemptsOfARun,
  entriesOrNothingIn,
  isThere,
} from './newest-finished-run-of-a-name.ts';
import { optionIn, } from './option-in.ts';
import { fieldsOf, } from './read-pi-fields.ts';
import {
  type ShownCommandEnding,
  runCommandShowingItsOutput,
  runCommandWithChildProcess,
} from './run-command.ts';
import { usageWasAsked, } from './usage-of-a-command.ts';
import { unitStateOf, } from './wait-for-a-unit-to-end.ts';
import { processesAtTheirFolders, } from './wake-a-run-whose-work-outlives-its-turn.ts';

/**
 The start of a Kimi review's run name.
 */
const REVIEW_RUN_START = 'kimi-review-';

/**
 The file a review's findings are read from.
 */
const REVIEW_FILE_NAME = 'review.md';

/**
 The option that names the checkout whose worktrees the launcher makes.
 */
const CHECKOUT_FOLDER_OPTION = '--checkout-folder=';

/**
 The option that names the folder the runs live in.
 */
const RUNS_FOLDER_OPTION = '--runs-folder=';

/**
 The option that names the folder that holds one folder per review.
 */
const REVIEW_FOLDER_OPTION = '--review-folder=';

/**
 The option that names a list of names that were paused, one per line after its first word.
 */
const PAUSED_FILE_OPTION = '--paused-file=';

/**
 The option that names the folder holding one entry per process.
 */
const PROCESS_FOLDER_OPTION = '--process-folder=';

/**
 The option that asks for the removals to be done rather than listed.
 */
const APPLY_OPTION = '--apply';

/**
 The option that names a folder inside the worktree to clean before it is removed.
 */
const CLEAN_FOLDER_OPTION = '--clean-folder=';

/**
 The option that names one word of the clean command.
 */
const CLEAN_WORD_OPTION = '--clean-word=';

/**
 The parts of the coordinator's runs folder below the home directory of whoever runs this.
 */
const DEFAULT_RUNS_FOLDER_IN_HOME: readonly string[] = [
  'temp',
  'agent',
  'pi-runs',
];

/**
 The parts of the coordinator's reviews folder below the home directory of whoever runs this.
 */
const DEFAULT_REVIEW_FOLDER_IN_HOME: readonly string[] = [
  'temp',
  'agent',
  'kimi-reviews',
];

/**
 The command's command line, which `--help` prints and an option the command cannot use is named against.
 */
const USAGE =
  'remove-finished-worktrees [--checkout-folder=<folder>] [--runs-folder=<folder>] [--review-folder=<folder>] '
    + '[--paused-file=<file>] [--process-folder=<folder>] [--apply] [--clean-folder=<path>] [--clean-word=<word>]...';

/**
 The words the command was given.
 */
const words = argv.slice(2,);

// The word that asks for help goes to the command as an argument, and the command answers it before it reads an option
// or touches anything: mise appends a word its task's usage spec does not claim to the task's text, and `--help` is
// mise's own flag, so this is the only place it can land.
if (usageWasAsked({
  words,
  usage: USAGE,
},)) {
  process.exit(0,);
}

/**
 The option that names the checkout, as it was written.
 */
const checkoutGiven = optionIn({
  commandLine: words,
  option: CHECKOUT_FOLDER_OPTION,
},);

/**
 The option that names the runs folder, as it was written.
 */
const runsGiven = optionIn({
  commandLine: words,
  option: RUNS_FOLDER_OPTION,
},);

/**
 The option that names the reviews folder, as it was written.
 */
const reviewGiven = optionIn({
  commandLine: words,
  option: REVIEW_FOLDER_OPTION,
},);

/**
 The option that names the process folder, as it was written.
 */
const processGiven = optionIn({
  commandLine: words,
  option: PROCESS_FOLDER_OPTION,
},);

/**
 What this run of the command was asked for.
 */
const options = {
  checkoutFolder: (checkoutGiven === '') ? cwd() : checkoutGiven,
  runsFolder: (runsGiven === '')
    ? join(
      homedir(),
      ...DEFAULT_RUNS_FOLDER_IN_HOME,
    )
    : runsGiven,
  reviewFolder: (reviewGiven === '')
    ? join(
      homedir(),
      ...DEFAULT_REVIEW_FOLDER_IN_HOME,
    )
    : reviewGiven,
  pausedFile: optionIn({
    commandLine: words,
    option: PAUSED_FILE_OPTION,
  },),
  processFolder: (processGiven === '') ? (env.PI_RUNS_PROCESS_FOLDER ?? '/proc') : processGiven,
  apply: words.includes(APPLY_OPTION,),
  cleanFolder: optionIn({
    commandLine: words,
    option: CLEAN_FOLDER_OPTION,
  },),
  cleanWords: words.filter(function namesACleanWord(word,) {
    return word.startsWith(CLEAN_WORD_OPTION,);
  },)
    .map(function wordOf(word,) {
      return word.slice(CLEAN_WORD_OPTION.length,);
    },),
};

/**
 The folder the launcher makes run worktrees under, which is where a run's worktree is looked for.
 */
const worktreesRoot = join(
  options.checkoutFolder,
  '.claude',
  'worktrees',
);

/**
 The names that were paused, which count only while their latest run did not end idle.
 */
const paused = (options.pausedFile === '')
  ? new Set<string>()
  : pausedNamesIn({ shown: await readFileOrNothing(options.pausedFile,), },);

/**
 What the worktree listing answered.
 */
const listed = await runCommandWithChildProcess({
  command: 'git',
  words: [
    'worktree',
    'list',
    '--porcelain',
  ],
  folder: options.checkoutFolder,
},);
if (listed.status !== 0) {
  throw new Error(`git could not list the worktrees of ${options.checkoutFolder}: ${listed.problem
    ?? listed.stderr
      .trim()}`,);
}

/**
 The folder every process of the machine works in, read once.
 */
const workingFolders = (await processesAtTheirFolders({ processFolder: options.processFolder, },))
  .map(function folderOf(process,) {
    return process.folder;
  },);

/**
 The lines to print, one per run's worktree.
 */
const printed: string[] = [];
/**
 The names of the worktrees that were removed.
 */
const removedNames: string[] = [];
/* oxlint-disable eslint/no-await-in-loop -- one worktree after another, each with its own run folders, its own unit and its own
   git question, so a failure names the run it belongs to. See doc/lint/sequential-awaits.md. */
for (const worktree of runWorktreesIn({
  worktrees: worktreesIn({ shown: listed.stdout, },),
  worktreeFolder: worktreesRoot,
},)) {
  /**
   The run's name.
   */
  const name = runNameOfWorktree({
    worktree,
    worktreeFolder: worktreesRoot,
  },);
  /**
   Whether this worktree belongs to a review.
   */
  const isReview = name.startsWith(REVIEW_RUN_START,);
  /**
   Every attempt of this run's name, newest first.
   */
  const attempts = await attemptsOfARun({
    runsFolder: options.runsFolder,
    runName: name,
  },);
  /**
   The newest attempt that wrote a summary, or that there is none.
   */
  const newest = (await Promise.all(attempts.map(async function finished(attempt,) {
    return (await isThere(join(
      options.runsFolder,
      attempt.folder,
      SUMMARY_FILE_NAME,
    ),))
      ? attempt.folder
      : NO_FINISHED_RUN_OF_THAT_NAME;
  },)))
    .find(function isAFolder(folder,) {
      return (typeof folder) === 'string';
    },);
  /**
   The summary of that attempt, when it has one.
   */
  const latestSummary = (typeof newest) === 'string'
    ? join(
      options.runsFolder,
      newest,
      SUMMARY_FILE_NAME,
    )
    : undefined;
  /**
   Why that run ended, or nothing when no summary says.
   */
  const { endReason, } = fieldsOf(JSON.parse((latestSummary === undefined)
    ? '{}'
    : ((await readFileOrNothing(latestSummary,)) || '{}'),),);
  /**
   Whether a brief for this name is waiting for a slot, which keeps its worktree.
   */
  const briefQueued = await briefQueuedAfter({
    runsFolder: options.runsFolder,
    name,
    ...(latestSummary === undefined ? {} : { latestSummary, }),
  },);
  /**
   The folders of this run that must hold no process.
   */
  /**
   Every entry of the runs folder.
   */
  const runEntries = await entriesOrNothingIn(options.runsFolder,);
  /**
   The folders of this run that must hold no process.
   */
  const ownFolders = ownFoldersOf({
    name,
    worktree: worktree.path,
    runsFolder: options.runsFolder,
    entries: runEntries,
    reviewFolder: options.reviewFolder,
  },);
  /**
   The folder a process still works in, if one does.
   */
  const inUse = folderSomeProcessWorksIn({
    folders: ownFolders,
    workingFolders,
  },);
  /**
   What git says about the worktree's files.
   */
  const status = await runCommandWithChildProcess({
    command: 'git',
    words: [
      '--no-optional-locks',
      'status',
      '--porcelain=v1',
      '--untracked-files=normal',
    ],
    folder: worktree.path,
  },);
  /**
   What is to be done with the worktree.
   */
  const decision = decideWorktree({
    ...((typeof endReason) === 'string' ? { endReason, } : {}),
    isReview,
    reviewWroteItsFile: isReview && await isThere(join(
      options.reviewFolder,
      name.slice(REVIEW_RUN_START.length,),
      REVIEW_FILE_NAME,
    ),),
    paused: paused.has(name,),
    briefQueued,
    unitState: await unitStateOf({
      run: runCommandWithChildProcess,
      unitName: `pi-${name}.service`,
    },),
    inUseFolder: inUse.folder ?? '',
    status: status.status,
    statusText: status.stdout,
  },);
  if (decision.action === 'skip') {
    printed.push(`${name}: left alone, ${decision.reason}`,);
    continue;
  }
  if (!options.apply) {
    printed.push(`${name}: would remove the worktree (branch ${(worktree.branch === '') ? 'detached' : worktree.branch} kept)`,);
    continue;
  }
  /**
   What the clean command freed, when the caller named one.
   */
  /**
   The words of the clean command the caller named.
   */
  const { cleanWords, } = options;
  /**
   Whether the caller named a clean command at all.
   */
  const hasCleanCommand = !((options.cleanFolder === '')
    || (cleanWords
      .length === 0));
  /**
   What the clean command answered, when the caller named one.
   */
  const cleaned = (!hasCleanCommand)
    ? undefined
    // oxlint-disable-next-line eslint/no-await-in-loop -- one clean command per worktree, in the folder the caller named inside it; see doc/lint/sequential-awaits.md
    : await runCommandShowingItsOutput({
      command: cleanWords[0] ?? '',
      words: cleanWords.slice(1,),
      folder: join(
        worktree.path,
        options.cleanFolder,
      ),
    },);
  /**
   What the clean command freed, as the line says it.
   */
  const freed = (cleaned === undefined) ? 'no clean command' : `clean ended with status ${String(cleaned.status,)}`;
  /**
   What git answered about removing the worktree.
   */
  const removal = await runCommandWithChildProcess({
    command: 'git',
    words: [
      'worktree',
      'remove',
      worktree.path,
    ],
    folder: options.checkoutFolder,
  },);
  if (removal.status === 0) {
    removedNames.push(name,);
  }
  printed.push(`${name}: ${freed}; worktree remove status ${String(removal.status,)}${(removal.status === 0) ? '' : `: ${removal.stderr
    .trim()
    .split('\n',)
    .at(-1,)
    ?? ''}`}`,);
}
/* oxlint-enable eslint/no-await-in-loop */
/**
 How many worktrees were removed.
 */
const removed = String(removedNames.length,);
printed.push(options.apply
  ? `removed ${removed} worktrees`
  : 'nothing was removed: pass --apply to remove the worktrees listed',);
stdout.write(`${printed.join('\n',)}\n`,);
