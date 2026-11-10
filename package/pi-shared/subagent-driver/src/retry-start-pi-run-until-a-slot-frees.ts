#!/usr/bin/env node
/**
 `retry-start-pi-run-until-a-slot-frees`: asks `start-pi-run-in-a-free-slot` for a slot again and again until the run starts or
 `--deadline-seconds` pass, then prints what the start answered and exits with its status.

 Usage: `retry-start-pi-run-until-a-slot-frees --deadline-seconds=<n> --period-seconds=<n> -- <start arguments>`

 @module
 */

import {
  extname,
  join,
} from 'node:path';
import process, {
  argv,
  stdout,
} from 'node:process';
import { setTimeout as wait, } from 'node:timers/promises';
import { MILLISECONDS_PER_SECOND, } from './driver-constants.ts';
import { LONGEST_CHILD_MILLISECONDS, } from './longest-child-bound.ts';
import { requiredPositiveNumber, } from './required-option-values.ts';
import {
  type RetryOutcome,
  retryAStartUntilASlotFrees,
  type StartAttempt,
} from './retry-a-start-until-a-slot-frees.ts';
import { runCommandWithChildProcess, } from './run-command.ts';
import { STARTER_COMMAND_NAME, } from './run-launchers.ts';

/**
 The command's command line, for the error message.
 */
const USAGE =
  'retry-start-pi-run-until-a-slot-frees --deadline-seconds=<n> --period-seconds=<n> -- <start-pi-run-in-a-free-slot arguments>';

/**
 How many characters one option's prefix takes, which is the dashes, the name and the equals sign.
 */
const OPTION_PREFIX_CHARACTERS = 3;

/**
 What the start's own output is cut to, which is what a person reads of a start that ran.
 */
const OUTPUT_KEPT_CHARACTERS = 3_000;

/**
 Builds the error this command answers a refused option with.

 @param message - what is wrong with the command line

 @returns the error the command exits with

 @example
 ```ts
 throw retryOptionsErrorOf('--period-seconds is required.',);
 ```
 */
function retryOptionsErrorOf(message: string,): Error {
  return new Error(`${message}\nUsage: ${USAGE}`,);
}

/**
 Reads one option of this command.

 @param name - the option's name, without the leading dashes

 @param value - the option's value as the command line parser read it

 @returns the option's number of seconds

 @example
 ```ts
 const period = secondsOf({ name: 'period-seconds', value: '13', });
 ```
 */
function secondsOf({
  name,
  value,
}: {
  readonly name: string;
  readonly value: unknown;
},): number {
  return requiredPositiveNumber({
    value,
    name,
    usage: USAGE,
    errorOf: retryOptionsErrorOf,
  },);
}

/**
 Where the arguments of this command end and the start's begin.
 */
const separator = argv.indexOf('--',);

/**
 This command's own arguments.
 */
const own = argv.slice(
  2,
  (separator === (-1)) ? argv.length : separator,
);

/**
 The start's arguments, which are handed to it unchanged.
 */
const startArguments = (separator === (-1)) ? [] : argv.slice(separator + 1,);

/**
 Reads the value of one of this command's options.

 @param argument - one of this command's own arguments

 @param name - the option's name, without the leading dashes

 @returns whether this argument writes that option

 @example
 ```ts
 const mine = isOptionOf({ argument: '--period-seconds=13', name: 'period-seconds', });
 ```
 */
function isOptionOf({
  argument,
  name,
}: {
  readonly argument: string;
  readonly name: string;
},): boolean {
  return argument.startsWith(`--${name}=`,);
}

/**
 Reads the value of one of this command's options.

 @param name - the option's name, without the leading dashes

 @returns the option's value, or empty text when it was not given

 @example
 ```ts
 const value = ownOptionOf('period-seconds',);
 ```
 */
function ownOptionOf(name: string,): unknown {
  return own.find(function isThisOption(argument,) {
    return isOptionOf({
      argument,
      name,
    },);
  },)
    ?.slice(name.length + OPTION_PREFIX_CHARACTERS,);
}

/**
 The command that starts a run, beside this one.
 */
const starterScript = join(
  import.meta.dirname,
  `${STARTER_COMMAND_NAME}${extname(import.meta.filename,)}`,
);

/**
 Asks the start for a slot once.

 @returns what it answered

 @example
 ```ts
 const attempt = await startOnce();
 ```
 */
async function startOnce(): Promise<StartAttempt> {
  /**
   What the start answered.
   */
  const answered = await runCommandWithChildProcess({
    command: process.execPath,
    words: [
      starterScript,
      ...startArguments,
    ],
    timeoutMilliseconds: LONGEST_CHILD_MILLISECONDS,
  },);
  return {
    status: answered.status,
    output: `${answered.stdout}${answered.stderr}`.slice(-OUTPUT_KEPT_CHARACTERS,),
  };
}

if (startArguments.length === 0) {
  throw retryOptionsErrorOf('the start\'s own arguments come after `--`.',);
}

/**
 How long the queue may wait for a slot.
 */
const deadlineSeconds = secondsOf({
  name: 'deadline-seconds',
  value: ownOptionOf('deadline-seconds',),
},);

/**
 How long the queue waits between two attempts.
 */
const periodSeconds = secondsOf({
  name: 'period-seconds',
  value: ownOptionOf('period-seconds',),
},);

/**
 What the queue answered.
 */
const outcome: RetryOutcome = await retryAStartUntilASlotFrees({
  startOnce,
  now: Date.now,
  wait: async function passTime(milliseconds,) {
    await wait(milliseconds,);
  },
  deadlineMilliseconds: Date.now() + (deadlineSeconds * MILLISECONDS_PER_SECOND),
  periodMilliseconds: periodSeconds * MILLISECONDS_PER_SECOND,
  describedAs: startArguments.join(' ',),
},);

stdout.write(`${outcome.output}\n`,);
process.exitCode = outcome.status;
