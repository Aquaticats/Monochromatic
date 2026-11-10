#!/usr/bin/env node
/**
 `start-kimi-reviews`: starts a round of Kimi K3.1 Pro reviews from a list, each in its own worktree at the commit under review,
 with the range's diff and every changed file named in its brief.

 A review runs on Synthetic, outside the Neuralwatt slots, and a review the provider rate-limited is started again on
 `neuralwatt/kimi-k3.1-pro-flex` with the same session.

 Usage: `start-kimi-reviews --list=<reviews.json> [--runs-folder=<folder>] [--repository=<folder>] [--reviews-folder=<folder>]
 [--wall-seconds=<n>] [--fallback-wall-seconds=<n>] [--deadline-seconds=<n>] [--period-seconds=<n>]`

 @module
 */

import {
  readFile,
} from 'node:fs/promises';
import {
  extname,
  isAbsolute,
  join,
} from 'node:path';
import process, {
  argv,
  stdout,
} from 'node:process';
import { setTimeout as wait, } from 'node:timers/promises';
import { LONGEST_CHILD_MILLISECONDS, } from './longest-child-bound.ts';
import { errorCodeOf, } from './error-code-of.ts';
import { parseKimiReviewList, } from './kimi-reviews.ts';
import {
  defaultKimiReviewsFolder,
  defaultRepository,
  defaultRunsFolder,
} from './pi-runs-folders.ts';
import { runCommandWithChildProcess, } from './run-command.ts';
import {
  NO_SUMMARY,
  type PreparedReview,
  prepareKimiReview,
} from './prepare-a-kimi-review.ts';
import {
  type AttemptEnding,
  startARoundOfKimiReviews,
} from './round-of-kimi-reviews.ts';

/**
 The queue command a review is started through, beside this one.
 */
const RETRY_COMMAND_NAME = 'retry-start-pi-run-until-a-slot-frees';

/**
 How long a review on Synthetic may last, which the coordinator's reviews were given.
 */
const REVIEW_WALL_SECONDS = 5_400;

/**
 How long the attempt on Neuralwatt's Kimi pool may last. The owner measured that pool at around 100% slower, so an attempt
 there gets twice what the Synthetic attempt got.
 */
const FALLBACK_WALL_SECONDS = 10_800;

/**
 How long a review's queue may wait for a slot before it gives up.
 */
const REVIEW_DEADLINE_SECONDS = 10_800;

/**
 How often a review's queue asks whether a slot freed, which is shorter than a batch's because a review outside the pools is
 refused only while its own provider is busy.
 */
const REVIEW_PERIOD_SECONDS = 13;

/**
 How far apart two reviews of a round are started, so their worktrees and units do not all begin at once.
 */
const ROUND_STAGGER_MILLISECONDS = 20_000;

/**
 What a review's unit is given of the machine's cores, which is what the coordinator's reviews were given.
 */
const REVIEW_CPU_PERCENT = 800;

/**
 What a review's unit is given of the machine's memory.
 */
const REVIEW_MEMORY = '12G';

/**
 How many tasks a review's unit may start.
 */
const REVIEW_TASKS = 3_000;

/**
 What a start's output is cut to, which is what a person reads of it.
 */
const OUTPUT_KEPT_CHARACTERS = 3_000;

/**
 The command's command line, for the error message.
 */
const USAGE =
  'start-kimi-reviews --list=<reviews.json> [--runs-folder=<folder>] [--repository=<folder>] [--reviews-folder=<folder>] [--wall-seconds=<n>] [--fallback-wall-seconds=<n>] [--deadline-seconds=<n>] [--period-seconds=<n>]';

/**
 Reads the value of one option of this command.

 @param name - the option's name, without the leading dashes

 @returns the value, or nothing when the option was not given

 @example
 ```ts
 const list = optionOf('list',);
 ```
 */
function optionOf(name: string,): string | typeof NOT_GIVEN {
  /**
   The option as it was written.
   */
  const written = argv.find(function startsWithTheOption(argument,) {
    return argument.startsWith(`--${name}=`,);
  },);
  return (written === undefined) ? NOT_GIVEN : written.slice(name.length + OPTION_PREFIX_CHARACTERS,);
}

/**
 How many characters one option's prefix takes, which is the dashes, the name and the equals sign.
 */
const OPTION_PREFIX_CHARACTERS = 3;

/**
 An option that was not given, as a symbol so no value can be mistaken for it.
 */
const NOT_GIVEN: unique symbol = Symbol('the option was not given',);

/**
 Reads one option that must be given, refusing an empty one.

 @param value - the option's value as this command read it

 @param name - the option's name, without the leading dashes

 @param fallback - what to use when the option was not given

 @returns the value

 @example
 ```ts
 const folder = givenOptionOf({ value: NOT_GIVEN, name: 'repository', fallback: '/repo', });
 ```
 */
function givenOptionOf({
  value,
  name,
  fallback,
}: {
  readonly value: string | typeof NOT_GIVEN;
  readonly name: string;
  readonly fallback: string;
},): string {
  if ((typeof value) === 'symbol') {
    return fallback;
  }
  if (value === '') {
    throw new Error(`--${name} needs a value\nUsage: ${USAGE}`,);
  }
  return value;
}

/**
 Reads one option as a number of seconds.

 @param name - the option's name, without the leading dashes

 @param fallback - what to use when the option was not given

 @returns the number of seconds

 @example
 ```ts
 const wall = secondsOptionOf({ name: 'wall-seconds', fallback: 5_400, });
 ```
 */
function secondsOptionOf({
  name,
  fallback,
}: {
  readonly name: string;
  readonly fallback: number;
},): number {
  /**
   The option as this command read it.
   */
  const value = optionOf(name,);
  if ((typeof value) === 'symbol') {
    return fallback;
  }
  /**
   The number it holds.
   */
  const seconds = Number(value,);
  if ((!Number.isFinite(seconds,)) || (seconds <= 0)) {
    throw new Error(`--${name} must be a number above zero; it was "${value}".\nUsage: ${USAGE}`,);
  }
  return seconds;
}

/**
 Folder the driver writes one run folder per run into.
 */
const runsFolder = givenOptionOf({
  value: optionOf('runs-folder',),
  name: 'runs-folder',
  fallback: defaultRunsFolder({},),
},);

/**
 The checkout that holds the worktrees folder and the code under review.
 */
const repository = givenOptionOf({
  value: optionOf('repository',),
  name: 'repository',
  fallback: defaultRepository({},),
},);

/**
 Where every review writes its diff, its reproductions and its answer.
 */
const reviewsFolder = givenOptionOf({
  value: optionOf('reviews-folder',),
  name: 'reviews-folder',
  fallback: defaultKimiReviewsFolder({},),
},);

/**
 The reviews to start.
 */
const listPath = optionOf('list',);

if ((typeof listPath) === 'symbol') {
  throw new Error(`--list names the reviews to start\nUsage: ${USAGE}`,);
}

/**
 The reviews to start.
 */
const reviews = parseKimiReviewList({
  text: await readFile(
    isAbsolute(listPath,) ? listPath : join(
      runsFolder,
      listPath,
    ),
    { encoding: 'utf8', },
  ),
},);

/**
 The queue command, beside this one.
 */
const retryScript = join(
  import.meta.dirname,
  `${RETRY_COMMAND_NAME}${extname(import.meta.filename,)}`,
);

/**
 Starts one review through its queue, which waits for a slot outside the pools.

 @param prepared - the review to start

 @param provider - what the attempt runs on

 @param model - what the attempt runs on

 @param run - the number of this attempt

 @param wallSeconds - how long this attempt may last

 @param session - the session file this attempt continues, when it continues one

 @returns what the queue answered

 @example
 ```ts
 const ending = await startAReviewAttempt({ prepared, provider: KIMI_REVIEW_PROVIDER, model: KIMI_REVIEW_MODEL, run: 1, wallSeconds: 5_400, });
 ```
 */
async function startAReviewAttempt({
  prepared,
  provider,
  model,
  run,
  wallSeconds,
  session,
}: {
  readonly prepared: PreparedReview;
  readonly provider: string;
  readonly model: string;
  readonly run: number;
  readonly wallSeconds: number;
  readonly session?: string;
},): Promise<AttemptEnding> {
  /**
   What the queue answered.
   */
  const answered = await runCommandWithChildProcess({
    command: process.execPath,
    words: [
      retryScript,
      `--deadline-seconds=${String(secondsOptionOf({
        name: 'deadline-seconds',
        fallback: REVIEW_DEADLINE_SECONDS,
      },),)}`,
      `--period-seconds=${String(secondsOptionOf({
        name: 'period-seconds',
        fallback: REVIEW_PERIOD_SECONDS,
      },),)}`,
      '--',
      `--name=${prepared.runName}`,
      `--branch=${prepared.branch}`,
      `--prompt=${prepared.promptFile}`,
      `--provider=${provider}`,
      `--model=${model}`,
      `--cpu=${String(REVIEW_CPU_PERCENT,)}`,
      `--memory=${REVIEW_MEMORY}`,
      `--tasks=${String(REVIEW_TASKS,)}`,
      `--wall=${String(wallSeconds,)}`,
      `--runs-folder=${runsFolder}`,
      `--repository=${repository}`,
      `--run=${String(run,)}`,
      ...((session === undefined) ? [] : [`--session=${session}`,]),
    ],
    timeoutMilliseconds: LONGEST_CHILD_MILLISECONDS,
  },);
  return {
    status: answered.status,
    output: `${answered.stdout}${answered.stderr}`.slice(-OUTPUT_KEPT_CHARACTERS,),
  };
}

/**
 Reads what one attempt left last.

 @param runFolder - the folder the attempt wrote into

 @returns the summary's text, or {@link NO_SUMMARY}

 @example
 ```ts
 const summary = await summaryTextOf({ runFolder: '/runs/kimi-review-a-1', });
 ```
 */
async function summaryTextOf({
  runFolder,
}: {
  readonly runFolder: string;
},): Promise<string | typeof NO_SUMMARY> {
  try {
    return await readFile(
      join(
        runFolder,
        'summary.json',
      ),
      { encoding: 'utf8', },
    );
  }
  catch (error) {
    // An attempt whose unit never started left no summary, which says nothing about a rate limit.
    if (errorCodeOf(error,) === 'ENOENT') {
      return NO_SUMMARY;
    }
    throw error;
  }
}

/**
 One block of lines per review.
 */
const lines = await startARoundOfKimiReviews({
  reviews,
  prepare: async function prepareReview(review,) {
    return await prepareKimiReview({
      review,
      repository,
      runsFolder,
      reviewsFolder,
      run: runCommandWithChildProcess,
    },);
  },
  startAttempt: startAReviewAttempt,
  summaryTextOf,
  wait: async function passTime(milliseconds,) {
    await wait(milliseconds,);
  },
  staggerMilliseconds: ROUND_STAGGER_MILLISECONDS,
  runsFolder,
  wallSeconds: secondsOptionOf({
    name: 'wall-seconds',
    fallback: REVIEW_WALL_SECONDS,
  },),
  fallbackWallSeconds: secondsOptionOf({
    name: 'fallback-wall-seconds',
    fallback: FALLBACK_WALL_SECONDS,
  },),
},);

stdout.write(`${lines.join('\n',)}\n`,);
