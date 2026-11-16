#!/usr/bin/env node
/**
 `put-ended-reviews-back-at-the-head-of-a-queue`: puts every review run that the reviewer's provider ended on its rate limit back
 at the head of the queue, as an entry that goes on from the run's own session, so no session has to notice the ending first.

 See `README.md` for what it refuses to put back. The filling calls this once a round.

 @module
 */

import {
  argv,
  env,
} from 'node:process';
import { caughtValueText, } from '@monochromatic-dev/caught-value-core/ts';
import {
  goingRunsNow,
  LAUNCHER_ENTRY_NAMES,
  REPLACED_LAUNCHER_ENTRY_NAMES,
} from './going-runs.ts';
import {
  limitsIn,
  POOL_LIMITS_VARIABLE,
  poolsWithTheseLimits,
} from './pool-limits.ts';
import {
  type QueueOptionWords,
  queueFileIn,
  reviewsFolderIn,
  runsFolderIn,
} from './parse-queue-of-runs-options.ts';
import {
  putBackSince,
  putEndedReviewsBack,
} from './putting-ended-reviews-back.ts';
import { reportOnStandardError, } from './kept-start-failure.ts';
import { runCommandWithChildProcess, } from './run-command.ts';
import { usageWasAsked, } from './usage-of-a-command.ts';

/**
 The command's command line, for the error message.
 */
const USAGE = 'put-ended-reviews-back-at-the-head-of-a-queue --queue=<file> [--runs-folder=<folder>] '
  + '[--reviews-folder=<folder>] [--since=<ISO 8601>]';

/**
 The option that names the earliest end a run is put back from.
 */
const SINCE_OPTION = '--since=';

/**
 The folder the process table is read from, which is where a Linux machine keeps it.
 */
const PROCESS_FOLDER = env.PI_RUNS_PROCESS_FOLDER ?? '/proc';

/**
 A command line the command cannot use.
 */
class PutEndedReviewsBackOptionsError extends Error {
  /**
   The name printed with the message.
   */
  override readonly name = 'PutEndedReviewsBackOptionsError';
}

/**
 Builds the error this command answers a refused option with.

 @param message - what is wrong with the command line

 @returns the error the command exits with

 @example
 ```ts
 throw putEndedReviewsBackOptionsErrorOf('--queue is required.',);
 ```
 */
function putEndedReviewsBackOptionsErrorOf(message: string,): Error {
  return new PutEndedReviewsBackOptionsError(message,);
}

/**
 The words this command's options are read with.

 @returns the usage line and the error it refuses with

 @example
 ```ts
 const words = putBackWords();
 ```
 */
function putBackWords(): QueueOptionWords {
  return {
    usage: USAGE,
    errorOf: putEndedReviewsBackOptionsErrorOf,
  };
}

// The word that asks for help goes to the command as an argument, and the command answers it before it reads an option
// or touches anything: mise appends a word its task's usage spec does not claim to the task's text, and `--help` is
// mise's own flag, so this is the only place it can land.
if (usageWasAsked({
  words: argv.slice(2,),
  usage: USAGE,
},)) {
  process.exitCode = 0;
  process.exit(0,);
}

/**
 The words the command was given.
 */
const commandLine = argv.slice(2,);

/**
 The words the options are read with.
 */
const words = putBackWords();

/**
 Reads the earliest end a run is put back from.

 @returns the moment the caller named, or the lookback before now when it named none

 @throws the error the command refuses with, when the moment is no ISO 8601 time
 */
function sinceIn(): number {
  /**
   The moment as the caller wrote it.
   */
  const written = commandLine.find(function namesTheMoment(word,) {
    return word.startsWith(SINCE_OPTION,);
  },)
    ?.slice(SINCE_OPTION.length,)
    ?? '';
  if (written === '') {
    return putBackSince({ now: Date.now(), },);
  }
  /**
   The moment as milliseconds since the epoch.
   */
  const since = Date.parse(written,);
  if (Number.isNaN(since,)) {
    throw words.errorOf(`--since must be an ISO 8601 time, got ${written}.\nUsage: ${USAGE}`,);
  }
  return since;
}

try {
  /**
   The runs going now, by name, which a review whose unit is up is not put back for.
   */
  const going = await goingRunsNow({
    run: runCommandWithChildProcess,
    processFolder: PROCESS_FOLDER,
    launcherEntryNames: [
      ...LAUNCHER_ENTRY_NAMES,
      ...REPLACED_LAUNCHER_ENTRY_NAMES,
    ],
    pools: poolsWithTheseLimits({ limits: limitsIn({ listed: env[POOL_LIMITS_VARIABLE] ?? '', },), },),
    onProblem: reportOnStandardError,
  },);
  /**
   What putting the ended reviews back answered.
   */
  const putBack = await putEndedReviewsBack({
    queueFile: queueFileIn({
      commandLine,
      words,
    },),
    runsFolder: runsFolderIn({ commandLine, },),
    reviewsFolder: reviewsFolderIn({ commandLine, },),
    since: sinceIn(),
    going: new Set(going.map(function nameOf(run,) {
      return run.name;
    },),),
    now: Date.now,
  },);
  for (const line of putBack.lines) {
    process.stdout
      .write(`${line}\n`,);
  }
  process.stdout
    .write(`${String(putBack.putBack
      .length,)} ${putBack.putBack
        .length
        === 1 ? 'review' : 'reviews'} put back\n`,);
}
catch (error) {
  process.stderr
    .write(`${caughtValueText(error,)}\n`,);
  process.exitCode = 1;
}
