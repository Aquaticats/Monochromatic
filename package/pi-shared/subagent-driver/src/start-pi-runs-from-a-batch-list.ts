#!/usr/bin/env node
/**
 `start-pi-runs-from-a-batch-list`: starts every entry of a batch list through the retrying start, one after another, so each waits
 for a slot that frees as the runs before it end. See `README.md` for the list's fields.

 Both of a start's streams are kept: a start that printed why it could not run on standard output and nothing else once left a
 batch's failures unexplained. An entry's line says the reason its start printed, and the whole of what it printed is appended to a
 file beside the runs that the line names.

 @module
 */

import { readFile, } from 'node:fs/promises';
import {
  argv,
  env,
} from 'node:process';
import { setTimeout as wait, } from 'node:timers/promises';
import {
  batchEntriesIn,
  startBatch,
} from './batch-list.ts';
import {
  keepFailedOutput,
  reportOnStandardError,
} from './kept-start-failure.ts';
import { parseBatchOptions, } from './parse-batch-options.ts';
import {
  retryingStartEntry,
  startThroughTheRetryingStart,
} from './start-through-the-retrying-start.ts';

/**
 What this run of the command was asked for.
 */
const options = parseBatchOptions(argv.slice(2,),);

/**
 The retrying start this package ships beside this command.
 */
const retryingStart = await retryingStartEntry();

/**
 The list of runs to start.
 */
const entries = batchEntriesIn({
  shown: await readFile(
    options.batchFile,
    'utf8',
  ),
  listFile: options.batchFile,
},);

/**
 The line each entry ended with.
 */
const lines = await startBatch({
  entries,
  runsFolder: options.runsFolder,
  deadlineSeconds: options.deadlineSeconds,
  start: function startOne({ words, },) {
    return startThroughTheRetryingStart({
      entry: retryingStart,
      words,
      environment: env,
    },);
  },
  wait,
  keepFailure: function keepFailure({
    name,
    output,
  },) {
    return keepFailedOutput({
      runsFolder: options.runsFolder,
      name,
      output,
      report: reportOnStandardError,
    },);
  },
},);

for (const line of lines) {
  process.stdout
    .write(`${line}\n`,);
}
