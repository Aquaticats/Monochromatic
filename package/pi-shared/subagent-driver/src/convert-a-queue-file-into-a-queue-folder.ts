#!/usr/bin/env node
/**
 `convert-a-queue-file-into-a-queue-folder`: carries a queue this package kept as one JSON array into the queue folder it keeps now, one
 file per entry, and renames the array aside once every entry of it is in the folder. Running it again passes over what is already there.

 @module
 */

import {
  argv,
} from 'node:process';
import { convertAQueueFileIntoAFolder, } from './converting-a-queue-file-into-a-folder.ts';
import {
  type QueueOptionWords,
  queueFolderIn,
  queueRefusalOf,
} from './parse-queue-of-runs-options.ts';
import { optionIn, } from './option-in.ts';
import { requiredText, } from './required-option-values.ts';
import { usageWasAsked, } from './usage-of-a-command.ts';

/**
 The command's command line, for the error message.
 */
const USAGE = 'convert-a-queue-file-into-a-queue-folder --queue=<folder> --from=<file>';

/**
 The option that names the array file the queue was kept as.
 */
const FROM_OPTION = '--from=';

/**
 A command line the command cannot use.
 */
class ConvertTheQueueOptionsError extends Error {
  /**
   The name printed with the message.
   */
  override readonly name = 'ConvertTheQueueOptionsError';
}

/**
 Builds the error this command answers a refused option with.

 @param message - what is wrong with the command line

 @returns the error the command exits with

 @example
 ```ts
 throw convertTheQueueOptionsErrorOf('--from is required.',);
 ```
 */
function convertTheQueueOptionsErrorOf(message: string,): Error {
  return new ConvertTheQueueOptionsError(message,);
}

/**
 The words this command's options are read with.

 @returns the usage line and the error it refuses with

 @example
 ```ts
 const words = convertWords();
 ```
 */
function convertWords(): QueueOptionWords {
  return {
    usage: USAGE,
    errorOf: convertTheQueueOptionsErrorOf,
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
const words = convertWords();

try {
  /**
   What the changeover did.
   */
  const done = await convertAQueueFileIntoAFolder({
    queueFile: requiredText({
      value: optionIn({
        commandLine,
        option: FROM_OPTION,
      },),
      name: FROM_OPTION.replaceAll(
        '=',
        '',
      ),
      usage: USAGE,
      errorOf: convertTheQueueOptionsErrorOf,
    },),
    queueFolder: queueFolderIn({
      commandLine,
      words,
    },),
  },);
  process.stdout
    .write(`carried ${String(done.entries,)} ${done.entries === 1 ? 'entry' : 'entries'} into the queue folder, passed over ${String(done.alreadyThere,)} already there, and left the array at ${done.arrayKeptAt}\n`,);
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
