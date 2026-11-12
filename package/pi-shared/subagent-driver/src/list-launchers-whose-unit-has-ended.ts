#!/usr/bin/env node
/**
 `list-launchers-whose-unit-has-ended`: names every launcher that is still running although the unit it started has ended, with the
 unit's state, how long the launcher has been up and the processes it started. With `--stop` it ends each of them, children first.
 See `README.md` for the options.

 A launcher that outlives its unit makes the pool gate answer `ALREADY RUNNING` for that run's name and holds that run's slot, and
 it waited on a waiter that never heard the unit end, so this is the line a coordinator acts on. It never ends a launcher whose
 unit's state could not be read.

 @module
 */

import {
  argv,
  stdout,
} from 'node:process';
import {
  LAUNCHER_ENTRY_NAMES,
  REPLACED_LAUNCHER_ENTRY_NAMES,
} from './going-runs.ts';
import {
  LAUNCHER_MINIMUM_AGE_SECONDS,
  childrenOf,
  launcherLineOf,
  launchersIn,
  launchersPastTheirUnits,
  stopLaunchers,
} from './launchers-whose-unit-has-ended.ts';
import { optionIn, } from './option-in.ts';
import { startProcessTable, } from './processes-under-a-pi-run.ts';
import { requiredPositiveNumber, } from './required-option-values.ts';
import { runCommandWithChildProcess, } from './run-command.ts';
import {
  unitStateIsRunning,
  unitStateOf,
} from './wait-for-a-unit-to-end.ts';

/**
 The command's command line, for the error message.
 */
const USAGE =
  'list-launchers-whose-unit-has-ended [--launcher-entry=<basename>]... [--minimum-age-seconds=<n>] [--stop]';

/**
 The option that names the basename a launcher is run as.
 */
const LAUNCHER_ENTRY_OPTION = '--launcher-entry=';

/**
 The option that names how long a launcher must have been running.
 */
const MINIMUM_AGE_OPTION = '--minimum-age-seconds=';

/**
 The option that asks for each launcher it names to be ended.
 */
const STOP_OPTION = '--stop';

/**
 A command line the command cannot use.
 */
class LauncherListOptionsError extends Error {
  /**
   The name printed with the message.
   */
  override readonly name = 'LauncherListOptionsError';
}

/**
 Builds the error this command answers a refused option with.

 @param message - what is wrong with the command line

 @returns the error the command exits with

 @example
 ```ts
 throw launcherListOptionsErrorOf('--minimum-age-seconds must be above zero.',);
 ```
 */
function launcherListOptionsErrorOf(message: string,): Error {
  return new LauncherListOptionsError(message,);
}

/**
 Ends one process with the polite signal, which is the only place this command signals anything.

 @param pid - the process number

 @param signal - the signal's name

 @example
 ```ts
 signalProcess({ pid: 12, signal: STOP_SIGNAL, });
 ```
 */
function signalProcess({
  pid,
  signal,
}: {
  readonly pid: number;
  readonly signal: string;
},): void {
  process.kill(
    pid,
    signal,
  );
}

/**
 The words the command was given.
 */
const words = argv.slice(2,);

/**
 The basenames the caller named a launcher by.
 */
const entriesGiven = words.filter(function namesAnEntry(word,) {
  return word.startsWith(LAUNCHER_ENTRY_OPTION,);
},)
  .map(function entryOf(word,) {
    return word.slice(LAUNCHER_ENTRY_OPTION.length,);
  },);

/**
 The basenames a launcher is run as: the caller's own, or this package's launcher and the scratch one it replaced.
 */
const launcherEntryNames = (entriesGiven.length === 0)
  ? [
    ...LAUNCHER_ENTRY_NAMES,
    ...REPLACED_LAUNCHER_ENTRY_NAMES,
  ]
  : entriesGiven;

/**
 The age a launcher must have reached, as it was written.
 */
const minimumAgeGiven = optionIn({
  commandLine: words,
  option: MINIMUM_AGE_OPTION,
},);

/**
 What this run of the command was asked for.
 */
const options = {
  launcherEntryNames,
  minimumAgeSeconds: (minimumAgeGiven === '')
    ? LAUNCHER_MINIMUM_AGE_SECONDS
    : requiredPositiveNumber({
      value: minimumAgeGiven,
      name: 'minimum-age-seconds',
      usage: USAGE,
      errorOf: launcherListOptionsErrorOf,
    },),
  stop: words.includes(STOP_OPTION,),
};

/**
 The process table, read once so every launcher is described at the same moment.
 */
const table = await startProcessTable({ run: runCommandWithChildProcess, },)();

/**
 The launchers that outlived their units.
 */
const past = await launchersPastTheirUnits({
  launchers: launchersIn({
    table,
    launcherEntryNames: options.launcherEntryNames,
  },),
  minimumAgeSeconds: options.minimumAgeSeconds,
  unitStateOf: async function stateOf(unitName,) {
    return await unitStateOf({
      run: runCommandWithChildProcess,
      unitName,
    },);
  },
  stateIsRunning: function isRunning(state,) {
    return unitStateIsRunning({ state, },);
  },
  childrenOfParent: function childrenOfLauncher(pid,) {
    return childrenOf({
      table,
      parentPid: pid,
    },);
  },
},);

/**
 The lines to print: one per launcher, then what could not be signalled.
 */
const printed: string[] = [];
if (past.length === 0) {
  /**
   How many entries were looked for, so a caller can tell no launcher from no entry.
   */
  const looked = String(
    options.launcherEntryNames
      .length,
  );
  printed.push(`no launcher outlived its unit (${looked} entries looked for)`,);
}
else {
  printed.push(...past.map(function lineOf(launcher,) {
    return launcherLineOf(launcher,);
  },),);
}
if (options.stop && (past.length > 0)) {
  /**
   What could not be signalled, which the caller's status reports.
   */
  const failures = stopLaunchers({
    past,
    signal: signalProcess,
  },);
  printed.push(...failures,);
  if (failures.length > 0) {
    process.exitCode = 1;
  }
}
stdout.write(`${printed.join('\n',)}\n`,);
