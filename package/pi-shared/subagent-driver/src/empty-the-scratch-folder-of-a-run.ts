#!/usr/bin/env node
/**
 `empty-the-scratch-folder-of-a-run`: deletes everything inside one run's scratch folder except its `keep/` subfolder, and says how
 much it removed, how much room the volume gained, how much `keep/` holds, and what it had to leave where it is. See `README.md`.

 The folder is computed from the run's name and the scratch root, so this command cannot be pointed at a folder that is not a run's
 own: a folder it is asked for under a name that is not the launcher's is refused with the reason.

 It is what a run's end runs, and a person can run it to empty one folder by hand. Before anything is deleted it proves, there and
 then, that the run which ended still owns the folder: no unit of that name is going and no process of the account works in it. A
 run's scratch folder is named by the run's name, and a name is free as soon as the old unit is collected, so a newer attempt of the
 same name may own the folder while the attempt before it is still ending. A reading that cannot be taken is no proof, and the folder
 stays with the reason said plainly.

 @module
 */

import {
  homedir,
} from 'node:os';
import {
  argv,
  env,
  stdout,
} from 'node:process';
import {
  type ScratchFolderEmptiness,
  emptyTheFolderItsRunStillOwns,
} from './emptying-a-scratch-folder.ts';
import { optionIn, } from './option-in.ts';
import { runCommandWithChildProcess, } from './run-command.ts';
import { requiredText, } from './required-option-values.ts';
import {
  defaultScratchRoot,
  scratchFolderOf,
} from './scratch-folder-of-a-run.ts';
import { scratchEmptyingLinesOf, } from './the-line-an-emptied-scratch-folder-answers-with.ts';
import { usageWasAsked, } from './usage-of-a-command.ts';

/**
 The command's command line, for the error message.
 */
const USAGE =
  'empty-the-scratch-folder-of-a-run --run-name=<name> [--scratch-root=<folder>]';

/**
 The option that names the run whose folder is emptied.
 */
const RUN_NAME_OPTION = '--run-name=';

/**
 The option that names the root every run's scratch folder lives under.
 */
const SCRATCH_ROOT_OPTION = '--scratch-root=';

/**
 The status this command exits with when it refused the folder it was asked for, which is not a deletion that failed but one it
 never began.
 */
const REFUSED_EXIT_STATUS = 1;

/**
 A command line the command cannot use.
 */
class ScratchEmptyingOptionsError extends Error {
  /**
   The name printed with the message.
   */
  override readonly name = 'ScratchEmptyingOptionsError';
}

/**
 Builds the error this command answers a refused option with.

 @param message - what is wrong with the command line

 @returns the error the command exits with

 @example
 ```ts
 throw scratchEmptyingOptionsErrorOf('--run-name is required.',);
 ```
 */
function scratchEmptyingOptionsErrorOf(message: string,): Error {
  return new ScratchEmptyingOptionsError(message,);
}

/**
 The words the command was given.
 */
const words = argv.slice(2,);

// The word that asks for help goes to the command as an argument, and the command answers it before it reads an option
// or touches anything: mise appends a word its task's usage spec does not claim to the task's text, and `--help` is
// mise's own flag, so this is the only place it can land.
if (usageWasAsked({
  words,
  usage: USAGE,
},)) {
  process.exitCode = 0;
  process.exit(0,);
}

/**
 The run whose folder is emptied.
 */
const runName = requiredText({
  value: optionIn({
    commandLine: words,
    option: RUN_NAME_OPTION,
  },),
  name: 'run-name',
  usage: USAGE,
  errorOf: scratchEmptyingOptionsErrorOf,
},);

/**
 The root every run's scratch folder lives under, which rule `ATH` puts at `~/temp/agent`.
 */
const scratchRootGiven = optionIn({
  commandLine: words,
  option: SCRATCH_ROOT_OPTION,
},);
/**
 The root this run of the command empties a folder under.
 */
const scratchRoot = (scratchRootGiven === '')
  ? defaultScratchRoot({ home: env.HOME ?? homedir(), },)
  : scratchRootGiven;

/**
 The run's scratch folder, which is what this command computes and never what a caller hands it.
 */
const folder = scratchFolderOf({
  scratchRoot,
  runName,
},);

/**
 What emptying the folder answered, which is refused unless the run that ended is proven still to own it at this moment.
 */
const emptying: ScratchFolderEmptiness = await emptyTheFolderItsRunStillOwns({
  folder,
  scratchRoot,
  runName,
  run: runCommandWithChildProcess,
  processFolder: env.PI_RUNS_PROCESS_FOLDER ?? '/proc',
},);

stdout.write(`${scratchEmptyingLinesOf({
  runName,
  folder,
  emptying,
},)
  .filter(function isALine(line,) {
    return line.trim() !== '';
  },)
  .join('\n',)}\n`,);
process.exitCode = (emptying.emptied === 'refused') ? REFUSED_EXIT_STATUS : 0;
