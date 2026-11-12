#!/usr/bin/env node
/**
 `list-running-pi-runs-with-their-turns`: prints one line per Pi run going now, naming its turns and how long its feed has been
 quiet, so a run that started and never took a turn shows beside one that is working. See `README.md` for the options.

 Nothing here ends or signals a run: it reads systemd's list of running units and the feeds under the runs folder.

 @module
 */

import { homedir, } from 'node:os';
import { join, } from 'node:path';
import { argv, } from 'node:process';
import { optionIn, } from './option-in.ts';
import { runCommandWithChildProcess, } from './run-command.ts';
import { DEFAULT_RUNS_FOLDER_IN_HOME, } from './reviewer-of-a-change.ts';
import { startRunningPiUnits, } from './running-pi-units.ts';
import { runningRunLinesOf, } from './turns-of-running-runs.ts';
import { usageWasAsked, } from './usage-of-a-command.ts';

/**
 The command's command line, for the error message.
 */
const USAGE = 'list-running-pi-runs-with-their-turns [--runs-folder=<folder>]';

/**
 The option that names the folder the driver writes one run folder per run into.
 */
const RUNS_FOLDER_OPTION = '--runs-folder=';

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
 The folder the runs live in, which is the one a coordinator's runs folder has always been unless the caller names another.
 */
const runsFolder = optionIn({
  commandLine: argv.slice(2,),
  option: RUNS_FOLDER_OPTION,
},) || join(
  homedir(),
  ...DEFAULT_RUNS_FOLDER_IN_HOME,
);

/**
 The Pi runs going now, as systemd lists them.
 */
const units = await startRunningPiUnits({ run: runCommandWithChildProcess, },)();

/**
 The lines one running run prints.
 */
const lines = await runningRunLinesOf({
  units,
  runsFolder,
  now: Date.now,
},);

process.stdout
  .write(`${lines.join('\n',)}\n`,);
process.stdout
  .write(`${String(units.length,)} Pi run${((units.length === 1) ? '' : 's')} going now\n`,);
