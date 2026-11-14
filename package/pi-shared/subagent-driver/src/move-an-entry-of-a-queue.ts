#!/usr/bin/env node
/**
 `move-an-entry-of-a-queue`: moves the entry of one name to a position counted from the queue's head, reading the queue and writing
 it back through a rename, so the filling never reads a half-written file.

 It says so and changes nothing when no entry has that name.

 @module
 */

import {
  argv,
} from 'node:process';
import {
  type QueueOptionWords,
  nameIn,
  positionIn,
  queueFileIn,
  queueRefusalOf,
} from './parse-queue-of-runs-options.ts';
import {
  moveAQueueEntryTo,
  readQueueOfRuns,
} from './queue-of-runs.ts';
import { usageWasAsked, } from './usage-of-a-command.ts';

/**
 The command's command line, for the error message.
 */
const USAGE = 'move-an-entry-of-a-queue --queue=<file> --name=<name> --position=<n>';

/**
 A command line the command cannot use.
 */
class MoveAnEntryOptionsError extends Error {
  /**
   The name printed with the message.
   */
  override readonly name = 'MoveAnEntryOptionsError';
}

/**
 Builds the error this command answers a refused option with.

 @param message - what is wrong with the command line

 @returns the error the command exits with

 @example
 ```ts
 throw moveAnEntryOptionsErrorOf('--queue is required.',);
 ```
 */
function moveAnEntryOptionsErrorOf(message: string,): Error {
  return new MoveAnEntryOptionsError(message,);
}

/**
 The words this command's options are read with.

 @returns the usage line and the error it refuses with

 @example
 ```ts
 const words = moveWords();
 ```
 */
function moveWords(): QueueOptionWords {
  return {
    usage: USAGE,
    errorOf: moveAnEntryOptionsErrorOf,
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
const words = moveWords();

try {
  /**
   The queue the entry is moved in.
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
   Where it was and where it went.
   */
  const moved = await moveAQueueEntryTo({
    queueFile,
    name,
    position: positionIn({
      commandLine,
      words,
    },),
  },);
  process.stdout
    .write(((moved.from === undefined) || (moved.to === undefined))
      ? `no entry named ${name} is in the queue of ${String((await readQueueOfRuns({ queueFile, },)).length,)}; nothing changed\n`
      : `moved ${name} from position ${String(moved.from,)} to ${String(moved.to,)} of ${String(moved.count,)}\n`,);
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
