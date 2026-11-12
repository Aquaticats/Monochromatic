/**
 Reads the command line of the pool gate. The options it reads are the ones it has to act on; every other word is the launcher's
 and is passed on untouched, in the order it was written, so the two commands keep one vocabulary rather than two.

 @module
 */

import { homedir, } from 'node:os';
import { join, } from 'node:path';
import { optionIn, } from './option-in.ts';
import {
  requiredSeconds,
  requiredText,
} from './required-option-values.ts';

/**
 The gate's command line, as the pool gate reads it: the launcher's own options, plus the ones only this command reads.
 */
export const POOL_GATE_USAGE: string =
  'start-pi-run-when-a-pool-has-room --name=<run name> [--provider=<name>] [--model=<name>] [--wall=<seconds>] '
    + '[--session-of=<run folder>] [--runs-folder=<folder>] [--counts-only] [<worktree launcher options>]';

/**
 The option that names the run.
 */
export const NAME_OPTION = '--name=';

/**
 The option that asks for the counts and no start.
 */
export const COUNTS_ONLY_OPTION = '--counts-only';

/**
 The option that names the folder the runs live in.
 */
export const RUNS_FOLDER_OPTION = '--runs-folder=';

/**
 The option that names the provider a run asks for.
 */
export const PROVIDER_OPTION = '--provider=';

/**
 The option that names the model a run asks for.
 */
export const MODEL_OPTION = '--model=';

/**
 The option that names how long a run may last, in seconds.
 */
export const WALL_OPTION = '--wall=';

/**
 The option that names a finished run whose session this run resumes.
 */
export const SESSION_OF_OPTION = '--session-of=';

/**
 The parts of the coordinator's runs folder below the home directory of whoever runs this.
 */
const DEFAULT_RUNS_FOLDER_IN_HOME: readonly string[] = [
  'temp',
  'agent',
  'pi-runs',
];

/**
 The value an option has when the line does not carry it.
 */
const ABSENT = '';

/**
 A command line the pool gate cannot use.
 */
export class PoolGateOptionsError extends Error {
  /**
   The name printed with the message.
   */
  override readonly name = 'PoolGateOptionsError';
}

/**
 Builds the error this command answers a refused option with.

 @param message - what is wrong with the command line

 @returns the error the command exits with

 @example
 ```ts
 throw poolGateOptionsErrorOf('--name is required.',);
 ```
 */
function poolGateOptionsErrorOf(message: string,): Error {
  return new PoolGateOptionsError(message,);
}

/**
 What the pool gate was asked for.
 */
export type PoolGateOptions = {
  /**
   The run's name, which is also the start of its run folder's name. Absent when the caller asked only for the counts, which
   name no run.
   */
  readonly name?: string;

  /**
   Folder the runs live in.
   */
  readonly runsFolder: string;

  /**
   Whether to print the counts and start nothing.
   */
  readonly countsOnly: boolean;

  /**
   The provider the run asks for, which the gate may replace with another pool's.
   */
  readonly provider?: string;

  /**
   The model the run asks for, which the gate may replace with another pool's.
   */
  readonly model?: string;

  /**
   How long the run may last in seconds, which is doubled for a pool that answers more slowly.
   */
  readonly wallSeconds?: number;

  /**
   The finished run whose session this run resumes, which stays on the pool it named.
   */
  readonly sessionOfRunFolder?: string;

  /**
   Every word of the command line, in order, which the gate hands the launcher with its own changes applied.
   */
  readonly commandLine: readonly string[];
};

/**
 Reads the folder the runs live in, which is the coordinator's own under the home directory unless the caller names one.

 @param given - the folder the caller named, empty when it named none

 @returns the folder

 @example
 ```ts
 const folder = runsFolderOf('',);
 ```
 */
function runsFolderOf(given: string,): string {
  return (given === ABSENT)
    ? join(
      homedir(),
      ...DEFAULT_RUNS_FOLDER_IN_HOME,
    )
    : given;
}

/**
 Reads an option's text, kept apart from the fields it fills, so that an option the caller left out stays absent rather than
 becoming an empty value.

 @param given - the option's text, empty when the line does not carry it

 @returns the option's field, or nothing at all when it was not given

 @example
 ```ts
 const provider = givenOption('neuralwatt',);
 ```
 */
function givenOption(given: string,): { readonly given?: string } {
  return (given === ABSENT) ? {} : { given, };
}

/**
 Reads the command line.

 @param commandLine - arguments after the program name

 @returns what the gate was asked for

 @throws PoolGateOptionsError when the run's name is missing or empty, or a limit cannot be read

 @example
 ```ts
 const options = parsePoolGateOptions(['--name=copy-route',],);
 ```
 */
export function parsePoolGateOptions(commandLine: readonly string[],): PoolGateOptions {
  /**
   Whether the caller asked only for the counts and no start.
   */
  const countsOnly = commandLine.includes(COUNTS_ONLY_OPTION,);
  /**
   The run's name, which a caller asking only for the counts need not name: the counts are the same whoever asks.
   */
  const name = countsOnly
    ? givenOption(optionIn({
      commandLine,
      option: NAME_OPTION,
    },),)
    : {
      given: requiredText({
        value: optionIn({
          commandLine,
          option: NAME_OPTION,
        },),
        name: 'name',
        usage: POOL_GATE_USAGE,
        errorOf: poolGateOptionsErrorOf,
      },),
    };
  /**
   The folder the runs live in.
   */
  const runsFolder = runsFolderOf(optionIn({
    commandLine,
    option: RUNS_FOLDER_OPTION,
  },),);
  /**
   The provider the run asks for.
   */
  const provider = givenOption(optionIn({
    commandLine,
    option: PROVIDER_OPTION,
  },),);
  /**
   The model the run asks for.
   */
  const model = givenOption(optionIn({
    commandLine,
    option: MODEL_OPTION,
  },),);
  /**
   The wall clock the run asks for, in seconds.
   */
  const wall = optionIn({
    commandLine,
    option: WALL_OPTION,
  },);
  /**
   The finished run this run resumes, when the caller named one.
   */
  const sessionOf = givenOption(optionIn({
    commandLine,
    option: SESSION_OF_OPTION,
  },),);
  return {
    ...(name.given === undefined ? {} : { name: name.given, }),
    runsFolder,
    countsOnly,
    ...(provider.given === undefined ? {} : { provider: provider.given, }),
    ...(model.given === undefined ? {} : { model: model.given, }),
    ...(wall === ABSENT
      ? {}
      : { wallSeconds: requiredSeconds({
        value: wall,
        name: 'wall',
        usage: POOL_GATE_USAGE,
        errorOf: poolGateOptionsErrorOf,
      },), }),
    ...(sessionOf.given === undefined ? {} : { sessionOfRunFolder: sessionOf.given, }),
    commandLine,
  };
}
