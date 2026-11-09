/**
 Readings of command-line options that both commands of this package refuse the same way: a value that is missing, empty, not a
 number above zero, or a duration whose deadline would not be finite is never allowed to stand.

 @module
 */

import { MILLISECONDS_PER_SECOND, } from './driver-constants.ts';

/**
 What a caller needs to refuse one option: the text it reads and the error its own command answers with.

 A command line error carries the usage of the command that read it, so the caller says how to build that error rather than this
 module knowing about either command.
 */
export type RequiredOption = {
  /**
   The option's value as the command line parser read it.
   */
  readonly value: unknown;

  /**
   The option's name, without the leading dashes, for the message.
   */
  readonly name: string;

  /**
   The command's usage, for the message.
   */
  readonly usage: string;

  /**
   Builds the error this command answers a refused option with.
   */
  readonly errorOf: (message: string) => Error;
};

/**
 Reads an option that must be present.

 @param value - the option's value as the command line parser read it

 @param name - the option's name, without the leading dashes, for the message

 @param usage - the command's usage, for the message

 @param errorOf - builds the error this command answers a refused option with

 @returns the option's text

 @throws the error the caller builds, when the option is missing or empty

 @example
 ```ts
 const folder = requiredText({ value: 'x', name: 'run-folder', usage, errorOf, },);
 ```
 */
export function requiredText({
  value,
  name,
  usage,
  errorOf,
}: RequiredOption,): string {
  if (((typeof value) !== 'string') || (value === '')) {
    throw errorOf(`--${name} is required.\nUsage: ${usage}`,);
  }
  return value;
}

/**
 Reads an option that must be a number above zero.

 @param value - the option's value as the command line parser read it

 @param name - the option's name, without the leading dashes, for the message

 @param usage - the command's usage, for the message

 @param errorOf - builds the error this command answers a refused option with

 @returns the option's number

 @throws the error the caller builds, when the option is missing or not a number above zero

 @example
 ```ts
 const turnCap = requiredPositiveNumber({ value: '80', name: 'turn-cap', usage, errorOf, },);
 ```
 */
export function requiredPositiveNumber({
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
   The option's number.
   */
  const parsed = Number(text,);
  if ((!Number.isFinite(parsed,)) || (parsed <= 0)) {
    throw errorOf(`--${name} must be a number above zero; it was "${text}".\nUsage: ${usage}`,);
  }
  return parsed;
}

/**
 Reads an option that must be a duration above zero whose milliseconds, and whose deadline from now, stay finite. A limit that
 becomes infinite is no limit at all.

 @param value - the option's value as the command line parser read it

 @param name - the option's name, without the leading dashes, for the message

 @param usage - the command's usage, for the message

 @param errorOf - builds the error this command answers a refused option with

 @returns the option's number of seconds

 @throws the error the caller builds, when the option is missing, not a number above zero, or too large to be a deadline

 @example
 ```ts
 const wallSeconds = requiredSeconds({ value: '60', name: 'wall-seconds', usage, errorOf, },);
 ```
 */
export function requiredSeconds({
  value,
  name,
  usage,
  errorOf,
}: RequiredOption,): number {
  /**
   The option's seconds.
   */
  const seconds = requiredPositiveNumber({
    value,
    name,
    usage,
    errorOf,
  },);
  /**
   The milliseconds a deadline is computed from, which is what any timer receives.
   */
  const milliseconds = seconds * MILLISECONDS_PER_SECOND;
  if ((!Number.isFinite(milliseconds,)) || (!Number.isFinite(Date.now() + milliseconds,))) {
    throw errorOf(`--${name} must be a number of seconds whose milliseconds stay finite; it was "${String(seconds,)}".\nUsage: ${usage}`,);
  }
  return seconds;
}
