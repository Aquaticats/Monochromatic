/**
 Reads the command line of `check-in-on-pi-runs`: which runs folder, and which units to describe.

 @module
 */

import { parseArgs, } from 'node:util';
import { unitNameOfGiven, } from './running-pi-units.ts';
import {
  type RequiredOption,
  requiredText,
} from './required-option-values.ts';

/**
 The command's command line, for the error message.
 */
export const CHECK_IN_USAGE =
  'check-in-on-pi-runs --runs-folder=<folder> --unit=<name>...';

/**
 A command line the check-in cannot use.
 */
export class CheckInOptionsError extends Error {
  /**
   The name printed with the message.
   */
  override readonly name = 'CheckInOptionsError';
}

/**
 What one run of the command was asked for.
 */
export type CheckInOptions = {
  /**
   Folder the driver writes one run folder per run into.
   */
  readonly runsFolder: string;

  /**
   The units to describe, as systemd names them.
   */
  readonly unitNames: readonly string[];
};

/**
 Builds the error this command answers a refused option with.

 @param message - what is wrong with the command line

 @returns the error the command exits with

 @example
 ```ts
 throw checkInOptionsErrorOf('--runs-folder is required.',);
 ```
 */
function checkInOptionsErrorOf(message: string,): Error {
  return new CheckInOptionsError(message,);
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
    usage: CHECK_IN_USAGE,
    errorOf: checkInOptionsErrorOf,
  };
}

/**
 Reads the command line.

 @param commandLine - arguments after the program name

 @returns what the run was asked for

 @throws CheckInOptionsError when an option is unknown, missing, empty or names no unit

 @example
 ```ts
 const options = parseCheckInOptions(argv.slice(2,),);
 ```
 */
export function parseCheckInOptions(commandLine: readonly string[],): CheckInOptions {
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
        unit: {
          type: 'string',
          multiple: true,
        },
      },
    },);
    /**
     The units named, which the command describes one by one.
     */
    const units = values.unit ?? [];
    if (units.length === 0) {
      throw new CheckInOptionsError(`--unit is required.\nUsage: ${CHECK_IN_USAGE}`,);
    }
    return {
      runsFolder: requiredText(optionOf({
        value: values['runs-folder'],
        name: 'runs-folder',
      },),),
      unitNames: units.map(function asUnitName(name,) {
        return unitNameOfGiven({ name, },);
      },),
    };
  }
  catch (error) {
    if (error instanceof CheckInOptionsError) {
      throw error;
    }
    throw new CheckInOptionsError(
      `${String(error,)}\nUsage: ${CHECK_IN_USAGE}`,
      { cause: error, },
    );
  }
}
