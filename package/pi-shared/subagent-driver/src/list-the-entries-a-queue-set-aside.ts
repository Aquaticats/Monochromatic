#!/usr/bin/env node
/**
 `list-the-entries-a-queue-set-aside`: names every entry the filling took out of the way, with the time it was set aside and the
 reason its start failed, so a person sees what is waiting to be put back without reading the journal.

 Each line names the entry's own record file, which holds the entry whole: `put-a-set-aside-entry-back-into-a-queue` puts it back
 from there.

 @module
 */

import {
  argv,
} from 'node:process';
import { caughtValueText, } from '@monochromatic-dev/caught-value-core/ts';
import {
  type QueueOptionWords,
  runsFolderIn,
} from './parse-queue-of-runs-options.ts';
import {
  setAsideFileOf,
  setAsideRecordsIn,
} from './set-aside-entry.ts';
import { usageWasAsked, } from './usage-of-a-command.ts';

/**
 The command's command line, for the error message.
 */
const USAGE = 'list-the-entries-a-queue-set-aside [--runs-folder=<folder>]';

/**
 A command line the command cannot use.
 */
class ListSetAsideOptionsError extends Error {
  /**
   The name printed with the message.
   */
  override readonly name = 'ListSetAsideOptionsError';
}

/**
 Builds the error this command answers a refused option with.

 @param message - what is wrong with the command line

 @returns the error the command exits with

 @example
 ```ts
 throw listSetAsideOptionsErrorOf('--queue is required.',);
 ```
 */
function listSetAsideOptionsErrorOf(message: string,): Error {
  return new ListSetAsideOptionsError(message,);
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
    errorOf: listSetAsideOptionsErrorOf,
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

try {
  /**
   The folder the runs and the set-aside records live in.
   */
  const runsFolder = runsFolderIn({ commandLine: argv.slice(2,), },);
  /**
   Every entry set aside, in the order the folder lists them.
   */
  const records = await setAsideRecordsIn({ runsFolder, },);
  for (const record of records) {
    process.stdout
      .write(`${record.setAsideAt} ${record.name} (${record.entry
        .reviewOfRange
        === undefined ? 'a run' : 'a review of a range'}): ${record.reason}\n`,);
    process.stdout
      .write(`  its record, holding the entry whole: ${setAsideFileOf({
        runsFolder,
        name: record.name,
      },)}\n`,);
    if (record.keptOutputAt !== undefined) {
      process.stdout
        .write(`  all its start printed: ${record.keptOutputAt}\n`,);
    }
  }
  process.stdout
    .write(`${String(records.length,)} ${records.length === 1 ? 'entry is' : 'entries are'} set aside beside the runs in ${runsFolder}\n`,);
}
catch (error) {
  process.stderr
    .write(`${listWords()
      .errorOf(caughtValueText(error,),)
      .message}\n`,);
  process.exitCode = 1;
}
