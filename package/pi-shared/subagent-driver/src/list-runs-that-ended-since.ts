#!/usr/bin/env node
/**
 `list-runs-that-ended-since`: prints the Pi runs that ended since a time, oldest first, each with the reason its summary gives,
 the status Pi left with and the head of its last message. See `README.md` for the options.

 Nothing here ends or signals a run: it reads the summaries under the runs folder.

 @module
 */

import { homedir, } from 'node:os';
import { join, } from 'node:path';
import { argv, } from 'node:process';
import { optionIn, } from './option-in.ts';
import { requiredText, } from './required-option-values.ts';
import { DEFAULT_RUNS_FOLDER_IN_HOME, } from './reviewer-of-a-change.ts';
import {
  endedRunLineOf,
  endedRunsSince,
  timeIn,
} from './runs-that-ended-since.ts';
import { usageWasAsked, } from './usage-of-a-command.ts';

/**
 The command's command line, for the error message.
 */
const USAGE = 'list-runs-that-ended-since --since=<ISO time> [--runs-folder=<folder>]';

/**
 The option that names the time to read from.
 */
const SINCE_OPTION = '--since=';

/**
 The option that names the folder the driver writes one run folder per run into.
 */
const RUNS_FOLDER_OPTION = '--runs-folder=';

/**
 A command line this command cannot use.
 */
export class EndedRunsOptionsError extends Error {
  /**
   The name printed with the message.
   */
  override readonly name = 'EndedRunsOptionsError';
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
 The time the caller asked for.
 */
const asked = timeIn({
  written: requiredText({
    value: optionIn({
      commandLine: words,
      option: SINCE_OPTION,
    },),
    name: 'since',
    usage: USAGE,
    errorOf: function endedRunsOptionsErrorOf(message,) {
      return new EndedRunsOptionsError(message,);
    },
  },),
},);

if (asked.since === undefined) {
  throw new EndedRunsOptionsError(
    `--since names no time a reader can follow, such as 2026-11-12T10:00:00-05:00.\nUsage: ${USAGE}`,
  );
}

/**
 The folder the runs live in.
 */
const runsFolder = optionIn({
  commandLine: words,
  option: RUNS_FOLDER_OPTION,
},) || join(
  homedir(),
  ...DEFAULT_RUNS_FOLDER_IN_HOME,
);

/**
 The runs that ended since then, oldest first.
 */
const ended = await endedRunsSince({
  runsFolder,
  since: asked.since,
},);

for (const run of ended) {
  process.stdout
    .write(`${endedRunLineOf(run,)}\n`,);
}
process.stdout
  .write(`${String(ended.length,)} run${((ended.length === 1) ? '' : 's')} ended since ${asked.since
    ? new Date(asked.since,)
      .toISOString()
    : ''}\n`,);
