#!/usr/bin/env node
/**
 `put-a-set-aside-entry-back-into-a-queue`: puts an entry the filling took out of the way back into the queue, at the head or at a
 position the caller names, from the record its failed start left beside the runs.

 The entry goes in before its record goes away, so a kill between the two leaves the entry queued and the next filling starts it.

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
  runsFolderIn,
} from './parse-queue-of-runs-options.ts';
import {
  putASetAsideEntryBack,
} from './set-aside-entry.ts';
import { usageWasAsked, } from './usage-of-a-command.ts';

/**
 The command's command line, for the error message.
 */
const USAGE = 'put-a-set-aside-entry-back-into-a-queue --queue=<file> --name=<name> [--position=<n>]';

/**
 A command line the command cannot use.
 */
class PutBackOptionsError extends Error {
  /**
   The name printed with the message.
   */
  override readonly name = 'PutBackOptionsError';
}

/**
 Builds the error this command answers a refused option with.

 @param message - what is wrong with the command line

 @returns the error the command exits with

 @example
 ```ts
 throw putBackOptionsErrorOf('--queue is required.',);
 ```
 */
function putBackOptionsErrorOf(message: string,): Error {
  return new PutBackOptionsError(message,);
}

/**
 The words this command's options are read with.

 @returns the usage line and the error it refuses with

 @example
 ```ts
 const words = putBackWords();
 ```
 */
function putBackWords(): QueueOptionWords {
  return {
    usage: USAGE,
    errorOf: putBackOptionsErrorOf,
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
const words = putBackWords();

try {
  /**
   What putting the entry back answered.
   */
  const put = await putASetAsideEntryBack({
    queueFile: queueFileIn({
      commandLine,
      words,
    },),
    runsFolder: runsFolderIn({ commandLine, },),
    name: nameIn({
      commandLine,
      words,
    },),
    position: positionIn({
      commandLine,
      words,
    },),
  },);
  process.stdout
    .write(`${put.line}\n`,);
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
