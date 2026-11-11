/**
 Reads the driver's command line. Every limit is required, so a run is always bounded and no default stands in for a decision.

 @module
 */

import { parseArgs, } from 'node:util';
import {
  MODEL_ERROR_CONCURRENCY_WAIT_SECONDS,
  MODEL_ERROR_RETRY_BASE_SECONDS,
  MODEL_ERROR_RETRY_CAP,
} from './driver-constants.ts';
import type { DriverOptions, } from './driver-types.ts';
import {
  type RequiredOption,
  requiredPositiveNumber,
  requiredSeconds,
  requiredText,
} from './required-option-values.ts';

/**
 The driver's command line, for the error message.
 */
export const DRIVER_USAGE =
  'drive-pi-subagent --run-folder=<folder> --working-folder=<folder> --prompt-file=<file> --provider=<name> --model=<name> --wall-seconds=<n> --idle-seconds=<n> --turn-cap=<n> [--session=<file>] [--retry-cap=<n>] [--retry-base-seconds=<n>] [--concurrency-wait-seconds=<n>] [--pi-argument=<word>]...';

/**
 A command line the driver cannot use.
 */
export class DriverOptionsError extends Error {
  /**
   The name printed with the message.
   */
  override readonly name = 'DriverOptionsError';
}

/**
 Builds the error this command answers a refused option with.

 @param message - what is wrong with the command line

 @returns the error the command exits with

 @example
 ```ts
 throw driverOptionsErrorOf('--runs-folder is required.',);
 ```
 */
function driverOptionsErrorOf(message: string,): Error {
  return new DriverOptionsError(message,);
}

/**
 Describes one option for the readers that refuse it.

 @param value - the option's value as the command line parser read it

 @param name - the option's name, without the leading dashes

 @returns what a reader of an option needs to refuse it

 @example
 ```ts
 const option = optionOf({
          value: values['run-folder'],
          name: 'run-folder',
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
    usage: DRIVER_USAGE,
    errorOf: driverOptionsErrorOf,
  };
}

/**
 Reads how many attempts may follow a model error: a whole number of them, of which zero is allowed because a coordinator may
 want a run that never retries.

 @param value - the option's value as the command line parser read it

 @param name - the option's name, without the leading dashes

 @param usage - the command's usage, for the message

 @param errorOf - builds the error this command answers a refused option with

 @returns the number of attempts

 @throws the error this command answers a refused option with, when the value is no whole number of attempts

 @example
 ```ts
 const retryCap = retryCapOf({ value: '0', name: 'retry-cap', usage, errorOf, },);
 ```
 */
function retryCapOf({
  value,
  name,
  usage,
  errorOf,
}: RequiredOption,): number {
  /**
   The option's text.
   */
  const text = requiredText({
    value,
    name,
    usage,
    errorOf,
  },);
  /**
   The option's number of attempts.
   */
  const parsed = Number(text,);
  if ((!Number.isSafeInteger(parsed,)) || (parsed < 0)) {
    throw errorOf(`--${name} must be a whole number of attempts, zero or more; it was "${text}".\nUsage: ${usage}`,);
  }
  return parsed;
}

/**
 Reads the command line.

 @param commandLine - arguments after the program name

 @returns options of this run

 @throws DriverOptionsError when an option is unknown, missing or unusable

 @example
 ```ts
 const options = parseDriverOptions(process.argv.slice(2,),);
 ```
 */
export function parseDriverOptions(commandLine: readonly string[],): DriverOptions {
  try {
    /**
     The options as written.
     */
    const { values, } = parseArgs({
      args: [...commandLine,],
      strict: true,
      allowPositionals: false,
      options: {
        'run-folder': { type: 'string', },
        'working-folder': { type: 'string', },
        'prompt-file': { type: 'string', },
        provider: { type: 'string', },
        model: { type: 'string', },
        session: { type: 'string', },
        'pi-argument': {
          type: 'string',
          multiple: true,
        },
        'wall-seconds': { type: 'string', },
        'idle-seconds': { type: 'string', },
        'turn-cap': { type: 'string', },
        'retry-cap': { type: 'string', },
        'retry-base-seconds': { type: 'string', },
        'concurrency-wait-seconds': { type: 'string', },
      },
    },);
    return {
      runFolder: requiredText(optionOf({
        value: values['run-folder'],
        name: 'run-folder',
      },),),
      workingFolder: requiredText(optionOf({
        value: values['working-folder'],
        name: 'working-folder',
      },),),
      promptFile: requiredText(optionOf({
        value: values['prompt-file'],
        name: 'prompt-file',
      },),),
      provider: requiredText(optionOf({
        value: values.provider,
        name: 'provider',
      },),),
      model: requiredText(optionOf({
        value: values.model,
        name: 'model',
      },),),
      ...(values.session === undefined ? {} : { session: values.session, }),
      piArguments: values['pi-argument'] ?? [],
      wallSeconds: requiredSeconds(optionOf({
        value: values['wall-seconds'],
        name: 'wall-seconds',
      },),),
      idleSeconds: requiredSeconds(optionOf({
        value: values['idle-seconds'],
        name: 'idle-seconds',
      },),),
      turnCap: requiredPositiveNumber(optionOf({
        value: values['turn-cap'],
        name: 'turn-cap',
      },),),
      retryCap: (values['retry-cap'] === undefined)
        ? MODEL_ERROR_RETRY_CAP
        : retryCapOf(optionOf({
          value: values['retry-cap'],
          name: 'retry-cap',
        },),),
      retryBaseSeconds: (values['retry-base-seconds'] === undefined) ? MODEL_ERROR_RETRY_BASE_SECONDS : requiredSeconds(
        optionOf({
          value: values['retry-base-seconds'],
          name: 'retry-base-seconds',
        },),
      ),
      concurrencyWaitSeconds: (values['concurrency-wait-seconds'] === undefined)
        ? MODEL_ERROR_CONCURRENCY_WAIT_SECONDS
        : requiredSeconds(optionOf({
          value: values['concurrency-wait-seconds'],
          name: 'concurrency-wait-seconds',
        },),),
    };
  }
  catch (error) {
    if (error instanceof DriverOptionsError) {
      throw error;
    }
    throw new DriverOptionsError(
      `${String(error,)}\nUsage: ${DRIVER_USAGE}`,
      { cause: error, },
    );
  }
}
