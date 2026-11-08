/**
 Reads the driver's command line. Every limit is required, so a run is always bounded and no default stands in for a decision.

 @module
 */

import { parseArgs, } from 'node:util';
import { MILLISECONDS_PER_SECOND, } from './driver-constants.ts';
import type { DriverOptions, } from './driver-types.ts';

/**
 The driver's command line, for the error message.
 */
export const DRIVER_USAGE =
  'drive-pi-subagent --run-folder=<folder> --working-folder=<folder> --prompt-file=<file> --provider=<name> --model=<name> --wall-seconds=<n> --idle-seconds=<n> --turn-cap=<n> [--session=<file>] [--pi-argument=<word>]...';

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
 Reads an option that must be present.

 @param value - option's value as parsed

 @param name - option's name for the message

 @returns option's text

 @throws DriverOptionsError when option is missing or empty

 @example
 ```ts
 const folder = requiredText({ value: 'x', name: 'run-folder', },);
 ```
 */
function requiredText({
  value,
  name,
}: {
  readonly value: unknown;
  readonly name: string
},): string {
  if (((typeof value) !== 'string') || (value === '')) {
    throw new DriverOptionsError(`--${name} is required.\nUsage: ${DRIVER_USAGE}`,);
  }
  return value;
}

/**
 Reads an option that must be a number above zero.

 @param value - option's value as parsed

 @param name - option's name for the message

 @returns option's number

 @throws DriverOptionsError when option is missing or not a number above zero

 @example
 ```ts
 const seconds = requiredPositiveNumber({ value: '60', name: 'wall-seconds', },);
 ```
 */
function requiredPositiveNumber({
  value,
  name,
}: {
  readonly value: unknown;
  readonly name: string
},): number {
  /**
   The option's text.
   */
  const text = requiredText({
    value,
    name,
  },);
  /**
   The option's number.
   */
  const parsed = Number(text,);
  if ((!Number.isFinite(parsed,)) || (parsed <= 0)) {
    throw new DriverOptionsError(`--${name} must be a number above zero; it was "${text}".\nUsage: ${DRIVER_USAGE}`,);
  }
  return parsed;
}

/**
 Reads an option that must be a duration above zero whose milliseconds, and whose deadline from now, stay finite. A limit that
 becomes infinite is no limit at all.

 @param value - option's value as parsed

 @param name - option's name for the message

 @returns option's number of seconds

 @throws DriverOptionsError when option is missing, not a number above zero, or too large to be a deadline

 @example
 ```ts
 const seconds = requiredSeconds({ value: '60', name: 'wall-seconds', },);
 ```
 */
function requiredSeconds({
  value,
  name,
}: {
  readonly value: unknown;
  readonly name: string
},): number {
  /**
   The option's seconds.
   */
  const seconds = requiredPositiveNumber({
    value,
    name,
  },);
  /**
   The milliseconds the driver computes a deadline from, which is what any timer receives.
   */
  const milliseconds = seconds * MILLISECONDS_PER_SECOND;
  if ((!Number.isFinite(milliseconds,)) || (!Number.isFinite(Date.now() + milliseconds,))) {
    throw new DriverOptionsError(`--${name} must be a number of seconds whose milliseconds stay finite; it was "${String(seconds,)}".\nUsage: ${DRIVER_USAGE}`,);
  }
  return seconds;
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
      },
    },);
    return {
      runFolder: requiredText({
        value: values['run-folder'],
        name: 'run-folder',
      },),
      workingFolder: requiredText({
        value: values['working-folder'],
        name: 'working-folder',
      },),
      promptFile: requiredText({
        value: values['prompt-file'],
        name: 'prompt-file',
      },),
      provider: requiredText({
        value: values.provider,
        name: 'provider',
      },),
      model: requiredText({
        value: values.model,
        name: 'model',
      },),
      ...(values.session === undefined ? {} : { session: values.session, }),
      piArguments: values['pi-argument'] ?? [],
      wallSeconds: requiredSeconds({
        value: values['wall-seconds'],
        name: 'wall-seconds',
      },),
      idleSeconds: requiredSeconds({
        value: values['idle-seconds'],
        name: 'idle-seconds',
      },),
      turnCap: requiredPositiveNumber({
        value: values['turn-cap'],
        name: 'turn-cap',
      },),
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
