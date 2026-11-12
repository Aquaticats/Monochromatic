#!/usr/bin/env node
/**
 `start-pi-runs-from-a-batch-list`: starts every entry of a batch list through the retrying start, one after another, so each waits
 for a slot that frees as the runs before it end. See `README.md` for the list's fields.

 Both of a start's streams are kept: a start that printed why it could not run on standard output and nothing else once left a
 batch's failures unexplained. An entry's line says the reason its start printed, and the whole of what it printed is appended to a
 file beside the runs that the line names.

 @module
 */

import { spawn, } from 'node:child_process';
import { once, } from 'node:events';
import {
  appendFile,
  readFile,
} from 'node:fs/promises';
import { join, } from 'node:path';
import {
  argv,
  env,
} from 'node:process';
import { setTimeout as wait, } from 'node:timers/promises';
import {
  batchEntriesIn,
  ENTRY_OUTPUT_KEPT_CHARACTERS,
  type StartedEntry,
  startBatch,
} from './batch-list.ts';
import { parseBatchOptions, } from './parse-batch-options.ts';
import { siblingCommandEntry, } from './sibling-command-entry.ts';
import { tailKeeper, } from './tail-of-a-command.ts';

/**
 What this run of the command was asked for.
 */
const options = parseBatchOptions(argv.slice(2,),);

/**
 The retrying start this package ships beside this command.
 */
const retryingStart = await siblingCommandEntry({
  folder: import.meta.dirname,
  command: 'retry-pi-run-start-until-a-pool-has-room',
},);

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
 Starts one entry through the retrying start, keeping both streams.

 @param words - the retrying start's words

 @returns how it ended, and the end of what it printed

 @example
 ```ts
 const started = await startOne({ words: ['--deadline-seconds=7200',], });
 ```
 */
async function startOne({ words, }: { readonly words: readonly string[] },): Promise<StartedEntry> {
  /**
   The retrying start's process.
   */
  const child = spawn(
    process.execPath,
    [
      retryingStart,
      ...words,
    ],
    {
      stdio: [
        'ignore',
        'pipe',
        'pipe',
      ],
      env,
    },
  );
  /**
   The end of what it printed, on either stream.
   */
  const printed = tailKeeper({ characters: ENTRY_OUTPUT_KEPT_CHARACTERS, },);

  child.stdout
    .on(
    'data',
    printed.keep,
    );
  child.stderr
    .on(
    'data',
    printed.keep,
    );
  try {
    /**
     What the process reported when it closed, which is its status or the signal that ended it.
     */
    const closed: readonly unknown[] = await once(
      child,
      'close',
    );
    /**
     The status the retrying start exited with, or nothing when a signal ended it.
     */
    const [status,] = closed;
    return {
      output: printed.text(),
      ...(((typeof status) === 'number') ? { status, } : {}),
    };
  }
  catch (error) {
    // `once` rejects when the process could not be started at all, and a batch reports that as the entry's own reason.
    return {
      output: printed.text(),
      startProblem: String(error,),
    };
  }
}

/**
 Keeps the whole of what a failed start printed, so the one line the batch prints can stay short and still be enough to find out
 more. The file sits beside the run folders and appends, so a name that failed twice keeps both attempts.

 @param name - the run's name

 @param output - what the start printed on both streams

 @returns the file the output was kept in, or nothing when it could not be written

 @example
 ```ts
 const keptAt = await keepFailedOutput({ name: 'a', output, },);
 ```
 */
async function keepFailedOutput({
  name,
  output,
}: {
  readonly name: string;
  readonly output: string;
},): Promise<{ readonly keptAt?: string }> {
  /**
   The file this run's failed attempts are kept in.
   */
  const file = join(
    options.runsFolder,
    `${name}-start-failure.log`,
  );
  try {
    await appendFile(
      file,
      `=== ${new Date().toISOString()} ===\n${output.trimEnd()}\n`,
    );
    return { keptAt: file, };
  }
  catch (error) {
    // The line still has to say why the start failed, so a file that could not be written is reported and the line goes without it.
    process.stderr
      .write(`the output of ${name} could not be kept in ${file}: ${String(error,)}\n`,);
    return {};
  }
}

/**
 The line each entry ended with.
 */
const lines = await startBatch({
  entries,
  runsFolder: options.runsFolder,
  deadlineSeconds: options.deadlineSeconds,
  start: startOne,
  wait,
  keepFailure: keepFailedOutput,
},);

for (const line of lines) {
  process.stdout
    .write(`${line}\n`,);
}
