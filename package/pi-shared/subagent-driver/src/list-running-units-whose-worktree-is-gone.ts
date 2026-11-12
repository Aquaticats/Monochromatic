#!/usr/bin/env node
/**
 `list-running-units-whose-worktree-is-gone`: names every running unit whose worktree folder a merge or a removal took away, and
 says how many running units were listed. See `README.md` for the options.

 A unit without its worktree cannot commit where a reader looks for its work, and it holds its pool slot until its own bound ends
 it, so this is the line a coordinator acts on.

 @module
 */

import { access, } from 'node:fs/promises';
import {
  argv,
  cwd,
  stdout,
} from 'node:process';
import { errorCodeOf, } from './error-code-of.ts';
import { checkoutFolderOf, } from './checkout-of-a-command.ts';
import { optionIn, } from './option-in.ts';
import { requiredText, } from './required-option-values.ts';
import { runCommandWithChildProcess, } from './run-command.ts';
import {
  countLineOf,
  goneLineOf,
  serviceUnitsIn,
  unitsWhoseWorktreeIsGone,
} from './units-whose-worktree-is-gone.ts';
import { usageWasAsked, } from './usage-of-a-command.ts';

/**
 The code the operating system gives a path that is not there.
 */
const MISSING_PATH_CODE = 'ENOENT';

/**
 The command's command line, for the error message.
 */
const RUNNING_UNITS_USAGE =
  'list-running-units-whose-worktree-is-gone [--checkout-folder=<folder>] [--unit-pattern=<pattern>]';

// The word that asks for help goes to the command as an argument, and the command answers it before it reads an option
// or touches anything: mise appends a word its task's usage spec does not claim to the task's text, and `--help` is
// mise's own flag, so this is the only place it can land.
if (usageWasAsked({
  words: argv.slice(2,),
  usage: RUNNING_UNITS_USAGE,
},)) {
  process.exitCode = 0;
  process.exit(0,);
}

/**
 The option that names the checkout whose worktrees the launcher makes.
 */
const CHECKOUT_FOLDER_OPTION = '--checkout-folder=';

/**
 The option that names the units to look at, as systemd matches a unit name.
 */
const UNIT_PATTERN_OPTION = '--unit-pattern=';

/**
 The units this command looks at when a caller names none: every unit of a Pi run, as the recipe names them.
 */
const PI_UNIT_PATTERN = 'pi-*';

/**
 Says whether a folder is there.

 @param folder - the folder to look for

 @returns whether it is there

 @throws the error a read gave, other than a folder that is not there

 @example
 ```ts
 const there = await isThere('/repo/.claude/worktrees/pi-a');
 ```
 */
async function isThere(folder: string,): Promise<boolean> {
  try {
    await access(folder,);
    return true;
  }
  catch (error) {
    if (errorCodeOf(error,) === MISSING_PATH_CODE) {
      return false;
    }
    throw error;
  }
}

/**
 A command line the command cannot use.
 */
class RunningUnitsOptionsError extends Error {
  /**
   The name printed with the message.
   */
  override readonly name = 'RunningUnitsOptionsError';
}

/**
 Builds the error this command answers a refused option with.

 @param message - what is wrong with the command line

 @returns the error the command exits with

 @example
 ```ts
 throw runningUnitsOptionsErrorOf('--checkout-folder must be a folder.',);
 ```
 */
function runningUnitsOptionsErrorOf(message: string,): Error {
  return new RunningUnitsOptionsError(message,);
}

/**
 The words the command was given.
 */
const words = argv.slice(2,);

/**
 The checkout folder the caller named.
 */
const checkoutFolderGiven = optionIn({
  commandLine: words,
  option: CHECKOUT_FOLDER_OPTION,
},);

/**
 The unit pattern the caller named.
 */
const unitPatternGiven = optionIn({
  commandLine: words,
  option: UNIT_PATTERN_OPTION,
},);

/**
 What this run of the command was asked for.
 */
const options = {
  checkoutFolder: (checkoutFolderGiven === '')
    // A task of this package runs in the package folder, so the folder the command was started in is not the checkout: the
    // checkout is the folder above it that holds `.git`, and a default of the package folder reads every unit as a run whose
    // worktree is gone, which is what this command reported on 2026-11-11.
    ? await checkoutFolderOf({ from: cwd(), },)
    : checkoutFolderGiven,
  // A pattern that is empty is one systemd would match every unit with, which is never what a caller meant.
  unitPattern: (unitPatternGiven === '')
    ? PI_UNIT_PATTERN
    : requiredText({
      value: unitPatternGiven,
      name: 'unit-pattern',
      usage: RUNNING_UNITS_USAGE,
      errorOf: runningUnitsOptionsErrorOf,
    },),
};

/**
 What systemd said about the running units.
 */
const listed = await runCommandWithChildProcess({
  command: 'systemctl',
  words: [
    '--user',
    'list-units',
    '--type=service',
    '--state=running',
    '--no-legend',
    '--plain',
    options.unitPattern,
  ],
},);
if (listed.status !== 0) {
  throw new Error(`systemctl could not list the running units: ${listed.problem
    ?? listed.stderr
      .trim()}`,);
}

/**
 The units that are running now.
 */
const unitNames = serviceUnitsIn({ shown: listed.stdout, },);

/**
 The units whose worktree folder is gone.
 */
const gone = await unitsWhoseWorktreeIsGone({
  unitNames,
  checkoutFolder: options.checkoutFolder,
  isThere,
},);

stdout.write(`${
  [
    ...gone.map(function lineOf(unit,) {
      return goneLineOf(unit,);
    },),
    countLineOf({
      listed: unitNames.length,
      gone: gone.length,
    },),
  ].join('\n',)
}\n`,);
