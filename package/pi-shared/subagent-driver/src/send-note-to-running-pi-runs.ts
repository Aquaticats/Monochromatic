#!/usr/bin/env node
/**
 `send-note-to-running-pi-runs`: publishes one note into the inbox of every Pi run that is going, under a name that is written
 apart and renamed, so the driver never reads half of it. It prints one line per run.

 Usage: `send-note-to-running-pi-runs --note=<file> --name=<inbox name> [--skip=<run>]... [--runs-folder=<folder>]`

 @module
 */

import { readFile, } from 'node:fs/promises';
import {
  argv,
  stdout,
} from 'node:process';
import { parseArgs, } from 'node:util';
import {
  A_USABLE_NAME,
  noteNameProblemOf,
  notesToRuns,
} from './note-to-running-pi-runs.ts';
import { defaultRunsFolder, } from './pi-runs-folders.ts';
import { runCommandWithChildProcess, } from './run-command.ts';
import { startRunningPiUnits, } from './running-pi-units.ts';

/**
 The command's command line, for the error message.
 */
const USAGE =
  'send-note-to-running-pi-runs --note=<file> --name=<inbox name> [--skip=<run>]... [--runs-folder=<folder>]';

/**
 The options as written.
 */
const { values, } = parseArgs({
  args: argv.slice(2,),
  strict: true,
  allowPositionals: false,
  options: {
    note: { type: 'string', },
    name: { type: 'string', },
    skip: {
      type: 'string',
      multiple: true,
      default: [],
    },
    'runs-folder': { type: 'string', },
  },
},);

if ((values.note === undefined) || (values.name === undefined)) {
  throw new Error(`--note and --name are required\nUsage: ${USAGE}`,);
}

/**
 Why the note's name cannot be used, when it cannot be.
 */
const problem = noteNameProblemOf({ noteName: values.name, },);

if ((typeof problem) !== 'symbol') {
  throw new Error(`${problem}\nUsage: ${USAGE}`,);
}

if (problem !== A_USABLE_NAME) {
  throw new Error(`${problem}\nUsage: ${USAGE}`,);
}

/**
 Folder the driver writes one run folder per run into.
 */
const runsFolder = values['runs-folder'] ?? defaultRunsFolder({},);

/**
 Every Pi run going now.
 */
const units = await startRunningPiUnits({ run: runCommandWithChildProcess, },)();

/**
 What became of every run.
 */
const lines = await notesToRuns({
  runsFolder,
  note: await readFile(
    values.note,
    'utf8',
  ),
  noteName: values.name,
  runs: units.map(function nameOf(unit,) {
    return unit.name;
  },),
  skip: values.skip ?? [],
},);

stdout.write(`${lines.join('\n',)}\n`,);
