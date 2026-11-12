#!/usr/bin/env node
/**
 `start-a-batch-in-its-own-unit`: starts a batch list through `start-pi-runs-from-a-batch-list` in a systemd user unit of its own,
 so the batch, its retries and every launcher it starts outlive the process that asked for them. See `README.md` for the options.

 The unit is named `launch-<the list's name>`, never `pi-*`: the slot count reads every `pi-*` unit as a run. Its output goes to a
 file beside the list, and nowhere else: a unit has no terminal.

 @module
 */

import {
  argv,
  env,
  stdout,
} from 'node:process';
import {
  BATCH_DEFAULT_DEADLINE_SECONDS,
} from './batch-list.ts';
import {
  batchOutputFileOf,
  batchUnitCommandOf,
  batchUnitNameOf,
  startedLineOf,
} from './batch-run-as-a-unit.ts';
import { optionIn, } from './option-in.ts';
import {
  requiredOutputOf,
} from './required-output-of-a-command.ts';
import {
  requiredPositiveNumber,
  requiredText,
} from './required-option-values.ts';
import { runCommandWithChildProcess, } from './run-command.ts';
import { siblingCommandEntry, } from './sibling-command-entry.ts';

/**
 The command's command line, for the error message.
 */
const USAGE =
  'start-a-batch-in-its-own-unit --batch-file=<file> [--deadline-seconds=<n>] [--output-file=<file>]';

/**
 The option that names the batch list.
 */
const BATCH_FILE_OPTION = '--batch-file=';

/**
 The option that names how long the batch keeps asking for a slot.
 */
const DEADLINE_OPTION = '--deadline-seconds=';

/**
 The option that names where the unit writes its output.
 */
const OUTPUT_FILE_OPTION = '--output-file=';

/**
 A command line the command cannot use.
 */
class BatchUnitOptionsError extends Error {
  /**
   The name printed with the message.
   */
  override readonly name = 'BatchUnitOptionsError';
}

/**
 Builds the error this command answers a refused option with.

 @param message - what is wrong with the command line

 @returns the error the command exits with

 @example
 ```ts
 throw batchUnitOptionsErrorOf('--batch-file is required.',);
 ```
 */
function batchUnitOptionsErrorOf(message: string,): Error {
  return new BatchUnitOptionsError(message,);
}

/**
 The words the command was given.
 */
const words = argv.slice(2,);

/**
 The batch list the unit starts runs from.
 */
const batchFile = requiredText({
  value: optionIn({
    commandLine: words,
    option: BATCH_FILE_OPTION,
  },),
  name: 'batch-file',
  usage: USAGE,
  errorOf: batchUnitOptionsErrorOf,
},);

/**
 The deadline, as the caller wrote it.
 */
const deadlineGiven = optionIn({
  commandLine: words,
  option: DEADLINE_OPTION,
},);

/**
 The output file, as the caller wrote it.
 */
const outputGiven = optionIn({
  commandLine: words,
  option: OUTPUT_FILE_OPTION,
},);

/**
 The unit's name, which every line about it names.
 */
const unit = batchUnitNameOf({ batchFile, },);

/**
 The command that starts the batch's unit.
 */
const command = batchUnitCommandOf({
  batchFile,
  deadlineSeconds: (deadlineGiven === '')
    ? BATCH_DEFAULT_DEADLINE_SECONDS
    : requiredPositiveNumber({
      value: deadlineGiven,
      name: 'deadline-seconds',
      usage: USAGE,
      errorOf: batchUnitOptionsErrorOf,
    },),
  outputFile: (outputGiven === '') ? batchOutputFileOf({ batchFile, },) : outputGiven,
  pathOfMiseEnvironment: await requiredOutputOf({
    run: runCommandWithChildProcess,
    command: 'mise',
    words: [
      'exec',
      '--',
      'printenv',
      'PATH',
    ],
  },),
  home: env.HOME ?? '',
  node: process.execPath,
  batchEntry: await siblingCommandEntry({
    folder: import.meta.dirname,
    command: 'start-pi-runs-from-a-batch-list',
  },),
},);

/**
 What systemd-run answered.
 */
const started = await runCommandWithChildProcess({
  command: command.command,
  words: command.words,
},);

stdout.write(`${startedLineOf({
  unit,
  outputFile: optionIn({
    commandLine: command.words,
    option: '--property=StandardOutput=append:',
  },),
  status: started.status,
  problem: started.problem
    ?? started.stderr
      .trim(),
},)}\n`,);
process.exitCode = (started.status === 0) ? 0 : 1;
