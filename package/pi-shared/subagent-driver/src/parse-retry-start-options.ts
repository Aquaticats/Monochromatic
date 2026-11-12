/**
 Reads the command line of the retrying start: how long it keeps asking, how long it waits between asks, and the pool gate's own
 words after a `--`.

 @module
 */

import {
  type RequiredOption,
  requiredSeconds,
} from './required-option-values.ts';

/**
 The command's command line, for the error message.
 */
export const RETRY_START_USAGE =
  'retry-pi-run-start-until-a-pool-has-room --deadline-seconds=<n> --period-seconds=<n> [--] <pool gate options>';

/**
 The word that separates this command's own options from the pool gate's, which a caller may write and which a task runner may drop.
 */
const SEPARATOR = '--';

/**
 This command's own options, which it reads wherever they stand and never hands to the pool gate.
 */
const OWN_OPTIONS: readonly string[] = [
  'deadline-seconds',
  'period-seconds',
];

/**
 A command line the retrying start cannot use.
 */
export class RetryStartOptionsError extends Error {
  /**
   The name printed with the message.
   */
  override readonly name = 'RetryStartOptionsError';
}

/**
 Builds the error this command answers a refused option with.

 @param message - what is wrong with the command line

 @returns the error the command exits with

 @example
 ```ts
 throw retryStartOptionsErrorOf('--deadline-seconds is required.',);
 ```
 */
function retryStartOptionsErrorOf(message: string,): Error {
  return new RetryStartOptionsError(message,);
}

/**
 Describes one option for the readers that refuse it.

 @param value - the option's value as the command line read it

 @param name - the option's name, without the leading dashes

 @returns what a reader of an option needs to refuse it

 @example
 ```ts
 const option = optionOf({ value: '60', name: 'deadline-seconds', },);
 ```
 */
function optionOf({
  value,
  name,
}: {
  readonly value: unknown;
  readonly name: string;
}): RequiredOption {
  return {
    value,
    name,
    usage: RETRY_START_USAGE,
    errorOf: retryStartOptionsErrorOf,
  };
}

/**
 Reads the value of one of this command's own options.

 @param own - this command's words, before the separator

 @param name - the option's name, without the leading dashes

 @returns the option's text, or nothing when the line does not carry it

 @example
 ```ts
 const written = writtenOption({ own: ['--period-seconds=30',], name: 'period-seconds', },);
 ```
 */
function writtenOption({
  own,
  name,
}: {
  readonly own: readonly string[];
  readonly name: string;
}): { readonly written?: string } {
  /**
   The option as it was written.
   */
  const written = own.find(function isTheOption(word,) {
    return word.startsWith(`--${name}=`,);
  },)
    ?.slice(`--${name}=`.length,);
  return (written === undefined) ? {} : { written, };
}

/**
 Reads a duration this command cannot do without.

 @param own - this command's words, before the separator

 @param name - the option's name, without the leading dashes

 @returns the seconds

 @throws RetryStartOptionsError when the option is missing or unusable

 @example
 ```ts
 const seconds = requiredOption({ own, name: 'deadline-seconds', },);
 ```
 */
function requiredOption({
  own,
  name,
}: {
  readonly own: readonly string[];
  readonly name: string;
}): number {
  /**
   The option as it was written.
   */
  const written = writtenOption({
    own,
    name,
  },);
  return requiredSeconds(optionOf({
    value: written.written,
    name,
  },),);
}

/**
 What the retrying start was asked for.
 */
export type RetryStartOptions = {
  /**
   How long it keeps asking for a slot.
   */
  readonly deadlineSeconds: number;

  /**
   How long it waits between asks.
   */
  readonly periodSeconds: number;

  /**
   The pool gate's words, which are also repeated by the line that says no slot freed.
   */
  readonly words: readonly string[];
};

/**
 Reads the command line.

 This command reads its own two options wherever they stand, and hands every other word to the pool gate, so a `--` between them
 is optional: `mise run //package/pi-shared/subagent-driver:retry-pi-run-start-until-a-pool-has-room -- --deadline-seconds=1
 --period-seconds=1 -- --name=a` lost its second `--` before it reached the script, and the task could not be used as its own
 usage line said. The gate has no `--deadline-seconds` and no `--period-seconds` of its own, so a word of either name can only be
 this command's.

 @param commandLine - arguments after the program name

 @returns what the retrying start was asked for

 @throws RetryStartOptionsError when an option is missing or unusable, or no word for the pool gate was given

 @example
 ```ts
 const options = parseRetryStartOptions(['--deadline-seconds=7200', '--period-seconds=30', '--name=a',],);
 ```
 */
export function parseRetryStartOptions(commandLine: readonly string[],): RetryStartOptions {
  /**
   This command's own words.
   */
  const own = commandLine.filter(function isOwnOption(word,) {
    return OWN_OPTIONS.some(function namesIt(name,) {
      return word.startsWith(`--${name}=`,);
    },);
  },);
  /**
   The pool gate's words: every word that is neither this command's nor a separator.
   */
  const words = commandLine.filter(function isTheGates(word,) {
    return (word !== SEPARATOR) && (!own.includes(word,));
  },);
  if (words.length === 0) {
    throw retryStartOptionsErrorOf(`no pool gate word was given.\nUsage: ${RETRY_START_USAGE}`,);
  }
  return {
    deadlineSeconds: requiredOption({
      own,
      name: 'deadline-seconds',
    },),
    periodSeconds: requiredOption({
      own,
      name: 'period-seconds',
    },),
    words,
  };
}
