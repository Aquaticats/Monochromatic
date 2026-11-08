/**
 Reads the driver's command line. Every limit is required, so a run is always bounded and no default stands in for a decision.

 @module
 */

import { parseArgs, } from 'node:util';
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
  override readonly name = 'DriverOptionsError';
}

/**
 Values as `parseArgs` gives them.
 */
type ParsedValues = Readonly<Record<string, string | boolean | (string | boolean)[] | undefined>>;

/**
 Reads an option that must be present.

 @param request - parsed values and option name

 @returns option's text

 @throws DriverOptionsError when option is missing or empty

 @example
 ```ts
 const folder = requiredText({ values: { 'run-folder': 'x', }, name: 'run-folder', },);
 ```
 */
function requiredText({ values, name, }: { readonly values: ParsedValues; readonly name: string; },): string {
  /**
   The value given, of any shape.
   */
  const value = values[name];
  if ((typeof value !== 'string') || (value === '')) {
    throw new DriverOptionsError(`--${name} is required.\nUsage: ${DRIVER_USAGE}`,);
  }
  return value;
}

/**
 Reads an option that must be a number above zero.

 @param request - parsed values and option name

 @returns option's number

 @throws DriverOptionsError when option is missing or not a number above zero

 @example
 ```ts
 const seconds = requiredPositiveNumber({ values: { 'wall-seconds': '60', }, name: 'wall-seconds', },);
 ```
 */
function requiredPositiveNumber({ values, name, }: { readonly values: ParsedValues; readonly name: string; },): number {
  /**
   The option's text.
   */
  const text = requiredText({ values, name, },);
  /**
   The option's number.
   */
  const value = Number(text,);
  if (!Number.isFinite(value,) || (value <= 0)) {
    throw new DriverOptionsError(`--${name} must be a number above zero; it was "${text}".\nUsage: ${DRIVER_USAGE}`,);
  }
  return value;
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
        'pi-argument': { type: 'string', multiple: true, },
        'wall-seconds': { type: 'string', },
        'idle-seconds': { type: 'string', },
        'turn-cap': { type: 'string', },
      },
    },);
    return {
      runFolder: requiredText({ values, name: 'run-folder', },),
      workingFolder: requiredText({ values, name: 'working-folder', },),
      promptFile: requiredText({ values, name: 'prompt-file', },),
      provider: requiredText({ values, name: 'provider', },),
      model: requiredText({ values, name: 'model', },),
      session: values.session ?? null,
      piArguments: values['pi-argument'] ?? [],
      wallSeconds: requiredPositiveNumber({ values, name: 'wall-seconds', },),
      idleSeconds: requiredPositiveNumber({ values, name: 'idle-seconds', },),
      turnCap: requiredPositiveNumber({ values, name: 'turn-cap', },),
    };
  }
  catch (error) {
    if (error instanceof DriverOptionsError) {
      throw error;
    }
    throw new DriverOptionsError(`${String(error,)}\nUsage: ${DRIVER_USAGE}`, { cause: error, },);
  }
}
