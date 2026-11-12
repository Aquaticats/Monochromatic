#!/usr/bin/env node
/**
 `wait-for-a-unit-to-become-active`: waits, bounded, until a systemd user unit is active, then says whether it came up and what it
 was doing if it did not. See `README.md` for the options.

 It ends a unit nothing: it asks the manager for the unit's state, and it exits 1 when the unit is not active at the deadline.

 @module
 */

import { argv, } from 'node:process';
import { setTimeout as wait, } from 'node:timers/promises';
import { MILLISECONDS_PER_SECOND, } from './driver-constants.ts';
import { optionIn, } from './option-in.ts';
import {
  requiredPositiveNumber,
  requiredText,
} from './required-option-values.ts';
import { runCommandWithChildProcess, } from './run-command.ts';
import { unitStateNow, } from './the-line-a-launcher-ends-with.ts';
import { usageWasAsked, } from './usage-of-a-command.ts';
import {
  waitedLineOf,
  waitForUnitToBecomeActive,
} from './waiting-for-a-unit-to-become-active.ts';

/**
 The command's command line, for the error message.
 */
const USAGE = 'wait-for-a-unit-to-become-active --unit=<name> --deadline-seconds=<n> [--period-seconds=<n>]';

/**
 The end of every systemd unit name.
 */
const UNIT_SUFFIX = '.service';

/**
 Reads the unit name systemd knows, from the name a caller wrote: a name without `.service` gets it, because systemd names every
 unit with it and `is-active` answers `inactive` for a name that is not one.

 @param given - the name the caller wrote, with or without the suffix

 @returns the unit's name

 @example
 ```ts
 const unitName = systemdUnitNameOf({ given: 'launch-a' },);
 ```
 */
export function systemdUnitNameOf({ given, }: { readonly given: string },): string {
  /**
   The name the caller wrote, without the spaces a shell may have left.
   */
  const name = given.trim();
  return name.endsWith(UNIT_SUFFIX,) ? name : `${name}${UNIT_SUFFIX}`;
}
/**
 The option that names the unit.
 */
const UNIT_OPTION = '--unit=';

/**
 The option that names how long the wait may take.
 */
const DEADLINE_OPTION = '--deadline-seconds=';

/**
 The option that names how long to wait between looks.
 */
const PERIOD_OPTION = '--period-seconds=';

/**
 Seconds between looks when the caller names none: long enough that a minute's wait asks the manager about a dozen times, short
 enough that a unit which comes up is seen within a third of a minute.
 */
const DEFAULT_PERIOD_SECONDS = 5;

/**
 A command line this command cannot use.
 */
class UnitWaitOptionsError extends Error {
  /**
   The name printed with the message.
   */
  override readonly name = 'UnitWaitOptionsError';
}

/**
 Builds the error this command answers a refused option with.

 @param message - what is wrong with the command line

 @returns the error the command exits with

 @example
 ```ts
 throw unitWaitOptionsErrorOf('--unit is required.',);
 ```
 */
function unitWaitOptionsErrorOf(message: string,): Error {
  return new UnitWaitOptionsError(message,);
}

// The word that asks for help goes to the command as an argument, and the command answers it before it reads an option
// or touches anything: mise appends a word its task's usage spec does not claim to the task's text, and `--help` is
// mise's own flag, so this is the only place it can land.
if (usageWasAsked({
  words: argv.slice(2,),
  usage: USAGE,
},)) {
  process.exitCode = 0;
  process.exit(0,);
}

/**
 The words this run of the command was given.
 */
const words = argv.slice(2,);

/**
 The unit's name as systemd prints it.
 */
const unitName = systemdUnitNameOf({
  given: requiredText({
    value: optionIn({
      commandLine: words,
      option: UNIT_OPTION,
    },),
    name: 'unit',
    usage: USAGE,
    errorOf: unitWaitOptionsErrorOf,
  },),
},);

/**
 How long the wait may take.
 */
const deadlineSeconds = requiredPositiveNumber({
  value: optionIn({
    commandLine: words,
    option: DEADLINE_OPTION,
  },),
  name: 'deadline-seconds',
  usage: USAGE,
  errorOf: unitWaitOptionsErrorOf,
},);

/**
 How long to wait between looks, as the caller wrote it.
 */
const periodGiven = optionIn({
  commandLine: words,
  option: PERIOD_OPTION,
},);

/**
 What waiting for the unit ended with.
 */
const waited = await waitForUnitToBecomeActive({
  stateOf: async function stateOf() {
    return await unitStateNow({
      run: runCommandWithChildProcess,
      unitName,
    },);
  },
  deadline: Date.now() + (deadlineSeconds * MILLISECONDS_PER_SECOND),
  periodMilliseconds: ((periodGiven === '')
    ? DEFAULT_PERIOD_SECONDS
    : requiredPositiveNumber({
      value: periodGiven,
      name: 'period-seconds',
      usage: USAGE,
      errorOf: unitWaitOptionsErrorOf,
    },)) * MILLISECONDS_PER_SECOND,
  wait,
  now: Date.now,
},);

process.stdout
  .write(`${waitedLineOf({
    unitName,
    waited,
  },)}\n`,);
process.exitCode = waited.becameActive ? 0 : 1;
