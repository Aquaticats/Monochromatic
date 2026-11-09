/**
 Reads the command line of `watch-overdue-pi-runs`. The runs folder and the deadline are required, so no look is ever taken
 without knowing where the runs are or how long to keep looking.

 @module
 */

import { parseArgs, } from 'node:util';
import { MILLISECONDS_PER_SECOND, } from './driver-constants.ts';
import {
  LOOK_INTERVAL_MILLISECONDS,
  RECHECK_MILLISECONDS,
} from './overdue-constants.ts';
import {
  type RequiredOption,
  requiredPositiveNumber,
  requiredSeconds,
  requiredText,
} from './required-option-values.ts';

/**
 The command's command line, for the error message.
 */
export const WATCH_USAGE =
  'watch-overdue-pi-runs --runs-folder=<folder> --deadline-seconds=<n> [--interval-seconds=<n>] [--recheck-seconds=<n>] [--limit-minutes=<n>] [--state-file=<file>] [--once]';

/**
 Milliseconds between looks when the caller names no interval.
 */
const DEFAULT_INTERVAL_SECONDS: number = LOOK_INTERVAL_MILLISECONDS / MILLISECONDS_PER_SECOND;

/**
 Milliseconds a reported run is left alone when the caller names no recheck interval.
 */
const DEFAULT_RECHECK_SECONDS: number = RECHECK_MILLISECONDS / MILLISECONDS_PER_SECOND;

/**
 A command line the watch cannot use.
 */
export class WatchOptionsError extends Error {
  /**
   The name printed with the message.
   */
  override readonly name = 'WatchOptionsError';
}

/**
 What one run of the command was asked for.
 */
export type WatchOptions = {
  /**
   Folder the driver writes one run folder per run into.
   */
  readonly runsFolder: string;

  /**
   How long the command may look for a run that is past a limit.
   */
  readonly deadlineSeconds: number;

  /**
   How long to wait between looks.
   */
  readonly intervalSeconds: number;

  /**
   How long a reported run is left alone before it is reported again.
   */
  readonly recheckSeconds: number;

  /**
   A limit in minutes that replaces both computed limits; absent for the limits the finished runs give.
   */
  readonly limitMinutes?: number;

  /**
   The file that keeps what was reported; absent for the user's state folder.
   */
  readonly stateFile?: string;

  /**
   Whether to look once and end.
   */
  readonly once: boolean;
};

/**
 Builds the error this command answers a refused option with.

 @param message - what is wrong with the command line

 @returns the error the command exits with

 @example
 ```ts
 throw watchOptionsErrorOf('--runs-folder is required.',);
 ```
 */
function watchOptionsErrorOf(message: string,): Error {
  return new WatchOptionsError(message,);
}

/**
 Describes one option for the readers that refuse it.

 @param value - the option's value as the command line parser read it

 @param name - the option's name, without the leading dashes

 @returns what a reader of an option needs to refuse it

 @example
 ```ts
 const option = optionOf({
          value: values['runs-folder'],
          name: 'runs-folder',
        },);
 ```
 */
function optionOf({
  value,
  name,
}: {
  readonly value: unknown;
  readonly name: string;
},): RequiredOption {
  return {
    value,
    name,
    usage: WATCH_USAGE,
    errorOf: watchOptionsErrorOf,
  };
}

/**
 Reads the command line.

 @param commandLine - arguments after the program name

 @returns what the run was asked for

 @throws WatchOptionsError when an option is unknown, missing or unusable

 @example
 ```ts
 const options = parseWatchOptions(argv.slice(2,),);
 ```
 */
export function parseWatchOptions(commandLine: readonly string[],): WatchOptions {
  try {
    /**
     The options as written.
     */
    const { values, } = parseArgs({
      args: [...commandLine,],
      strict: true,
      allowPositionals: false,
      options: {
        'runs-folder': { type: 'string', },
        'deadline-seconds': { type: 'string', },
        'interval-seconds': { type: 'string', },
        'recheck-seconds': { type: 'string', },
        'limit-minutes': { type: 'string', },
        'state-file': { type: 'string', },
        once: { type: 'boolean', },
      },
    },);
    return {
      runsFolder: requiredText(optionOf({
        value: values['runs-folder'],
        name: 'runs-folder',
      },),),
      deadlineSeconds: requiredSeconds(optionOf({
        value: values['deadline-seconds'],
        name: 'deadline-seconds',
      },),),
      intervalSeconds: (values['interval-seconds'] === undefined)
        ? DEFAULT_INTERVAL_SECONDS
        : requiredSeconds(optionOf({
          value: values['interval-seconds'],
          name: 'interval-seconds',
        },),),
      recheckSeconds: (values['recheck-seconds'] === undefined)
        ? DEFAULT_RECHECK_SECONDS
        : requiredSeconds(optionOf({
          value: values['recheck-seconds'],
          name: 'recheck-seconds',
        },),),
      ...(values['limit-minutes'] === undefined ? {} : { limitMinutes: requiredPositiveNumber(optionOf({
        value: values['limit-minutes'],
        name: 'limit-minutes',
      },),), }),
      ...(values['state-file'] === undefined ? {} : { stateFile: requiredText(optionOf({
        value: values['state-file'],
        name: 'state-file',
      },),), }),
      once: values.once ?? false,
    };
  }
  catch (error) {
    if (error instanceof WatchOptionsError) {
      throw error;
    }
    throw new WatchOptionsError(
      `${String(error,)}\nUsage: ${WATCH_USAGE}`,
      { cause: error, },
    );
  }
}
