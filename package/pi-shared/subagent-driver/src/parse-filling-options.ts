/**
 Reads the command line of `fill-free-places-from-a-queue`: which queue, the level the agents going are counted against, and how
 long and how often the filling goes round.

 @module
 */

import { optionIn, } from './option-in.ts';
import {
  type QueueOptionWords,
  queueFileIn,
  reviewsFolderIn,
  runsFolderIn,
  wholeNumberFromOne,
} from './parse-queue-of-runs-options.ts';
import { requiredText, } from './required-option-values.ts';
import { FILLING_STARTS_A_ROUND, } from './filling-free-places-from-a-queue.ts';
import { FILLING_ROUND_SECONDS, } from './stalled-filling-notice.ts';
import { MILLISECONDS_PER_SECOND, } from './driver-constants.ts';
import { BATCH_DEFAULT_DEADLINE_SECONDS, } from './batch-list.ts';

/**
 The filling's command line, for the error message.
 */
export const FILLING_USAGE: string = 'fill-free-places-from-a-queue --queue=<file> --level=<n> [--runs-folder=<folder>] '
  + '[--reviews-folder=<folder>] [--checkout-folder=<folder>] [--starts-a-round=<n>] '
  + '[--seconds=<n>] [--round-seconds=<n>] [--deadline-seconds=<n>] [--once] [--without-putting-ended-reviews-back]';

/**
 The option that names the level: the most agents that may be going at once.
 */
const LEVEL_OPTION = '--level=';

/**
 The option that names the checkout the entries' reviews are read in.
 */
const CHECKOUT_FOLDER_OPTION = '--checkout-folder=';

/**
 The option that names how many entries one round starts at most.
 */
const STARTS_A_ROUND_OPTION = '--starts-a-round=';

/**
 The option that names how long the filling keeps going.
 */
const SECONDS_OPTION = '--seconds=';

/**
 The option that names the wait between two rounds.
 */
const ROUND_SECONDS_OPTION = '--round-seconds=';

/**
 The option that names how long one entry's batch keeps asking for a slot.
 */
const DEADLINE_SECONDS_OPTION = '--deadline-seconds=';

/**
 The word that asks for one round only.
 */
const ONCE_WORD = '--once';

/**
 The word that leaves the rate-limited reviews where they are, for a caller that puts them back itself.
 */
const WITHOUT_PUTTING_ENDED_REVIEWS_BACK_WORD = '--without-putting-ended-reviews-back';

/**
 A command line the filling cannot use.
 */
export class FillingOptionsError extends Error {
  /**
   The name printed with the message.
   */
  override readonly name = 'FillingOptionsError';
}

/**
 Seconds a filling keeps going when the caller names none: the eight hours the coordinator's own filling unit was started for, so
 the two are the same run of work.
 */
export const DEFAULT_FILLING_SECONDS = 28_800;

/**
 Builds the error this command answers a refused option with.

 @param message - what is wrong with the command line

 @returns the error the command exits with

 @example
 ```ts
 throw fillingOptionsErrorOf('--queue is required.',);
 ```
 */
function fillingOptionsErrorOf(message: string,): Error {
  return new FillingOptionsError(message,);
}

/**
 The words the filling's own options are read with.

 @returns the usage line and the error it refuses with

 @example
 ```ts
 const words = fillingWords();
 ```
 */
function fillingWords(): QueueOptionWords {
  return {
    usage: FILLING_USAGE,
    errorOf: fillingOptionsErrorOf,
  };
}

/**
 What the filling was asked for.
 */
export type FillingOptions = {
  /**
   The queue it fills the free places from.
   */
  readonly queueFile: string;

  /**
   The most agents that may be going at once.
   */
  readonly level: number;

  /**
   Folder the runs live in.
   */
  readonly runsFolder: string;

  /**
   Folder that holds one folder per review.
   */
  readonly reviewsFolder: string;

  /**
   The checkout the entries' reviews are read in.
   */
  readonly checkoutFolder: string;

  /**
   How many entries one round starts at most.
   */
  readonly startsAtMost: number;

  /**
   How long the filling keeps going, in seconds.
   */
  readonly secondsAtMost: number;

  /**
   How long it waits between two rounds, in milliseconds.
   */
  readonly roundMilliseconds: number;

  /**
   How long one entry's batch keeps asking for a slot, in seconds.
   */
  readonly deadlineSeconds: number;

  /**
   Whether it stops after one round.
   */
  readonly once: boolean;

  /**
   Whether it puts the reviews the rate limit ended back at the head of the queue each round.
   */
  readonly putBackEndedReviews: boolean;
};

/**
 Reads the command line.

 @param commandLine - the words the command was given

 @returns what the filling was asked for

 @throws FillingOptionsError when the queue or the level is missing, or a number cannot be read

 @example
 ```ts
 const options = parseFillingOptions({ commandLine: ['--queue=/runs/q.json', '--level=16'], });
 ```
 */
export function parseFillingOptions({ commandLine, }: { readonly commandLine: readonly string[] },): FillingOptions {
  /**
   The words the options are read with.
   */
  const words = fillingWords();
  /**
   The queue to fill from.
   */
  const queueFile = queueFileIn({
    commandLine,
    words,
  },);
  /**
   The level, as the caller wrote it.
   */
  const level = requiredText({
    value: optionIn({
      commandLine,
      option: LEVEL_OPTION,
    },),
    name: 'level',
    usage: FILLING_USAGE,
    errorOf: words.errorOf,
  },);
  /**
   The level as a whole number.
   */
  const levelOf = Number(level,);
  if ((!Number.isInteger(levelOf,)) || (levelOf < 1)) {
    throw new FillingOptionsError(`--level must be a whole number from 1, got ${level}.\nUsage: ${FILLING_USAGE}`,);
  }
  /**
   The checkout the reviews are read in.
   */
  const checkoutFolder = optionIn({
    commandLine,
    option: CHECKOUT_FOLDER_OPTION,
  },);
  /**
   How long the filling keeps going, as the caller wrote it.
   */
  const seconds = optionIn({
    commandLine,
    option: SECONDS_OPTION,
  },);
  /**
   How long it waits between rounds, as the caller wrote it.
   */
  const roundSeconds = optionIn({
    commandLine,
    option: ROUND_SECONDS_OPTION,
  },);
  /**
   How long one batch keeps asking for a slot, as the caller wrote it.
   */
  const deadlineSeconds = optionIn({
    commandLine,
    option: DEADLINE_SECONDS_OPTION,
  },);
  return {
    queueFile,
    level: levelOf,
    runsFolder: runsFolderIn({ commandLine, },),
    reviewsFolder: reviewsFolderIn({ commandLine, },),
    checkoutFolder: (checkoutFolder === '') ? process.cwd() : checkoutFolder,
    startsAtMost: wholeNumberFromOne({
      commandLine,
      option: STARTS_A_ROUND_OPTION,
      words,
    },)
      .value
      ?? FILLING_STARTS_A_ROUND,
    secondsAtMost: (seconds === '')
      ? DEFAULT_FILLING_SECONDS
      : requiredWholeNumberFromOne({
        written: seconds,
        name: 'seconds',
        words,
      },),
    roundMilliseconds: ((roundSeconds === '')
      ? FILLING_ROUND_SECONDS
      : requiredWholeNumberFromOne({
        written: roundSeconds,
        name: 'round-seconds',
        words,
      },)) * MILLISECONDS_PER_SECOND,
    deadlineSeconds: (deadlineSeconds === '')
      ? BATCH_DEFAULT_DEADLINE_SECONDS
      : requiredWholeNumberFromOne({
        written: deadlineSeconds,
        name: 'deadline-seconds',
        words,
      },),
    once: commandLine.includes(ONCE_WORD,),
    putBackEndedReviews: !commandLine.includes(WITHOUT_PUTTING_ENDED_REVIEWS_BACK_WORD,),
  };
}

/**
 Reads an option that must be a whole number from one.

 @param written - the value as the caller wrote it

 @param name - the option's name, without its dashes

 @param words - the command's usage line and the error it refuses with

 @returns the number

 @throws the caller's error, when the value is no whole number from one

 @example
 ```ts
 const seconds = requiredWholeNumberFromOne({ written: '28800', name: 'seconds', words });
 ```
 */
function requiredWholeNumberFromOne({
  written,
  name,
  words,
}: {
  readonly written: string;
  readonly name: string;
  readonly words: QueueOptionWords;
},): number {
  /**
   The value as a number.
   */
  const value = Number(written,);
  if ((!Number.isInteger(value,)) || (value < 1)) {
    throw words.errorOf(`--${name} must be a whole number from 1, got ${written}.\nUsage: ${words.usage}`,);
  }
  return value;
}
