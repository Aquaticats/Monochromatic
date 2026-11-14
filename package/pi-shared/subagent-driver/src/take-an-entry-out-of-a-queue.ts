#!/usr/bin/env node
/**
 `take-an-entry-out-of-a-queue`: takes the entry of one name out of a queue of runs and prints it, reading the queue and writing
 it back through a rename, so the filling never reads a half-written file.

 It says so and changes nothing when no entry has that name, which happens when the filling started it meanwhile.

 @module
 */

import {
  argv,
} from 'node:process';
import {
  type QueueOptionWords,
  nameIn,
  queueFileIn,
  queueRefusalOf,
} from './parse-queue-of-runs-options.ts';
import {
  readQueueOfRuns,
  takeAQueueEntryOut,
} from './queue-of-runs.ts';
import { usageWasAsked, } from './usage-of-a-command.ts';

/**
 The command's command line, for the error message.
 */
const USAGE = 'take-an-entry-out-of-a-queue --queue=<file> --name=<name>';

/**
 A command line the command cannot use.
 */
class TakeAnEntryOptionsError extends Error {
  /**
   The name printed with the message.
   */
  override readonly name = 'TakeAnEntryOptionsError';
}

/**
 Builds the error this command answers a refused option with.

 @param message - what is wrong with the command line

 @returns the error the command exits with

 @example
 ```ts
 throw takeAnEntryOptionsErrorOf('--queue is required.',);
 ```
 */
function takeAnEntryOptionsErrorOf(message: string,): Error {
  return new TakeAnEntryOptionsError(message,);
}

/**
 The words this command's options are read with.

 @returns the usage line and the error it refuses with

 @example
 ```ts
 const words = takeWords();
 ```
 */
function takeWords(): QueueOptionWords {
  return {
    usage: USAGE,
    errorOf: takeAnEntryOptionsErrorOf,
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
const words = takeWords();

try {
  /**
   The queue the entry is taken from.
   */
  const queueFile = queueFileIn({
    commandLine,
    words,
  },);
  /**
   The entry's name.
   */
  const name = nameIn({
    commandLine,
    words,
  },);
  /**
   The entries taken out, and the queue as it stands afterwards.
   */
  const taken = await takeAQueueEntryOut({
    queueFile,
    name,
  },);
  if (taken.taken
    .length
    === 0) {
    /**
     How many entries the queue holds, read again because the filling may have taken one out meanwhile.
     */
    const queue = await readQueueOfRuns({ queueFile, },);
    process.stdout
      .write(`no entry named ${name} is in the queue of ${String(queue.length,)}; nothing changed\n`,);
  }
  else {
    process.stdout
      .write(`took out ${JSON.stringify(taken.taken,)}; ${String(taken.entries
        .length,)} entries left\n`,);
  }
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
