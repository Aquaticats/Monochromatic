#!/usr/bin/env node
/**
 `list-the-queue-of-runs`: prints every entry of a queue in the order it will start, with the place it holds, the identity its write minted,
 what it is, and which filling holds it now. A queue is a folder of one file per entry, so this is the one command that shows the whole queue
 in its order.

 @module
 */

import {
  argv,
} from 'node:process';
import type {
  QueueRecord,
} from './queue-of-runs.ts';
import {
  readQueueOfRuns,
} from './reading-and-writing-the-folder-a-queue-is.ts';
import { shapeOfAQueueEntry, } from './entry-of-a-queue.ts';
import { readTheClaimsOnAQueue, } from './reading-the-claims-beside-a-queue.ts';
import {
  type QueueOptionWords,
  queueFolderIn,
  queueRefusalOf,
} from './parse-queue-of-runs-options.ts';
import { usageWasAsked, } from './usage-of-a-command.ts';

/**
 The command's command line, for the error message.
 */
const USAGE = 'list-the-queue-of-runs --queue=<folder>';

/**
 A command line the command cannot use.
 */
class ListTheQueueOptionsError extends Error {
  /**
   The name printed with the message.
   */
  override readonly name = 'ListTheQueueOptionsError';
}

/**
 Builds the error this command answers a refused option with.

 @param message - what is wrong with the command line

 @returns the error the command exits with

 @example
 ```ts
 throw listTheQueueOptionsErrorOf('--queue is required.',);
 ```
 */
function listTheQueueOptionsErrorOf(message: string,): Error {
  return new ListTheQueueOptionsError(message,);
}

/**
 The words this command's options are read with.

 @returns the usage line and the error it refuses with

 @example
 ```ts
 const words = listWords();
 ```
 */
function listWords(): QueueOptionWords {
  return {
    usage: USAGE,
    errorOf: listTheQueueOptionsErrorOf,
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
const words = listWords();

try {
  /**
   The queue's folder.
   */
  const queueFolder = queueFolderIn({
    commandLine,
    words,
  },);
  /**
   The entries, in the order they will start.
   */
  const entries = await readQueueOfRuns({ queueFolder, },);
  /**
   The claims the queue holds, by the entry each is on.
   */
  const claims = await readTheClaimsOnAQueue({ queueFolder, },);
  entries.forEach(function say(
    entry: QueueRecord,
    place: number,
  ) {
    /**
     What this package reads the entry as.
     */
    const shape = shapeOfAQueueEntry({ record: entry, },);
    /**
     What the entry is.
     */
    const what = (shape.kind === 'unreadable') ? `unreadable: ${shape.reason}` : shape.kind;
    /**
     The claim on this entry, when a filling holds one.
     */
    const held = claims.find(function isOnThisEntry(claim,) {
      return (claim.name === entry.name);
    },);
    process.stdout
      .write(`${String(place + 1,)}. ${entry.name} (${what}); place ${String(entry.order,)}; identity ${String(entry.queueId,)}${(held === undefined) ? '' : `; claimed by process ${String(held.claim
        .pid,)} since ${held.claim
          .at}`}\n`,);
  },);
  process.stdout
    .write(`${String(entries.length,)} ${entries.length === 1 ? 'entry' : 'entries'} in the queue at ${queueFolder}, in the order they start\n`,);
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
