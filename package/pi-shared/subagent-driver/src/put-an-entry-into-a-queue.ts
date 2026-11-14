#!/usr/bin/env node
/**
 `put-an-entry-into-a-queue`: puts one entry into a queue of runs at a position counted from its head, reading the queue and
 writing it back through a rename, so the filling never reads a half-written file.

 See `README.md` for the entry shapes. An entry of that name is refused rather than queued twice.

 @module
 */

import {
  argv,
} from 'node:process';
import {
  type QueueOptionWords,
  entryIn,
  positionIn,
  queueFileIn,
  queueRefusalOf,
} from './parse-queue-of-runs-options.ts';
import { putAQueueEntryIn, } from './queue-of-runs.ts';
import { usageWasAsked, } from './usage-of-a-command.ts';

/**
 The command's command line, for the error message.
 */
const USAGE = 'put-an-entry-into-a-queue --queue=<file> --entry=<json> [--position=<n>]';

/**
 A command line the command cannot use.
 */
class PutAnEntryOptionsError extends Error {
  /**
   The name printed with the message.
   */
  override readonly name = 'PutAnEntryOptionsError';
}

/**
 Builds the error this command answers a refused option with.

 @param message - what is wrong with the command line

 @returns the error the command exits with

 @example
 ```ts
 throw putAnEntryOptionsErrorOf('--queue is required.',);
 ```
 */
function putAnEntryOptionsErrorOf(message: string,): Error {
  return new PutAnEntryOptionsError(message,);
}

/**
 The words this command's options are read with.

 @returns the usage line and the error it refuses with

 @example
 ```ts
 const words = putWords();
 ```
 */
function putWords(): QueueOptionWords {
  return {
    usage: USAGE,
    errorOf: putAnEntryOptionsErrorOf,
  };
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
 The words the command was given.
 */
const commandLine = argv.slice(2,);

/**
 The words the options are read with.
 */
const words = putWords();

try {
  /**
   The entry to queue.
   */
  const entry = entryIn({
    commandLine,
    words,
  },);
  /**
   Where it landed, and how many entries the queue holds.
   */
  const queued = await putAQueueEntryIn({
    queueFile: queueFileIn({
      commandLine,
      words,
    },),
    entry,
    position: positionIn({
      commandLine,
      words,
    },),
  },);
  process.stdout
    .write(`queued ${entry.name} at position ${String(queued.position,)} of ${String(queued.count,)}\n`,);
}
catch (error) {
  process.stderr
    .write(`${queueRefusalOf({
      error,
      words,
    },)
      .message}\n`,);
  process.exitCode = 1;
}
