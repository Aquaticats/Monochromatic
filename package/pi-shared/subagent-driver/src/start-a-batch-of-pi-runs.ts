#!/usr/bin/env node
/**
 `start-a-batch-of-pi-runs`: starts every entry of a batch list, each through its own queue waiting for a slot, and prints one
 line per entry when all of them have started.

 Usage: `start-a-batch-of-pi-runs <list.json> [deadline-seconds]`, where the list is a JSON array of entries and each entry's
 `prompt` is a path relative to the runs folder. An entry naming `run` and `sessionOf` continues a run this batch started
 before, which is what makes a batch resumable after its deadline.

 @module
 */

import { spawn, } from 'node:child_process';
import { caughtValueText, } from '@monochromatic-dev/caught-value-core';
import { once, } from 'node:events';
import { readFile, } from 'node:fs/promises';
import {
  extname,
  isAbsolute,
  join,
} from 'node:path';
import process, {
  argv,
  stdout,
} from 'node:process';
import { setTimeout as wait, } from 'node:timers/promises';
import {
  BATCH_PERIOD_SECONDS,
  BATCH_STAGGER_MILLISECONDS,
  type BatchEntry,
  DEFAULT_BATCH_CPU_PERCENT,
  DEFAULT_BATCH_MEMORY,
  DEFAULT_BATCH_TASKS,
  DEFAULT_BATCH_WALL_SECONDS,
  parseBatchList,
  startABatch,
  type StartedEntry,
} from './batch-of-pi-runs.ts';
import {
  defaultRunsFolder,
  promptFileOf,
} from './pi-runs-folders.ts';

/**
 What a queue prints about a failure, kept so one line of it can be reported.
 */
const ERRORS_KEPT_CHARACTERS = 400;

/**
 How long a batch waits for a slot when the caller names no deadline.
 */
const DEFAULT_DEADLINE_SECONDS = 7_200;

/**
 The name of the queue command, which sits beside this one.
 */
const RETRY_COMMAND_NAME = 'retry-start-pi-run-until-a-slot-frees';

/**
 The command's command line, for the error message.
 */
const USAGE = 'start-a-batch-of-pi-runs <list.json> [deadline-seconds]';

/**
 Folder the driver writes one run folder per run into.
 */
const runsFolder = defaultRunsFolder({},);

/**
 The list of entries to start, and how long each of them may wait for a slot.
 */
const [listArgument, deadlineArgument,] = argv.slice(2,);

if (listArgument === undefined) {
  throw new Error(`a batch list is required\nUsage: ${USAGE}`,);
}

/**
 The queue command, beside this one.
 */
const retryScript = join(
  import.meta.dirname,
  `${RETRY_COMMAND_NAME}${extname(import.meta.filename,)}`,
);

/**
 How long an entry may wait for a slot.
 */
const deadlineSeconds = deadlineArgument ?? String(DEFAULT_DEADLINE_SECONDS,);

/**
 Reads the entry's prompt file, which a list may write relative to the runs folder.

 @param entry - the entry to read the prompt file for

 @returns the prompt file's path

 @example
 ```ts
 const prompt = promptFileFor({ entry: { name: 'a', branch: 'b', }, });
 ```
 */
function promptFileFor({
  entry,
}: {
  readonly entry: BatchEntry;
},): string {
  if (entry.prompt === undefined) {
    return promptFileOf({
      runsFolder,
      name: entry.name,
    },);
  }
  return isAbsolute(entry.prompt,) ? entry.prompt : join(
    runsFolder,
    entry.prompt,
  );
}

/**
 Builds one entry's start arguments.

 @param entry - the entry to start

 @returns the arguments of `start-pi-run-in-a-free-slot`

 @example
 ```ts
 const words = startArgumentsOf({ entry: { name: 'a', branch: 'b', }, });
 ```
 */
function startArgumentsOf({
  entry,
}: {
  readonly entry: BatchEntry;
},): readonly string[] {
  return [
    `--name=${entry.name}`,
    `--branch=${entry.branch}`,
    `--prompt=${promptFileFor({ entry, },)}`,
    `--cpu=${String(entry.cpu ?? DEFAULT_BATCH_CPU_PERCENT,)}`,
    `--memory=${entry.memory ?? DEFAULT_BATCH_MEMORY}`,
    `--tasks=${String(entry.tasks ?? DEFAULT_BATCH_TASKS,)}`,
    `--wall=${String(entry.wall ?? DEFAULT_BATCH_WALL_SECONDS,)}`,
    `--runs-folder=${runsFolder}`,
    ...((entry.run === undefined) ? [] : [`--run=${String(entry.run,)}`,]),
    ...((entry.sessionOf === undefined) ? [] : [`--session-of=${entry.sessionOf}`,]),
  ];
}

/**
 Reads the last line a queue printed, which is what its failure says.

 @param errors - what the queue complained about

 @returns the last line that holds anything, or empty text when it complained about nothing

 @example
 ```ts
 const line = lastLineOf({ errors: 'a\nb\n', });
 ```
 */
function lastLineOf({
  errors,
}: {
  readonly errors: string;
},): string {
  return errors.split('\n',)
    .filter(function holdsSomething(line,) {
      return line.trim() !== '';
    },)
    .at(-1,)
    ?.trim()
    ?? '';
}

/**
 Starts one entry by asking its own queue for a slot.

 @param entry - the entry to start

 @returns what that entry ended with

 @example
 ```ts
 const started = await startEntry({ name: 'a', branch: 'b', });
 ```
 */
async function startEntry(entry: BatchEntry,): Promise<StartedEntry> {
  /**
   The queue, writing nothing of its own so one line per entry is what a caller reads.
   */
  const child = spawn(
    process.execPath,
    [
      retryScript,
      `--deadline-seconds=${deadlineSeconds}`,
      `--period-seconds=${String(BATCH_PERIOD_SECONDS,)}`,
      '--',
      ...startArgumentsOf({ entry, },),
    ],
    {
      cwd: runsFolder,
      stdio: [
        'ignore',
        'ignore',
        'pipe',
      ],
    },
  );
  /**
   What the queue has said so far: the end of its complaints, which is what a line reports, and how it ended.
   */
  const heard: { errors: string } = { errors: '', };
  child.stderr
    .setEncoding('utf8',);
  child.stderr
    .on(
    'data',
    function keepTheEnd(chunk: string,) {
      heard.errors = `${heard.errors}${chunk}`.slice(-ERRORS_KEPT_CHARACTERS,);
    },
    );
  try {
    /**
     How the queue ended, as its own event reports it.
     */
    const [code,] = await once(
      child,
      'close',
    ) as [unknown];
    return {
      name: entry.name,
      status: ((typeof code) === 'number') ? code : -1,
      lastLine: lastLineOf({ errors: heard.errors, },),
    };
  }
  catch (error) {
    return {
      name: entry.name,
      status: -1,
      lastLine: `could not start the queue (${caughtValueText(error,)})`.slice(
        0,
        ERRORS_KEPT_CHARACTERS,
      ),
    };
  }
}

/**
 The list's path: a path relative to the runs folder is what a coordinator writes, and an absolute one is used as it stands.
 */
const listPath = isAbsolute(listArgument,) ? listArgument : join(
  runsFolder,
  listArgument,
);

/**
 The entries the list holds.
 */
const entries = parseBatchList({ text: await readFile(
  listPath,
  'utf8',
), },);

/**
 One line per entry, once every one of them has started.
 */
const lines = await startABatch({
  entries,
  startEntry,
  wait: async function passTime(milliseconds,) {
    await wait(milliseconds,);
  },
  staggerMilliseconds: BATCH_STAGGER_MILLISECONDS,
},);

stdout.write(`${lines.join('\n',)}\n`,);
